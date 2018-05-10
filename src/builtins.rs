use std::collections::HashMap;
use std::path::PathBuf;
use node_resolve::{Resolver, is_core_module};
use quicli::prelude::Result;

/// Map builtin module names to a resolvable module ID.
pub trait Builtins {
    fn is_builtin(&self, module_id: &str) -> bool;
    fn resolve(&self, resolver: &Resolver, module_id: &str) -> Result<Option<PathBuf>>;
}

#[derive(Clone)]
pub enum NodeBuiltin {
    Stub,
    Package(String),
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
            mapping: [
                ("assert".to_string(), NodeBuiltin::Package("assert/".to_string())),
                ("buffer".to_string(), NodeBuiltin::Package("buffer/".to_string())),
                ("crypto".to_string(), NodeBuiltin::Package("crypto-browserify".to_string())),
                ("events".to_string(), NodeBuiltin::Package("events/".to_string())),
                ("fs".to_string(), NodeBuiltin::Stub),
                ("http".to_string(), NodeBuiltin::Package("stream-http".to_string())),
                ("https".to_string(), NodeBuiltin::Package("https-browserify".to_string())),
                ("os".to_string(), NodeBuiltin::Package("os-browserify".to_string())),
                ("path".to_string(), NodeBuiltin::Package("path-browserify".to_string())),
                ("process".to_string(), NodeBuiltin::Package("process/".to_string())),
                ("querystring".to_string(), NodeBuiltin::Package("querystring-es3".to_string())),
                ("stream".to_string(), NodeBuiltin::Package("stream-browserify".to_string())),
                ("string_decoder".to_string(), NodeBuiltin::Package("string_decoder".to_string())),
                ("timers".to_string(), NodeBuiltin::Package("timers-browserify".to_string())),
                ("tty".to_string(), NodeBuiltin::Package("tty-browserify".to_string())),
                ("url".to_string(), NodeBuiltin::Package("url/".to_string())),
                ("util".to_string(), NodeBuiltin::Package("util/".to_string())),
                ("vm".to_string(), NodeBuiltin::Package("vm-browserify".to_string())),
            ].iter().cloned().collect(),
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
