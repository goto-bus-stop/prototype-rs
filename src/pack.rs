use std::path::Path;
use serde_json;
use graph::ModuleMap;

/// Pack a ModuleMap into a browserify-style javascript bundle.
pub struct Pack<'a> {
    modules: &'a ModuleMap,
}

impl<'a> Pack<'a> {
    pub fn new(modules: &ModuleMap) -> Pack {
        Pack { modules }
    }

    pub fn to_string(&self) -> String {
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
        let mut entries = vec![];
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

            if record.entry {
                entries.push(path_to_string(&record.path));
            }
        }

        string.push_str("},{},");
        string.push_str(&serde_json::to_string(&entries).unwrap());
        string.push_str(");");
        string
    }
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}
