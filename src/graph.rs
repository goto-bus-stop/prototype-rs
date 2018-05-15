use std::cmp::Ordering;
use std::collections::{HashMap, BTreeMap};
use std::path::PathBuf;
use std::rc::Rc;
use digest::generic_array::GenericArray;
use digest::generic_array::typenum::U20;
use easter::stmt::Script;
use serde_json::Value;

/// Map dependency IDs used inside require() to their full paths.
pub type Dependencies = BTreeMap<String, Dependency>;
pub type Hash = GenericArray<u8, U20>;

/// A source file.
#[derive(Debug)]
pub enum SourceFile {
    CJS {
        /// Path to the file.
        path: PathBuf,
        /// The file source content.
        source: String,
        /// Hash of the source content.
        hash: Hash,
        /// Syntax tree.
        ast: Option<Script>,
        /// Dependencies.
        dependencies: Vec<String>,
    },
    /// A JSON source file on disk.
    JSON {
        /// Path to the file.
        path: PathBuf,
        /// The file source content.
        source: String,
        /// Hash of the source content.
        hash: Hash,
        /// The JSON object.
        value: Value,
    },
}

// TODO There's probably a way to do this with a macro
impl SourceFile {
    pub fn path(&self) -> &PathBuf {
        match *self {
            SourceFile::CJS { ref path, .. } => path,
            SourceFile::JSON { ref path, .. } => path,
        }
    }

    pub fn source(&self) -> &String {
        match *self {
            SourceFile::CJS { ref source, .. } => source,
            SourceFile::JSON { ref source, .. } => source,
        }
    }

    pub fn hash(&self) -> &Hash {
        match *self {
            SourceFile::CJS { ref hash, .. } => hash,
            SourceFile::JSON { ref hash, .. } => hash,
        }
    }
}

/// A Module.
#[derive(Debug)]
pub struct ModuleRecord {
    pub file: SourceFile,
    /// A unique ID for this module.
    pub id: u32,
    /// Whether this module is an entry point to the graph.
    pub entry: bool,
    /// Map of dependency names to ModuleRecords.
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
        self.set_record(record);
        self
    }

    pub fn set_record(&mut self, record: &Rc<ModuleRecord>) -> () {
        self.record = Some(Rc::clone(record));
    }
}

impl ModuleRecord {
    pub fn hash_cmp(&self, other: &Self) -> Ordering {
        for i in 0..self.file.hash().len() {
            let order = self.file.hash()[i].cmp(&other.file.hash()[i]);
            if order != Ordering::Equal {
                return order
            }
        }
        Ordering::Equal
    }
}

/// Keeps track of modules.
pub type ModuleMap = HashMap<String, Rc<ModuleRecord>>;
