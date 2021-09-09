use bf19::exec;
use std::io;

fn main() {
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    exec(&input).unwrap()
}
