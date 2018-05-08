use std::cmp::Ordering;
use std::collections::{HashMap, BTreeMap};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use digest::generic_array::GenericArray;
use digest::generic_array::typenum::U20;

/// Map dependency IDs used inside require() to their full paths.
pub type Dependencies = BTreeMap<String, PathBuf>;
pub type Hash = GenericArray<u8, U20>;

/// A Module.
#[derive(Debug)]
pub struct ModuleRecord {
    pub path: Box<Path>,
    pub source: String,
    pub hash: Hash,
    pub entry: bool,
    pub dependencies: Dependencies,
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
