extern crate easter;
extern crate esprit;
extern crate node_resolve;
extern crate serde_json;
#[macro_use] extern crate quicli;

mod detect;

use std::fs::File;
use std::io::{Read, BufReader};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;
use esprit::script;
use node_resolve::Resolver;
use quicli::prelude::*;
use detect::detect;

#[derive(Debug, StructOpt)]
struct Options {
    entry: String,
}

type Dependencies = HashMap<String, PathBuf>;

#[derive(Debug)]
struct ModuleRecord {
    path: Box<Path>,
    source: String,
    dependencies: Dependencies,
}

type ModuleMap = HashMap<String, Rc<ModuleRecord>>;

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

struct Deps {
    resolver: Resolver,
    module_map: ModuleMap,
}

impl Deps {
    fn new() -> Deps {
        let resolver = Resolver::new()
            .with_extensions(vec![".js", ".json"]);
        let module_map = ModuleMap::new();

        Deps { resolver, module_map }
    }

    fn run(&mut self, entry: &str) -> Result<()> {
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
        let dependencies = detect(ast);
        let box_path = path.into_boxed_path();
        let basedir = box_path.parent().unwrap().to_path_buf();
        Ok(ModuleRecord {
            path: box_path,
            source,
            dependencies: self.resolve_deps(basedir, dependencies)?,
        })
    }

    fn resolve_deps(&mut self, basedir: PathBuf, dependencies: Vec<String>) -> Result<HashMap<String, PathBuf>> {
        let resolver = self.resolver.with_basedir(basedir);
        let mut map = HashMap::new();
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

struct Pack<'a> {
    modules: &'a ModuleMap,
}

impl<'a> Pack<'a> {
    fn new(modules: &ModuleMap) -> Pack {
        Pack { modules }
    }

    fn to_string(&self) -> String {
        let mut string = String::from("_require = (function () {
			function outer(modules, cache, entry) {
				var previousRequire = typeof require == 'function' && require;

				function newRequire(name, jumped){
					if(!cache[name]) {
						if(!modules[name]) {
							var currentRequire = typeof require == 'function' && require;
							if (!jumped && currentRequire) return currentRequire(name, true);

							if (previousRequire) return previousRequire(name, true);
							var err = new Error('Cannot find module \\'' + name + '\\'');
							err.code = 'MODULE_NOT_FOUND';
							throw err;
						}
						var m = cache[name] = {exports:{}};
						modules[name][0].call(m.exports, function(x){
							var id = modules[name][1][x];
							return newRequire(id ? id : x);
						},m,m.exports,outer,modules,cache,entry);
					}
					return cache[name].exports;
				}
				for(var i=0;i<entry.length;i++) newRequire(entry[i]);

				return newRequire;
			}

			return outer;
        })()({\n");

        let mut first = true;
        for record in self.modules.values() {
            if !first { string.push_str(",\n"); }
            string.push_str("\"");
            string.push_str(&path_to_string(&record.path));
            string.push_str("\":[function(require,exports,module){\n");
            string.push_str(&record.source);
            string.push_str("\n},");
            string.push_str(&serde_json::to_string(&record.dependencies).unwrap());
            string.push_str("]");
            first = false;
        }

        string.push_str("},{},[]);");
        string
    }
}

main!(|args: Options| {
    let mut deps = Deps::new();
    deps.run(&args.entry)?;
    println!("{}", Pack::new(&deps).to_string());
});
