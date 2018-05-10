use std::collections::HashMap;

#[derive(Clone)]
pub enum NodeBuiltin {
    Stub,
    Package(String),
}

pub fn get_builtin_mapping() -> HashMap<String, NodeBuiltin> {
    [
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
    ].iter().cloned().collect()
}
