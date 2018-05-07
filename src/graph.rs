use std::collections::HashMap;
use std::rc::Rc;
use std::path::{Path, PathBuf};

/// Map dependency IDs used inside require() to their full paths.
pub type Dependencies = HashMap<String, PathBuf>;

/// A Module.
#[derive(Debug)]
pub struct ModuleRecord {
    pub path: Box<Path>,
    pub source: String,
    pub entry: bool,
    pub dependencies: Dependencies,
}

/// Keeps track of modules.
pub type ModuleMap = HashMap<String, Rc<ModuleRecord>>;
