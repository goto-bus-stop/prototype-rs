use std::collections::BTreeMap;
use std::rc::Rc;
use serde_json;
use graph::{ModuleMap, ModuleRecord};

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
        let mut modules: Vec<&Rc<ModuleRecord>> = self.modules.values().collect();
        modules.sort_unstable_by(|a, b| a.hash_cmp(b));
        for record in modules {
            if !first { string.push_str(",\n"); }
            string.push_str(&format!(
                "{id}:[function(require,exports,module){{\n{source}\n}},{deps}]",
                id = serde_json::to_string(&record.id).unwrap(),
                source = record.file.source(),
                deps = serde_json::to_string(
                    &record.dependencies.iter()
                        .map(|(key, val)| (key, match val.record {
                             Some(ref rec) => Some(rec.id),
                             None => None,
                         }))
                        .collect::<BTreeMap<&String, Option<u32>>>()
                ).unwrap(),
            ));
            first = false;

            if record.entry {
                entries.push(record.id);
            }
        }

        string.push_str("},{},");
        string.push_str(&serde_json::to_string(&entries).unwrap());
        string.push_str(");");
        string
    }
}
