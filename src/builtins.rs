use std::collections::HashMap;
use std::path::PathBuf;
use node_resolve::{Resolver, is_core_module};
use quicli::prelude::Result;
use node_core_shims::{NodeBuiltin, get_builtin_mapping};

/// Map builtin module names to a resolvable module ID.
pub trait Builtins {
    fn is_builtin(&self, module_id: &str) -> bool;
    fn resolve(&self, resolver: &Resolver, module_id: &str) -> Result<Option<PathBuf>>;
}

/// Support Node builtins.
pub struct NodeBuiltins {
    basedir: PathBuf,
    mapping: HashMap<String, NodeBuiltin>,
}

impl NodeBuiltins {
    pub fn new(basedir: PathBuf) -> Self {
        NodeBuiltins {
            basedir,
            mapping: get_builtin_mapping(),
        }
    }
}

impl Builtins for NodeBuiltins {
    fn is_builtin(&self, module_id: &str) -> bool {
        is_core_module(module_id)
    }

    fn resolve(&self, resolver: &Resolver, module_id: &str) -> Result<Option<PathBuf>> {
        let builtin: &NodeBuiltin = self.mapping.get(module_id)
            .unwrap_or_else(|| panic!("Missing builtin mapping for {}", module_id));

        match *builtin {
            NodeBuiltin::Package(ref package_id) => {
                resolver
                    .with_basedir(self.basedir.clone())
                    .resolve(package_id)
                    .map(|r| Some(r))
                    .map_err(|e| e.into())
            },
            NodeBuiltin::Stub => Ok(None),
        }
    }
}

/// Do not support any builtins.
pub struct NoBuiltins;
impl Builtins for NoBuiltins {
    fn is_builtin(&self, _module_id: &str) -> bool { false }
    fn resolve(&self, _resolver: &Resolver, _module_id: &str) -> Result<Option<PathBuf>> { Ok(None) }
}
