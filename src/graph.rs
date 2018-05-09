use std::cmp::Ordering;
use std::collections::{HashMap, BTreeMap};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use digest::generic_array::GenericArray;
use digest::generic_array::typenum::U20;

/// Map dependency IDs used inside require() to their full paths.
pub type Dependencies = BTreeMap<String, Dependency>;
pub type Hash = GenericArray<u8, U20>;

/// A Module.
#[derive(Debug)]
pub struct ModuleRecord {
    pub path: Box<Path>,
    pub id: u32,
    pub source: String,
    pub hash: Hash,
    pub entry: bool,
    pub dependencies: Dependencies,
}

#[derive(Debug)]
pub struct Dependency {
    pub name: String,
    pub resolved: Option<PathBuf>,
    pub record: Option<Rc<ModuleRecord>>,
}

impl Dependency {
    pub fn uninitialized(name: String) -> Self {
        Dependency {
            name,
            resolved: None,
            record: None,
        }
    }

    pub fn resolved(name: String, resolved: PathBuf) -> Self {
        Dependency {
            name,
            resolved: Some(resolved),
            record: None,
        }
    }

    pub fn with_record(mut self, record: &Rc<ModuleRecord>) -> Self {
        self.record = Some(Rc::clone(record));
        self
    }

    pub fn set_record(&mut self, record: &Rc<ModuleRecord>) -> () {
        self.record = Some(Rc::clone(record));
    }
}

impl ModuleRecord {
    pub fn hash_cmp(&self, other: &Self) -> Ordering {
        for i in 0..self.hash.len() {
            let order = self.hash[i].cmp(&other.hash[i]);
            if order != Ordering::Equal {
                return order
            }
        }
        Ordering::Equal
    }
}

/// Keeps track of modules.
pub type ModuleMap = HashMap<String, Rc<ModuleRecord>>;
