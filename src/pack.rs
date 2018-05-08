use std::path::Path;
use serde_json;
use graph::ModuleMap;

/// Pack a `ModuleMap` into a browserify-style javascript bundle.
pub struct Pack<'a> {
    modules: &'a ModuleMap,
}

impl<'a> Pack<'a> {
    pub fn new(modules: &ModuleMap) -> Pack {
        Pack { modules }
    }

    pub fn to_string(&self) -> String {
        let mut string = String::from("_require = ");
        string.push_str(include_str!("./runtime.js"));
        string.push_str("({\n");

        let mut first = true;
        let mut entries = vec![];
        for record in self.modules.values() {
            if !first { string.push_str(",\n"); }
            string.push_str(&format!(
                "{id}:[function(require,exports,module){{\n{source}\n}},{deps}]",
                id = serde_json::to_string(&path_to_string(&record.path)).unwrap(),
                source = record.source,
                deps = serde_json::to_string(&record.dependencies).unwrap(),
            ));
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
