use std::fs::File;
use std::io::{Read, BufReader};
use std::path::{Path, PathBuf};
use std::ops::Deref;
use std::rc::Rc;
use esprit::script;
use node_resolve::Resolver;
use estree_detect_requires::detect;
use graph::{ModuleMap, Dependencies, ModuleRecord};
use quicli::prelude::Result; // TODO use `failure`?

pub struct Deps {
    resolver: Resolver,
    module_map: ModuleMap,
}

impl Deps {
    pub fn new() -> Deps {
        let resolver = Resolver::new()
            .with_extensions(vec![".js", ".json"]);
        let module_map = ModuleMap::new();

        Deps { resolver, module_map }
    }

    pub fn run(&mut self, entry: &str) -> Result<()> {
        let resolved = self.resolver.with_basedir(PathBuf::from("."))
            .resolve(&entry)?;

        let record = self.read_file(resolved)?;
        let rec_path = path_to_string(&record.path);
        self.add_module(&rec_path, record);

        self.read_deps(&rec_path)?;
        Ok(())
    }

    fn read_file(&mut self, path: PathBuf) -> Result<ModuleRecord> {
        let file = File::open(&path)?;
        let mut reader = BufReader::new(file);
        let mut source = String::new();
        reader.read_to_string(&mut source)?;

        let ast = script(&source).unwrap();
        let dependencies = detect(&ast);
        let box_path = path.into_boxed_path();
        let basedir = box_path.parent().unwrap().to_path_buf();
        Ok(ModuleRecord {
            path: box_path,
            source,
            dependencies: self.resolve_deps(basedir, dependencies)?,
        })
    }

    fn resolve_deps(&mut self, basedir: PathBuf, dependencies: Vec<String>) -> Result<Dependencies> {
        let resolver = self.resolver.with_basedir(basedir);
        let mut map = Dependencies::new();
        for dep_id in dependencies {
            let path = resolver.resolve(&dep_id)?;
            map.insert(dep_id, path);
        }
        Ok(map)
    }

    fn read_deps(&mut self, rec_path: &String) -> Result<()> {
        let record = { self.module_map.get(rec_path).unwrap().to_owned() };
        for path in record.dependencies.values() {
            if !self.module_map.contains_key(&path_to_string(&path)) {
                let new_record = self.read_file(path.clone())?;
                let new_path = path_to_string(&new_record.path);
                self.add_module(&new_path, new_record);
                self.read_deps(&new_path)?;
            }
        }
        Ok(())
    }

    fn add_module(&mut self, rec_path: &String, record: ModuleRecord) -> () {
        self.module_map.insert(rec_path.clone(), Rc::new(record));
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
