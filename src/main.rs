extern crate easter;
extern crate esprit;
extern crate node_resolve;
extern crate serde_json;
extern crate estree_detect_requires;
extern crate time;
#[macro_use] extern crate quicli;

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
}

main!(|args: Options| {
    let start = PreciseTime::now();
    let mut deps = Deps::new();
    deps.run(&args.entry)?;
    let mut out = stdout();
    let bundle = Pack::new(&deps).to_string();
    let size = bundle.len();
    out.write_all(bundle.as_bytes())?;
    let end = PreciseTime::now();
    eprint!("wrote {} bytes, took {}ms\n", size, start.to(end).num_milliseconds());
});
