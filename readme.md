# prototype

A basic JS bundler in Rust, using [esprit](https://github.com/dherman/esprit) and [node-resolve](https://github.com/goto-bus-stop/node-resolve).

It's a bit similar to browserify but without all the features.

This is a learning project for me so there will be breaking changes with every commit probably as I figure things out!

## Install

```bash
git clone https://github.com/goto-bus-stop/prototype-rs.git
# IMPORTANT: Must be cloned adjacent to prototype-rs!
# Because our Cargo.toml specifies a dependency on "../esprit".
git clone https://github.com/dherman/esprit.git
cd prototype-rs
cargo run ~/path/to/entry/point.js > output.js
```

## TODO

 - [ ] insert-module-globals
 - [ ] transform
 - [ ] async
