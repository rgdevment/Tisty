// Releases from 1.31 on carry no command line, so versions.yml builds this in their own tree to meet them.
#[path = "../tests/person/mod.rs"]
mod person;

use std::path::Path;

// What the command lines said in words, a core says as the trouble that stopped it.
const ASKS_TO_UPDATE: [&str; 4] = ["Newer(", "Needs(", "Shape(", "UnsupportedVersion"];

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
        ["sync", "--confirm", whose, ..] => {
            if !confirms(home, &person, whose) {
                eprintln!("{whose} could not be confirmed");
                std::process::exit(1);
            }
        }
        ["sync", ..] => {
            if let Err(trouble) = person.syncs() {
                if ASKS_TO_UPDATE.iter().any(|said| trouble.contains(said)) {
                    eprintln!("a newer Tisty wrote here, so nothing was synced: {trouble}");
                } else {
                    eprintln!("nothing was synced: {trouble}");
                }
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

fn confirms(home: &Path, person: &person::Person, whose: &str) -> bool {
    let paths = tisty_core::Paths::new(home.join("data"), home.join("config"));
    let config = tisty_core::Config::load_or_init(&paths).expect("a config");
    let Some(shared) = config.sync.as_ref().and_then(|way| way.place()) else {
        return false;
    };
    let who = tisty_core::DeviceId(whose.to_string());
    let says = person.state().keys.get(&who).cloned().or_else(|| {
        tisty_core::store::key_said_in(&shared.join(tisty_carrier::STORE).join(whose), &who)
    });
    let Some(says) = says else {
        return false;
    };
    if !tisty_core::vouched::confirm(paths.data(), &who, &says) {
        return false;
    }
    tisty_core::turned::let_through(paths.data(), whose);
    true
}
