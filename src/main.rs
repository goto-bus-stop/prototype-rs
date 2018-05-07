extern crate easter;
extern crate esprit;
extern crate node_resolve;
extern crate serde_json;
extern crate estree_detect_requires;
#[macro_use] extern crate quicli;

mod graph;
mod deps;
mod pack;

use quicli::prelude::*;
use deps::Deps;
use pack::Pack;

#[derive(Debug, StructOpt)]
struct Options {
    entry: String,
}

main!(|args: Options| {
    let mut deps = Deps::new();
    deps.run(&args.entry)?;
    println!("{}", Pack::new(&deps).to_string());
});
