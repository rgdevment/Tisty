// Releases from 1.31 on carry no command line, so versions.yml builds this in their own tree to meet them.
#[path = "../tests/person/mod.rs"]
mod person;

use std::path::Path;

fn main() {
    let data = std::env::var_os("TISTY_DATA").expect("TISTY_DATA names where the history is kept");
    let home = Path::new(&data)
        .parent()
        .expect("TISTY_DATA sits inside a home");
    let person = person::Person::at(home);
    let asked: Vec<String> = std::env::args().skip(1).collect();
    let asked: Vec<&str> = asked.iter().map(String::as_str).collect();
    match asked.as_slice() {
        ["config", "set", "remote", at] => person.shares_through(Path::new(at)),
        ["sync", ..] => {
            if let Err(trouble) = person.syncs() {
                eprintln!("nothing was synced: {trouble}");
                std::process::exit(1);
            }
        }
        ["ls", "all"] => println!("{}", person.open()),
        [said] => person.begins(said),
        _ => {
            eprintln!(
                "this release is met only through config set remote, sync, ls all and a capture"
            );
            std::process::exit(2);
        }
    }
}
