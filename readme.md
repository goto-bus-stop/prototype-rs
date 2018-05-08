# prototype

A basic JS bundler in Rust, using [esprit](https://github.com/dherman/esprit) and [node-resolve](https://github.com/goto-bus-stop/node-resolve).

It's a bit similar to browserify but without all the features. It doesn't include shims for Node builtins or whatever.

Output is probably different every time because it's just outputting it in the order of the HashMap used internally. Don't worry about it!

## Install

```bash
git clone https://github.com/goto-bus-stop/prototype-rs.git
cd prototype-rs
cargo run ~/path/to/entry/point.js > output.js
```

## TODO

 - [ ] bundle node builtin modules
 - [ ] insert-module-globals
 - [ ] transform
 - [ ] async
