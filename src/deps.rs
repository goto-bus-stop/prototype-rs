use std::collections::HashSet;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use quicli::prelude::*; // TODO use `failure`?
use node_resolve::Resolver;
use builtins::{Builtins, NodeBuiltins, NoBuiltins};
use graph::{ModuleMap, Dependency, Dependencies, SourceFile, ModuleRecord};
use loader::LoadFile;

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

        let source_file = LoadFile::new(resolved).run()?;
        let mut record = self.to_record(source_file, true)?;
        let rec_path = path_to_string(&record.file.path());
        self.loaded_files.insert(record.file.path().clone());
        self.read_deps(&mut record)?;
        self.add_module(&rec_path, record);
        Ok(())
    }

    fn to_record(&mut self, file: SourceFile, entry: bool) -> Result<ModuleRecord> {
        self.module_id += 1;
        let basedir = file.path().clone().parent().unwrap().to_path_buf();
        let dependencies = match file {
            SourceFile::CJS { ref dependencies, .. } => self.resolve_deps(basedir, dependencies)?,
            _ => Dependencies::new(),
        };
        Ok(ModuleRecord {
            id: self.module_id,
            file,
            entry,
            dependencies,
        })
    }

    fn resolve_deps(&mut self, basedir: PathBuf, dependencies: &Vec<String>) -> Result<Dependencies> {
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
            path.map(|resolved| map.insert(dep_id.clone(), Dependency::resolved(dep_id.clone(), resolved)));
        }
        Ok(map)
    }

    fn read_deps(&mut self, record: &mut ModuleRecord) -> Result<()> {
        for dependency in record.dependencies.values_mut() {
            let dep_record = if let Some(ref resolved) = dependency.resolved {
                if !self.loaded_files.contains(resolved) {
                    let source_file = LoadFile::new(resolved.clone()).run()?;
                    let mut new_record = self.to_record(source_file, true)?;
                    let new_path = path_to_string(&new_record.file.path());
                    self.loaded_files.insert(new_record.file.path().to_path_buf());
                    self.read_deps(&mut new_record)?;
                    self.add_module(&new_path, new_record);
                }
                self.module_map.get(&path_to_string(resolved)).map(|rc| rc.to_owned())
            } else {
                None
            };

            if dep_record.is_none() {
                warn!("Could not resolve ModuleRecord for {} from {}", dependency.name, record.file.path().to_string_lossy());
            }
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
