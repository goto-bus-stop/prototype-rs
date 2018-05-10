use std::process::Command;

fn main() {
    Command::new("npm").args(&["install"]).status().unwrap();
}
