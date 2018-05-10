extern crate digest;
extern crate easter;
extern crate esprit;
extern crate node_resolve;
extern crate serde_json;
extern crate sha1;
extern crate estree_detect_requires;
extern crate node_core_shims;
extern crate time;
#[macro_use] extern crate quicli;

mod builtins;
mod graph;
mod deps;
mod pack;

use std::io::{Write, stdout};
use time::PreciseTime;
use quicli::prelude::*;
use deps::Deps;
use pack::Pack;

#[derive(Debug, StructOpt)]
struct Options {
    entry: String,
    #[structopt(long = "no-builtins", help = "Exclude shims for builtin modules. Useful when generating a bundle for Node.")]
    no_builtins: bool,
}

main!(|args: Options| {
    let start = PreciseTime::now();
    let mut deps = Deps::new()
        .include_builtins(!args.no_builtins)
        .with_builtins_path("./crates/node-core-shims".into());

    deps.run(&args.entry)?;
    let mut out = stdout();
    let bundle = Pack::new(&deps).to_string();
    let size = bundle.len();
    out.write_all(bundle.as_bytes())?;
    let end = PreciseTime::now();
    eprint!("wrote {} bytes, took {}ms\n", size, start.to(end).num_milliseconds());
});
