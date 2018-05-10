use std::collections::HashSet;
use std::error::Error as StdError;
use std::fmt;
use std::fs::File;
use std::io::{Read, BufReader};
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use esprit::script;
use esprit::error::Error as EspritError;
use quicli::prelude::Result; // TODO use `failure`?
use serde_json;
use sha1::{Sha1, Digest};
use node_resolve::Resolver;
use estree_detect_requires::detect;
use builtins::{Builtins, NodeBuiltins, NoBuiltins};
use graph::{ModuleMap, Hash, Dependency, Dependencies, ModuleRecord};

#[derive(Debug)]
struct ParseError {
    filename: PathBuf,
    inner: EspritError,
}

impl ParseError {
    fn new(filename: &PathBuf, inner: EspritError) -> ParseError {
        ParseError { filename: filename.clone(), inner }
    }

    fn into_inner(self) -> EspritError {
        self.inner
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let position = match self.inner {
            EspritError::UnexpectedToken(ref token) | EspritError::FailedASI(ref token) |
            EspritError::IllegalBreak(ref token) | EspritError::IllegalContinue(ref token) |
            EspritError::DuplicateDefault(ref token) | EspritError::StrictWith(ref token) |
            EspritError::ThrowArgument(ref token) | EspritError::OrphanTry(ref token) =>
                Some(token.location),
            EspritError::TopLevelReturn(ref span) | EspritError::ForOfLetExpr(ref span) |
            EspritError::ContextualKeyword(ref span, _) | EspritError::IllegalStrictBinding(ref span, _) =>
                Some(*span),
            EspritError::InvalidLabel(ref id) | EspritError::InvalidLabelType(ref id) =>
                id.location,
            EspritError::LexError(_) => None,
            EspritError::InvalidLHS(span, _) => span,
            EspritError::UnsupportedFeature(_) => None,
            EspritError::UnexpectedDirective(span, _) => span,
            EspritError::UnexpectedModule(span) => span,
            EspritError::ImportInScript(ref _import) => None, // For now
            EspritError::ExportInScript(ref _export) => None, // For now
            EspritError::CompoundParamWithUseStrict(ref _patt) => None, // For now
        };
        write!(f, "Parse error in {}:{}\n{}", path_to_string(&self.filename), match position {
            Some(span) => format!("{}:{}", span.start.line, span.start.column),
            None => "0:0".into(),
        }, self.description())
    }
}

impl StdError for ParseError {
    fn description(&self) -> &str {
        self.inner.description()
    }
    fn cause(&self) -> Option<&StdError> {
        Some(&self.inner)
    }
}

/// Builds a dependency tree for Node modules.
pub struct Deps {
    module_id: u32,
    resolver: Resolver,
    loaded_files: HashSet<PathBuf>,
    module_map: ModuleMap,
    include_builtins: bool,
    builtins: Box<Builtins>,
}

impl Deps {
    /// Create a new dependency tree.
    pub fn new() -> Deps {
        let resolver = Resolver::new()
            .with_extensions(vec![".js", ".json"]);
        let module_map = ModuleMap::new();
        let module_id = 0;
        let loaded_files = HashSet::new();
        let builtins = NoBuiltins;

        Deps {
            resolver,
            module_map,
            module_id,
            loaded_files,
            include_builtins: true,
            builtins: Box::new(builtins),
        }
    }

    /// Use a different resolver.
    ///
    /// # Examples
    ///
    /// ```
    /// use node_resolve::Resolver;
    /// use deps::Deps;
    ///
    /// let deps = Deps::new()
    ///     .with_resolver(Resolver::new().preserve_symlinks(false));
    /// ```
    pub fn with_resolver(mut self, resolver: Resolver) -> Self {
        self.resolver = resolver;
        self
    }

    /// Configure the base path for Node builtin shims resolution.
    ///
    /// # Examples
    ///
    /// ```
    /// use deps::Deps;
    /// // Use builtin shims provided by the node-libs-browser package.
    /// let deps = Deps::new()
    ///     .with_builtins_path("./node_modules/node-libs-browser".into())
    /// ```
    pub fn with_builtins_path(mut self, path: PathBuf) -> Self {
        self.builtins = Box::new(NodeBuiltins::new(path));
        self
    }

    /// Disable bundling builtin modules.
    pub fn no_builtins(mut self) -> Self {
        self.builtins = Box::new(NoBuiltins);
        self
    }

    /// Toggle inclusion of builtins.
    /// If `false`, builtin modules will stay as external `require()` calls.
    /// Then whatever program runs the bundle (eg. node) will provide these
    /// modules.
    /// If `true`, shims for builtin modules will be included in the bundle.
    pub fn include_builtins(mut self, include: bool) -> Self {
        self.include_builtins = include;
        self
    }

    /// Start dependency resolution at an entry file.
    pub fn run(&mut self, entry: &str) -> Result<()> {
        let resolved = self.resolver.with_basedir(PathBuf::from("."))
            .resolve(entry)?;

        let mut record = self.read_file(resolved, true)?;
        let rec_path = path_to_string(&record.path);
        self.loaded_files.insert(record.path.to_path_buf());
        self.read_deps(&mut record)?;
        self.add_module(&rec_path, record);
        Ok(())
    }

    fn read_file(&mut self, path: PathBuf, is_entry: bool) -> Result<ModuleRecord> {
        self.module_id += 1;
        let file = File::open(&path)?;
        let mut reader = BufReader::new(file);
        let mut source = String::new();
        reader.read_to_string(&mut source)?;

        let is_json = path.extension().map_or(false, |ext| ext == "json");
        let dependencies = if is_json {
            vec![]
        } else {
            let ast = script(&source).map_err(|err| ParseError::new(&path, err))?;
            detect(&ast)
        };

        if is_json {
            let _value: serde_json::Value = serde_json::from_str(&source)?; // Check syntax
            source = format!("module.exports = {}", source);
        }

        let hash = Sha1::digest_str(&source);

        let box_path = path.into_boxed_path();
        let basedir = box_path.parent().unwrap().to_path_buf();
        Ok(ModuleRecord {
            path: box_path,
            source,
            id: self.module_id,
            hash: hash as Hash,
            entry: is_entry,
            dependencies: self.resolve_deps(basedir, dependencies)?,
        })
    }

    fn resolve_deps(&mut self, basedir: PathBuf, dependencies: Vec<String>) -> Result<Dependencies> {
        let resolver = self.resolver.with_basedir(basedir);
        let mut map = Dependencies::new();
        for dep_id in dependencies {
            // TODO include core module shims
            let path = if self.builtins.is_builtin(&dep_id) {
                if self.include_builtins {
                    self.builtins.resolve(&resolver, &dep_id)?
                } else {
                    None
                }
            } else {
                Some(resolver.resolve(&dep_id)?)
            };
            path.map(|resolved| map.insert(dep_id.clone(), Dependency::resolved(dep_id, resolved)));
        }
        Ok(map)
    }

    fn read_deps(&mut self, record: &mut ModuleRecord) -> Result<()> {
        for dependency in record.dependencies.values_mut() {
            let dep_record = if let Some(ref resolved) = dependency.resolved {
                if !self.loaded_files.contains(resolved) {
                    let mut new_record = self.read_file(resolved.clone(), false)?;
                    let new_path = path_to_string(&new_record.path);
                    self.loaded_files.insert(new_record.path.to_path_buf());
                    self.read_deps(&mut new_record)?;
                    self.add_module(&new_path, new_record);
                }
                self.module_map.get(&path_to_string(resolved)).map(|rc| rc.to_owned())
            } else {
                None
            };

            dep_record.map(|d| dependency.set_record(&d));
        }
        Ok(())
    }

    fn add_module(&mut self, rec_path: &str, record: ModuleRecord) -> () {
        self.module_map.insert(rec_path.to_string(), Rc::new(record));
    }
}

impl Deref for Deps {
    type Target = ModuleMap;
    fn deref(&self) -> &Self::Target {
        &self.module_map
    }
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}
