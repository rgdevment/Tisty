use super::*;

struct Sensing;

fn where_asked() -> Result<(), Hitch> {
    match on_a_runtime() {
        true => Err(Hitch::Broke("asked on the runtime".into())),
        false => Ok(()),
    }
}

impl Remote for Sensing {
    fn limits(&self) -> Limits {
        unreachable!()
    }
    fn list(&self, _: &str) -> Result<Vec<Seen>, Hitch> {
        where_asked().map(|()| Vec::new())
    }
    fn fetch(&self, name: &str, _: u64, into: &mut dyn Write) -> Result<Seen, Hitch> {
        where_asked()?;
        for part in [b"uno ".as_slice(), b"dos ", b"tres"] {
            into.write_all(part)
                .map_err(|e| Hitch::Broke(e.to_string()))?;
        }
        Ok(Seen {
            name: name.to_string(),
            bytes: 12,
            hash: String::new(),
            revision: "r1".into(),
        })
    }
    fn put(&self, _: &str, _: &Path, _: Expect) -> Result<Seen, Hitch> {
        unreachable!()
    }
    fn delete(&self, _: &str, _: Option<&str>) -> Result<(), Hitch> {
        where_asked()
    }
    fn hash_of(&self, _: &Path) -> std::io::Result<String> {
        unreachable!()
    }
}

fn on_the_runtime<T>(work: impl FnOnce() -> T) -> T {
    tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(async { work() })
}

#[test]
fn a_request_asked_on_the_runtime_runs_off_it() {
    let off = OffRuntime(Sensing);

    assert_eq!(on_the_runtime(|| off.list("")), Ok(Vec::new()));
    assert_eq!(on_the_runtime(|| off.delete("a", None)), Ok(()));
    assert_eq!(off.list(""), Ok(Vec::new()));
}

#[test]
fn a_download_asked_on_the_runtime_streams_every_byte_to_its_writer() {
    let off = OffRuntime(Sensing);
    let mut into = Vec::new();

    let seen = on_the_runtime(|| off.fetch("a", 0, &mut into)).unwrap();

    assert_eq!(seen.revision, "r1");
    assert_eq!(into, b"uno dos tres");
}

struct Refusing;

impl Write for Refusing {
    fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
        Err(std::io::Error::other("disco lleno"))
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[test]
fn a_writer_that_refuses_stops_the_download_and_says_why() {
    let off = OffRuntime(Sensing);

    let refused = on_the_runtime(|| off.fetch("a", 0, &mut Refusing));

    assert!(matches!(refused, Err(Hitch::Broke(why)) if why.contains("disco lleno")));
}
