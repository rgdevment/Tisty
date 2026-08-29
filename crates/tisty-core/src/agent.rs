use crate::{
    Config, Result,
    event::{DeviceId, DeviceKind, Op},
    paths::Paths,
    store::Store,
};

/// Minting is the person's act: nothing reachable over a wire calls this.
pub fn register(paths: &Paths) -> Result<DeviceId> {
    let mut config = Config::load_or_init(paths)?;
    if let Some(held) = config.agent_id.clone() {
        return Ok(held);
    }

    let who = DeviceId(crate::config::new_device_id());
    config.agent_id = Some(who.clone());
    config.save(paths)?;

    let mut store = Store::open(paths.store(), who.clone())?;
    store.append(Op::DeviceJoin {
        d: who.clone(),
        k: Some(DeviceKind::Agent),
    })?;
    Ok(who)
}

/// What it wrote stays: retiring takes the voice, never the words.
pub fn retire(paths: &Paths) -> Result<Option<DeviceId>> {
    let mut config = Config::load_or_init(paths)?;
    let Some(who) = config.agent_id.clone() else {
        return Ok(None);
    };

    config.agent_id = None;
    config.save(paths)?;

    let mut store = Store::open(paths.store(), config.device_id.clone())?;
    store.append(Op::DeviceRemove { d: who.clone() })?;
    Ok(Some(who))
}

pub fn registered(paths: &Paths) -> Result<Option<DeviceId>> {
    Ok(Config::load_or_init(paths)?.agent_id)
}

/// Where an agent may take a file from: what it attaches reaches the shared folder.
pub fn may_attach(source: &std::path::Path, paths: &Paths) -> Result<std::path::PathBuf> {
    let refused = || crate::Error::OutsideTheStore(source.display().to_string());
    let at = source.canonicalize().map_err(|_| refused())?;

    let mine = [paths.data(), paths.config(), paths.cache()];
    if mine
        .iter()
        .filter_map(|one| one.canonicalize().ok())
        .any(|one| at.starts_with(one))
    {
        return Err(refused());
    }

    reachable()
        .iter()
        .any(|root| at.starts_with(root))
        .then_some(at)
        .ok_or_else(refused)
}

pub fn reachable() -> Vec<std::path::PathBuf> {
    let mut roots = vec![std::env::temp_dir()];
    if let Some(dirs) = directories::UserDirs::new() {
        for one in [
            dirs.download_dir(),
            dirs.document_dir(),
            dirs.picture_dir(),
            dirs.desktop_dir(),
        ] {
            roots.extend(one.map(std::path::Path::to_path_buf));
        }
    }
    roots
        .iter()
        .filter_map(|one| one.canonicalize().ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paths(at: &std::path::Path) -> Paths {
        Paths::new(at.join("data"), at.join("config"))
    }

    #[test]
    fn registering_twice_does_not_mint_a_second_identity() {
        let tmp = tempfile::tempdir().unwrap();
        let paths = paths(tmp.path());

        let first = register(&paths).unwrap();
        let again = register(&paths).unwrap();

        assert_eq!(first, again);
        let events = crate::store::read_all(paths.store()).unwrap();
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(&e.op, Op::DeviceJoin { .. }))
                .count(),
            1
        );
    }

    #[test]
    fn an_agent_joins_as_one_and_the_state_says_so() {
        let tmp = tempfile::tempdir().unwrap();
        let paths = paths(tmp.path());

        let who = register(&paths).unwrap();
        let state = crate::State::replay(&crate::store::read_all(paths.store()).unwrap());

        assert!(
            state.agents.contains(&who),
            "a machine would not be listed here"
        );
        assert!(state.devices.contains(&who));
    }

    #[test]
    fn retiring_takes_the_voice_and_leaves_the_words() {
        let tmp = tempfile::tempdir().unwrap();
        let paths = paths(tmp.path());
        let who = register(&paths).unwrap();

        let mut store = Store::open(paths.store(), who.clone()).unwrap();
        let id = ulid::Ulid::generate();
        store
            .append(Op::TaskAdd {
                id,
                d: crate::event::TaskAdd::new("what it filed", "a0"),
            })
            .unwrap();

        assert_eq!(retire(&paths).unwrap(), Some(who.clone()));
        assert_eq!(registered(&paths).unwrap(), None);

        let state = crate::State::replay(&crate::store::read_all(paths.store()).unwrap());
        assert!(state.tasks.contains_key(&id), "retiring is not a purge");
        assert!(!state.agents.contains(&who));
    }

    #[test]
    fn retiring_when_none_was_registered_is_not_an_error() {
        let tmp = tempfile::tempdir().unwrap();
        assert_eq!(retire(&paths(tmp.path())).unwrap(), None);
    }
}
