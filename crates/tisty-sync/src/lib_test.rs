use super::segments::{Alike, hand_on, ours_reaches_further, ours_went_missing, same};
use super::*;
use std::path::{Path, PathBuf};
use tisty_core::event::{DeviceId, TaskAdd};
use tisty_core::{Op, Store};
use ulid::Ulid;

struct Machine {
    _dir: tempfile::TempDir,
    data: PathBuf,
    store: PathBuf,
    device: String,
}

fn blank(named: &str) -> Machine {
    let dir = tempfile::tempdir().unwrap();
    let data = dir.path().join("data");
    let store = data.join("store");
    Machine {
        _dir: dir,
        data,
        store,
        device: named.into(),
    }
}

fn machine(named: &str) -> Machine {
    let one = blank(named);
    wrote(&one, format!("lo de {named}"));
    one
}

fn filed(who: &Machine, file: &str, body: &str) {
    let mut held = signing(who);
    held.append(Op::DocAdd {
        id: Ulid::generate(),
        d: tisty_core::event::DocAdd {
            wrote: None,
            guest: false,
            made: None,
            by: None,
            said: None,
            file: file.to_string(),
            order: "a0".into(),
            folder: None,
            page_of: None,
        },
    })
    .unwrap();
    tisty_core::docs::write(&who.data.join(PAPERS), file, body).unwrap();
}

#[test]
fn a_meeting_place_that_went_empty_is_not_seeded_again_behind_our_back() {
    let one = machine("uno");
    let aside = tempfile::tempdir().unwrap();
    let shared = tempfile::tempdir().unwrap();
    carry_leaning_on(
        &one.data,
        Some(aside.path()),
        &one.device,
        shared.path(),
        Way::Both,
        &[],
    )
    .unwrap();
    assert!(shared.path().join(STORE).join(&one.device).exists());

    std::fs::remove_dir_all(shared.path().join(STORE)).unwrap();
    let why = carry_leaning_on(
        &one.data,
        Some(aside.path()),
        &one.device,
        shared.path(),
        Way::Both,
        &[],
    )
    .unwrap_err();

    assert_eq!(why, Trouble::Emptied(shared.path().display().to_string()));
    assert!(
        !shared.path().join(STORE).join(&one.device).exists(),
        "an unmounted drive must not be repopulated as if it were a fresh folder"
    );
}

#[test]
fn sending_it_all_again_is_how_a_person_says_the_empty_folder_is_the_right_one() {
    let one = machine("uno");
    let aside = tempfile::tempdir().unwrap();
    let shared = tempfile::tempdir().unwrap();
    carry_leaning_on(
        &one.data,
        Some(aside.path()),
        &one.device,
        shared.path(),
        Way::Both,
        &[],
    )
    .unwrap();
    std::fs::remove_dir_all(shared.path().join(STORE)).unwrap();

    carry_leaning_on(
        &one.data,
        Some(aside.path()),
        &one.device,
        shared.path(),
        Way::Again,
        &[],
    )
    .unwrap();

    assert!(
        shared.path().join(STORE).join(&one.device).exists(),
        "the way out of the guard is the button that says send it all again"
    );
}

#[test]
fn a_folder_we_never_carried_to_is_still_a_fresh_start() {
    let one = machine("uno");
    let aside = tempfile::tempdir().unwrap();
    let first = tempfile::tempdir().unwrap();
    let second = tempfile::tempdir().unwrap();
    let go = |dest: &std::path::Path| {
        carry_leaning_on(
            &one.data,
            Some(aside.path()),
            &one.device,
            dest,
            Way::Both,
            &[],
        )
    };
    go(first.path()).unwrap();

    go(second.path()).unwrap();

    assert!(second.path().join(STORE).join(&one.device).exists());
}

#[test]
fn a_shorter_history_arriving_first_never_replaces_the_longer_one_we_hold() {
    let one = machine("uno");
    for said in ["dos", "tres", "cuatro"] {
        wrote(&one, said.into());
    }
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    let two = blank("dos");
    carry(&two.data, &two.device, shared.path(), Way::Pull, &[]).unwrap();
    let held = tisty_core::store::check_device(&two.store.join(&one.device)).unwrap();
    assert!(held >= 4);

    let theirs = shared.path().join(STORE).join(&one.device);
    let at = theirs.join("active.tisty");
    let whole = std::fs::read_to_string(&at).unwrap();
    let first = whole.lines().next().unwrap();
    std::fs::write(&at, format!("{first}\n")).unwrap();

    carry(&two.data, &two.device, shared.path(), Way::Pull, &[]).unwrap();

    assert_eq!(
        tisty_core::store::check_device(&two.store.join(&one.device)).unwrap(),
        held,
        "una historia mas corta piso la que ya teniamos"
    );
}

static ALONE: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn watching(at: &Path) -> tisty_core::paths::Paths {
    let paths = tisty_core::paths::Paths::new(at.join("data"), at.join("config"));
    witness::keeps(tisty_core::witness::file(&paths), false);
    paths
}

fn said_about(paths: &tisty_core::paths::Paths, who: &str) -> Vec<String> {
    tisty_core::witness::recent(paths, 200)
        .into_iter()
        .filter(|line| line.contains("a shorter history") && line.contains(who))
        .collect()
}

fn trailing(theirs: &Path) {
    let at = theirs.join("active.tisty");
    let whole = std::fs::read_to_string(&at).unwrap();
    let first = whole.lines().next().unwrap();
    std::fs::write(
        &at,
        format!(
            "{first}
"
        ),
    )
    .unwrap();
}

#[test]
fn a_shared_folder_merely_behind_ours_is_pushed_to_without_a_word() {
    let _alone = ALONE.lock().unwrap_or_else(|e| e.into_inner());
    let one = machine("uno");
    for said in ["dos", "tres", "cuatro"] {
        wrote(&one, said.into());
    }
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    let two = blank("dos");
    carry(&two.data, &two.device, shared.path(), Way::Pull, &[]).unwrap();
    trailing(&shared.path().join(STORE).join(&one.device));

    let kept = tempfile::tempdir().unwrap();
    let paths = watching(kept.path());
    carry(&two.data, &two.device, shared.path(), Way::Both, &[]).unwrap();

    assert!(said_about(&paths, &one.device).is_empty());
    assert_eq!(
        tisty_core::store::check_device(&shared.path().join(STORE).join(&one.device)).unwrap(),
        tisty_core::store::check_device(&two.store.join(&one.device)).unwrap(),
    );
}

#[test]
fn a_shared_folder_that_walked_off_on_its_own_is_still_reported() {
    let _alone = ALONE.lock().unwrap_or_else(|e| e.into_inner());
    let one = machine("uno");
    for said in ["dos", "tres", "cuatro"] {
        wrote(&one, said.into());
    }
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    let two = blank("dos");
    carry(&two.data, &two.device, shared.path(), Way::Pull, &[]).unwrap();

    let apart = blank("uno");
    wrote(&apart, "otra vida".into());
    let theirs = shared.path().join(STORE).join(&one.device);
    std::fs::copy(
        apart.store.join(&apart.device).join("active.tisty"),
        theirs.join("active.tisty"),
    )
    .unwrap();

    let kept = tempfile::tempdir().unwrap();
    let paths = watching(kept.path());
    carry(&two.data, &two.device, shared.path(), Way::Pull, &[]).unwrap();

    assert!(!said_about(&paths, &one.device).is_empty());
}

#[test]
fn a_history_that_grew_on_the_other_side_still_comes_across() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    let two = blank("dos");
    carry(&two.data, &two.device, shared.path(), Way::Pull, &[]).unwrap();
    let before = tisty_core::store::check_device(&two.store.join(&one.device)).unwrap();

    wrote(&one, "algo mas".into());
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();
    carry(&two.data, &two.device, shared.path(), Way::Pull, &[]).unwrap();

    assert_eq!(
        tisty_core::store::check_device(&two.store.join(&one.device)).unwrap(),
        before + 1
    );
}

#[cfg(unix)]
#[test]
fn a_machine_folder_that_points_somewhere_else_never_receives_the_log() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    let elsewhere = tempfile::tempdir().unwrap();
    let there = shared.path().join(STORE);
    std::fs::create_dir_all(&there).unwrap();
    std::os::unix::fs::symlink(elsewhere.path(), there.join(&one.device)).unwrap();

    let outcome = carry(&one.data, &one.device, shared.path(), Way::Push, &[]);

    assert!(matches!(outcome, Err(Trouble::Refused(_))), "{outcome:?}");
    assert!(
        std::fs::read_dir(elsewhere.path())
            .unwrap()
            .next()
            .is_none(),
        "el log aterrizo donde apuntaba el enlace"
    );
}

#[cfg(unix)]
#[test]
fn a_shelf_that_points_somewhere_else_never_receives_an_attachment() {
    let one = machine("uno");
    let kept = planted(&one.data, "foto.png", b"unos bytes cualesquiera");
    let shelf = kept.split('/').nth(1).unwrap().to_string();
    let shared = tempfile::tempdir().unwrap();
    let elsewhere = tempfile::tempdir().unwrap();
    let there = shared.path().join(HELD);
    std::fs::create_dir_all(&there).unwrap();
    std::os::unix::fs::symlink(elsewhere.path(), there.join(&shelf)).unwrap();

    let outcome = carry(&one.data, &one.device, shared.path(), Way::Push, &[]);

    assert!(matches!(outcome, Err(Trouble::Refused(_))), "{outcome:?}");
    assert!(
        std::fs::read_dir(elsewhere.path())
            .unwrap()
            .next()
            .is_none(),
        "el adjunto aterrizo donde apuntaba el enlace"
    );
}

#[cfg(unix)]
#[test]
fn a_folder_with_no_links_in_it_still_carries_as_it_always_did() {
    let one = machine("uno");
    planted(&one.data, "foto.png", b"unos bytes cualesquiera");
    let shared = tempfile::tempdir().unwrap();

    let moved = carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    assert!(moved.sent > 0);
}

#[test]
fn the_seam_is_written_down_before_the_new_name_is_taken() {
    let one = machine("uno");
    let two = machine("dos");
    let shared = tempfile::tempdir().unwrap();
    carry(&two.data, &two.device, shared.path(), Way::Push, &[]).unwrap();
    let was = tisty_core::store::identity(&one.store).unwrap();

    stitch(&one.data, &one.device, shared.path(), None).unwrap();

    let said = tisty_core::State::replay(&tisty_core::store::read_all(&one.store).unwrap());
    assert_eq!(said.forebears.len(), 2, "la costura no quedo en el log");
    assert!(said.forebears.contains(&was));
}

#[test]
fn a_seam_left_half_done_can_still_be_finished_and_says_the_same_thing() {
    let one = machine("uno");
    let two = machine("dos");
    let shared = tempfile::tempdir().unwrap();
    carry(&two.data, &two.device, shared.path(), Way::Push, &[]).unwrap();

    let was = tisty_core::store::identity(&one.store).unwrap();
    stitch(&one.data, &one.device, shared.path(), None).unwrap();
    std::fs::write(one.store.join(MARKER), was.as_bytes()).unwrap();

    stitch(&one.data, &one.device, shared.path(), None).unwrap();

    let said = tisty_core::State::replay(&tisty_core::store::read_all(&one.store).unwrap());
    assert!(said.forebears.contains(&was), "el linaje viejo se perdio");
    assert_eq!(
        tisty_core::store::peek_identity(&one.store).unwrap(),
        tisty_core::store::peek_identity(shared.path().join(STORE)).unwrap()
    );
}

#[cfg(unix)]
#[test]
fn a_document_that_is_a_link_to_a_secret_never_becomes_one_of_ours() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    let elsewhere = tempfile::tempdir().unwrap();
    let secret = elsewhere.path().join("id_ed25519");
    std::fs::write(&secret, b"PRIVATE KEY nadie deberia ver esto").unwrap();
    says(
        &one,
        Op::DocAdd {
            id: Ulid::generate(),
            d: tisty_core::event::DocAdd {
                wrote: None,
                guest: false,
                made: None,
                by: None,
                file: "uno-0001".into(),
                order: "a0".into(),
                said: None,
                folder: None,
                page_of: None,
            },
        },
    );
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    let there = shared.path().join(PAPERS);
    std::fs::create_dir_all(&there).unwrap();
    std::os::unix::fs::symlink(&secret, there.join("uno-0001.md")).unwrap();

    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

    let landed = one.data.join(PAPERS).join("uno-0001.md");
    assert!(
        !landed.exists()
            || !std::fs::read_to_string(&landed)
                .unwrap()
                .contains("PRIVATE KEY"),
        "el secreto entro como documento nuestro"
    );
}

#[cfg(unix)]
#[test]
fn keeping_theirs_never_reads_through_a_link_either() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    let elsewhere = tempfile::tempdir().unwrap();
    let secret = elsewhere.path().join("id_ed25519");
    std::fs::write(&secret, b"PRIVATE KEY nadie deberia ver esto").unwrap();
    tisty_core::docs::write(&one.data.join(PAPERS), "uno-0001", "# Lo mio\n").unwrap();

    let there = shared.path().join(PAPERS);
    std::fs::create_dir_all(&there).unwrap();
    std::os::unix::fs::symlink(&secret, there.join("uno-0001.md")).unwrap();

    let outcome = settle(&one.data, shared.path(), "uno-0001", Keep::Theirs);

    assert!(matches!(outcome, Err(Trouble::Refused(_))), "{outcome:?}");
    assert_eq!(
        std::fs::read_to_string(one.data.join(PAPERS).join("uno-0001.md")).unwrap(),
        "# Lo mio\n"
    );
}

#[test]
fn a_retired_attachment_is_never_pushed_back_up_to_the_folder() {
    let one = machine("uno");
    let kept = planted(&one.data, "foto.png", b"una fotografia retirada");
    let shared = tempfile::tempdir().unwrap();

    says(&one, Op::AttachRetire { d: kept.clone() });
    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

    assert!(
        !shared.path().join(&kept).exists(),
        "lo retirado se subio a la carpeta compartida"
    );
}

#[test]
fn an_attachment_nobody_retired_still_goes_up() {
    let one = machine("uno");
    let kept = planted(&one.data, "foto.png", b"una fotografia cualquiera");
    let shared = tempfile::tempdir().unwrap();

    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

    assert!(shared.path().join(&kept).is_file());
}

fn rotated(who: &Machine) {
    let at = who.store.join(&who.device);
    let whole = std::fs::read_to_string(at.join("active.tisty")).unwrap();
    let lines: Vec<&str> = whole.lines().collect();
    let (closed, tail) = lines.split_at(lines.len() - 1);
    std::fs::write(at.join("000001.tisty"), format!("{}\n", closed.join("\n"))).unwrap();
    std::fs::write(at.join("000001.count"), closed.len().to_string()).unwrap();
    std::fs::write(at.join("active.tisty"), format!("{}\n", tail.join("\n"))).unwrap();
}

#[test]
fn a_pull_cut_between_two_segments_does_not_wedge_the_next_one() {
    let one = machine("uno");
    for said in ["dos", "tres", "cuatro"] {
        wrote(&one, said.into());
    }
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    let two = blank("dos");
    carry(&two.data, &two.device, shared.path(), Way::Pull, &[]).unwrap();

    rotated(&one);
    let theirs = shared.path().join(STORE).join(&one.device);
    let mine = two.store.join(&one.device);
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();
    for leaf in ["000001.tisty", "000001.count"] {
        std::fs::copy(theirs.join(leaf), mine.join(leaf)).unwrap();
    }

    wrote(&one, "lo que vino despues de rotar".into());
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();
    carry(&two.data, &two.device, shared.path(), Way::Pull, &[]).unwrap();

    assert_eq!(
        tisty_core::store::distinct_in(&mine).unwrap(),
        tisty_core::store::distinct_in(&theirs).unwrap(),
        "la maquina quedo atascada y no recibe nada mas"
    );
}

#[test]
fn a_torn_local_copy_is_never_replaced_by_a_shorter_history() {
    let one = machine("uno");
    for said in ["dos", "tres", "cuatro"] {
        wrote(&one, said.into());
    }
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    let two = blank("dos");
    carry(&two.data, &two.device, shared.path(), Way::Pull, &[]).unwrap();

    let mine = two.store.join(&one.device).join("active.tisty");
    let whole = std::fs::read_to_string(&mine).unwrap();
    std::fs::write(&mine, format!("{whole}{{\"v\":3,\"ts\"")).unwrap();

    let theirs = shared
        .path()
        .join(STORE)
        .join(&one.device)
        .join("active.tisty");
    let short = std::fs::read_to_string(&theirs).unwrap();
    let first = short.lines().next().unwrap();
    std::fs::write(&theirs, format!("{first}\n")).unwrap();

    carry(&two.data, &two.device, shared.path(), Way::Pull, &[]).unwrap();

    let after = std::fs::read_to_string(&mine).unwrap();
    assert!(
        after.starts_with(&whole),
        "una historia corta piso la copia local rota"
    );
}

#[test]
fn a_machine_that_was_removed_cannot_stitch_itself_into_the_folder() {
    let one = machine("uno");
    let two = machine("dos");
    says(
        &one,
        Op::DeviceJoin {
            d: DeviceId("dev_otra".into()),
            k: Some(tisty_core::DeviceKind::Machine),
            p: None,
        },
    );
    says(
        &one,
        Op::DeviceRemove {
            d: DeviceId(one.device.clone()),
        },
    );
    let shared = tempfile::tempdir().unwrap();
    carry(&two.data, &two.device, shared.path(), Way::Push, &[]).unwrap();
    let was = tisty_core::store::identity(&one.store).unwrap();

    let outcome = stitch(&one.data, &one.device, shared.path(), None);

    assert!(
        matches!(outcome, Err(Trouble::NotAllowed(_))),
        "{outcome:?}"
    );
    assert_eq!(
        tisty_core::store::peek_identity(&one.store).as_deref(),
        Some(was.as_str()),
        "adopto un nombre en el que no puede escribir"
    );
}

#[test]
fn a_segment_that_cannot_be_read_is_never_called_a_clash() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();
    assert_eq!(kinship(&one.store, shared.path()), Kin::SameLineage);

    let mine = one.store.join(&one.device);
    let file = tisty_core::store::segments_in(&mine).unwrap().remove(0);
    std::fs::remove_file(&file).unwrap();
    std::fs::create_dir(&file).unwrap();

    assert_eq!(
        kinship(&one.store, shared.path()),
        Kin::Unsure(one.device.clone())
    );
    assert!(matches!(
        stitch(&one.data, &one.device, shared.path(), None),
        Err(Trouble::Unreadable(_))
    ));
}

#[test]
fn a_clash_is_found_even_when_another_machine_already_proved_the_lineage() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    let both = "compartida";
    for at in [one.store.join(both), shared.path().join(STORE).join(both)] {
        std::fs::create_dir_all(&at).unwrap();
    }
    std::fs::write(one.store.join(both).join("active.tisty"), b"lo de aqui\n").unwrap();
    std::fs::write(
        shared.path().join(STORE).join(both).join("active.tisty"),
        b"lo de alli\n",
    )
    .unwrap();

    assert_eq!(
        kinship(&one.store, shared.path()),
        Kin::Clash(both.to_string()),
        "un choque real se perdio porque otra maquina ya habia probado el linaje"
    );
}

#[test]
fn stitching_twice_never_writes_a_seam_that_joins_a_history_to_itself() {
    let one = machine("uno");
    let two = machine("dos");
    let shared = tempfile::tempdir().unwrap();
    carry(&two.data, &two.device, shared.path(), Way::Push, &[]).unwrap();
    tisty_core::store::identity(&one.store).unwrap();

    stitch(&one.data, &one.device, shared.path(), None).unwrap();
    let again = stitch(&one.data, &one.device, shared.path(), None).unwrap();

    assert!(again.stitch.is_none(), "anoto una costura de si misma");
    let seams = tisty_core::store::read_all(&one.store)
        .unwrap()
        .into_iter()
        .filter(|one| matches!(one.op, Op::StoresJoined { .. }))
        .count();
    assert_eq!(seams, 1);
}

#[test]
fn a_seam_says_which_history_was_absorbed_and_which_one_survived() {
    let one = machine("uno");
    let two = machine("dos");
    let shared = tempfile::tempdir().unwrap();
    carry(&two.data, &two.device, shared.path(), Way::Push, &[]).unwrap();
    let was = tisty_core::store::identity(&one.store).unwrap();

    let done = stitch(&one.data, &one.device, shared.path(), None).unwrap();

    let seam = done.stitch.unwrap();
    assert_eq!(seam.absorbed, was);
    assert_eq!(
        seam.survivor,
        tisty_core::store::peek_identity(shared.path().join(STORE)).unwrap()
    );
    assert_ne!(seam.absorbed, seam.survivor);
}

#[test]
fn a_seam_says_which_machines_came_from_each_side() {
    let one = machine("uno");
    let two = machine("dos");
    let shared = tempfile::tempdir().unwrap();
    carry(&two.data, &two.device, shared.path(), Way::Push, &[]).unwrap();
    tisty_core::store::identity(&one.store).unwrap();

    let seam = stitch(&one.data, &one.device, shared.path(), None)
        .unwrap()
        .stitch
        .unwrap();

    assert!(seam.ours.contains(&DeviceId(one.device.clone())));
    assert!(seam.theirs.contains(&DeviceId(two.device.clone())));
    assert!(!seam.ours.contains(&DeviceId(two.device.clone())));
}

#[test]
fn stitching_where_there_is_no_folder_says_so_instead_of_something_else() {
    let one = machine("uno");
    let nowhere = tempfile::tempdir().unwrap();

    let outcome = stitch(
        &one.data,
        &one.device,
        &nowhere.path().join("no-esta"),
        None,
    );

    assert!(matches!(outcome, Err(Trouble::NotThere(_))), "{outcome:?}");
}

#[test]
fn a_document_too_big_to_read_is_named_instead_of_vanishing_from_the_sync() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    let here = one.data.join(PAPERS);
    std::fs::create_dir_all(&here).unwrap();
    let there = shared.path().join(PAPERS);
    std::fs::create_dir_all(&there).unwrap();
    let huge = "x".repeat((tisty_core::docs::BODY_AT_MOST + 1) as usize);
    std::fs::write(here.join("uno-0001.md"), &huge).unwrap();
    std::fs::write(there.join("uno-0001.md"), "# Lo de alli\n").unwrap();

    let moved = carry_papers(&one.data, shared.path(), &["uno-0001".into()]).unwrap();

    assert_eq!(moved.astray, vec!["uno-0001".to_string()]);
    assert!(moved.undecided.is_empty());
}

#[test]
fn a_document_that_reads_fine_is_never_called_astray() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    let here = one.data.join(PAPERS);
    std::fs::create_dir_all(&here).unwrap();
    std::fs::write(here.join("uno-0001.md"), "# Lo mio\n").unwrap();

    let moved = carry_papers(&one.data, shared.path(), &["uno-0001".into()]).unwrap();

    assert!(moved.astray.is_empty());
}

#[test]
fn the_body_that_settled_is_kept_so_the_next_round_has_a_base() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    filed(&one, "uno-0001", "# Kit\n\nlo que quedo asentado\n");

    carry_papers(&one.data, shared.path(), &["uno-0001".into()]).unwrap();

    assert_eq!(
        tisty_core::docs::read_carried(&one.data, "uno-0001").as_deref(),
        Some("# Kit\n\nlo que quedo asentado\n")
    );
}

#[test]
fn the_base_follows_the_body_when_the_other_side_wins() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    let there = shared.path().join(PAPERS);
    std::fs::create_dir_all(&there).unwrap();
    filed(&one, "uno-0001", "# Kit\n\nlo mio\n");
    carry_papers(&one.data, shared.path(), &["uno-0001".into()]).unwrap();

    std::fs::write(there.join("uno-0001.md"), "# Kit\n\nlo de alli\n").unwrap();
    carry_papers(&one.data, shared.path(), &["uno-0001".into()]).unwrap();

    assert_eq!(
        tisty_core::docs::read_carried(&one.data, "uno-0001").as_deref(),
        Some("# Kit\n\nlo de alli\n"),
        "la base se quedo con lo viejo"
    );
}

#[test]
fn a_base_is_never_kept_for_a_document_neither_side_has() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();

    carry_papers(&one.data, shared.path(), &["uno-0001".into()]).unwrap();

    assert!(tisty_core::docs::read_carried(&one.data, "uno-0001").is_none());
}

#[test]
fn the_base_never_travels_to_the_shared_folder() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    filed(&one, "uno-0001", "# Kit\n\ncuerpo\n");

    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

    assert!(!shared.path().join("carried").exists());
    assert!(one.data.join("carried").is_dir());
}

fn wrote_body(at: &Path, id: &str, body: &str) {
    std::fs::create_dir_all(at).unwrap();
    std::fs::write(at.join(format!("{id}.md")), body).unwrap();
}

#[test]
fn two_machines_touching_different_parts_end_up_with_one_document_and_no_question() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    let here = one.data.join(PAPERS);
    let there = shared.path().join(PAPERS);
    let base = "# Kit\n\nla introduccion\n\nel cuerpo\n\nel cierre\n";
    filed(&one, "uno-0001", base);
    carry_papers(&one.data, shared.path(), &["uno-0001".into()]).unwrap();

    wrote_body(
        &here,
        "uno-0001",
        "# Kit\n\nla introduccion del mac\n\nel cuerpo\n\nel cierre\n",
    );
    wrote_body(
        &there,
        "uno-0001",
        "# Kit\n\nla introduccion\n\nel cuerpo\n\nel cierre\n\nlo de windows\n",
    );

    let done = carry_papers(&one.data, shared.path(), &["uno-0001".into()]).unwrap();

    assert!(done.undecided.is_empty(), "pregunto pudiendo juntarlo");
    assert_eq!(done.joined, vec!["uno-0001".to_string()]);
    assert_eq!(
        (done.sent, done.brought),
        (0, 1),
        "the join waits here for the log that answers for it"
    );
    let whole = std::fs::read_to_string(here.join("uno-0001.md")).unwrap();
    assert!(whole.contains("del mac"), "{whole}");
    assert!(whole.contains("lo de windows"), "{whole}");

    let after = carry_papers(&one.data, shared.path(), &["uno-0001".into()]).unwrap();
    assert_eq!(after.sent, 1, "{after:?}");
    assert_eq!(
        whole,
        std::fs::read_to_string(there.join("uno-0001.md")).unwrap()
    );
}

#[test]
fn the_same_paragraph_written_two_ways_still_asks() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    let here = one.data.join(PAPERS);
    let there = shared.path().join(PAPERS);
    filed(&one, "uno-0001", "# Kit\n\nla introduccion\n\nel cierre\n");
    carry_papers(&one.data, shared.path(), &["uno-0001".into()]).unwrap();

    wrote_body(
        &here,
        "uno-0001",
        "# Kit\n\nla introduccion del mac\n\nel cierre\n",
    );
    wrote_body(
        &there,
        "uno-0001",
        "# Kit\n\nla introduccion de windows\n\nel cierre\n",
    );

    let done = carry_papers(&one.data, shared.path(), &["uno-0001".into()]).unwrap();

    assert_eq!(done.undecided.len(), 1);
    assert!(done.joined.is_empty());
}

#[test]
fn without_a_base_nothing_is_joined_and_the_person_still_decides() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    let here = one.data.join(PAPERS);
    let there = shared.path().join(PAPERS);
    filed(&one, "uno-0001", "# Kit\n\nuno\n\ndos\n");
    carry_papers(&one.data, shared.path(), &["uno-0001".into()]).unwrap();
    tisty_core::docs::forget_carried(&one.data, "uno-0001");

    wrote_body(&here, "uno-0001", "# Kit del mac\n\nuno\n\ndos\n");
    wrote_body(&there, "uno-0001", "# Kit\n\nuno\n\ndos\n\ntres\n");

    let done = carry_papers(&one.data, shared.path(), &["uno-0001".into()]).unwrap();

    assert_eq!(done.undecided.len(), 1);
    assert!(done.joined.is_empty());
}

#[test]
fn a_join_that_would_name_an_attachment_we_do_not_hold_is_never_written() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    let here = one.data.join(PAPERS);
    let there = shared.path().join(PAPERS);
    filed(&one, "uno-0001", "# Kit\n\nuno\n\ndos\n");
    carry_papers(&one.data, shared.path(), &["uno-0001".into()]).unwrap();

    wrote_body(&here, "uno-0001", "# Kit del mac\n\nuno\n\ndos\n");
    wrote_body(
        &there,
        "uno-0001",
        "# Kit\n\nuno\n\ndos\n\n![foto](attachments/ab/foto-a1b2c3d4.png)\n",
    );

    let done = carry_papers(&one.data, shared.path(), &["uno-0001".into()]).unwrap();

    assert!(done.joined.is_empty(), "junto una referencia rota");
    assert_eq!(done.undecided.len(), 1);
}

#[test]
fn a_join_leaves_the_base_on_what_both_sides_now_hold() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    let here = one.data.join(PAPERS);
    let there = shared.path().join(PAPERS);
    filed(&one, "uno-0001", "# Kit\n\nuno\n\ndos\n");
    carry_papers(&one.data, shared.path(), &["uno-0001".into()]).unwrap();

    wrote_body(&here, "uno-0001", "# Kit del mac\n\nuno\n\ndos\n");
    wrote_body(&there, "uno-0001", "# Kit\n\nuno\n\ndos\n\ntres\n");
    carry_papers(&one.data, shared.path(), &["uno-0001".into()]).unwrap();

    let done = carry_papers(&one.data, shared.path(), &["uno-0001".into()]).unwrap();

    assert!(done.joined.is_empty(), "volvio a juntar lo ya junto");
    assert_eq!(
        tisty_core::docs::read_carried(&one.data, "uno-0001"),
        Some(std::fs::read_to_string(here.join("uno-0001.md")).unwrap())
    );
}

#[test]
fn two_histories_that_never_met_are_strangers() {
    let one = machine("uno");
    let two = machine("dos");
    let shared = tempfile::tempdir().unwrap();
    carry(&two.data, &two.device, shared.path(), Way::Push, &[]).unwrap();

    assert_eq!(kinship(&one.store, shared.path()), Kin::Strangers);
}

#[test]
fn a_folder_that_already_holds_everything_of_ours_is_the_same_lineage() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    assert_eq!(kinship(&one.store, shared.path()), Kin::SameLineage);
}

#[test]
fn a_tail_we_never_sent_is_still_the_same_lineage_not_a_clash() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();
    wrote(&one, "algo que se quedo aqui".into());

    assert_eq!(kinship(&one.store, shared.path()), Kin::SameLineage);
}

#[test]
fn the_same_name_writing_two_different_things_is_the_clash_that_is_refused() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    let theirs = shared.path().join(STORE).join(&one.device);
    let file = tisty_core::store::segments_in(&theirs).unwrap().remove(0);
    let mut said = std::fs::read(&file).unwrap();
    said[0] ^= 0xff;
    std::fs::write(&file, said).unwrap();

    assert_eq!(
        kinship(&one.store, shared.path()),
        Kin::Clash(one.device.clone())
    );
}

#[test]
fn a_folder_ahead_of_us_is_the_same_lineage_because_only_the_end_grows() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    let theirs = shared.path().join(STORE).join(&one.device);
    let file = tisty_core::store::segments_in(&theirs).unwrap().remove(0);
    let mut said = std::fs::read(&file).unwrap();
    said.extend_from_slice(b"{\"v\":3}\n");
    std::fs::write(&file, said).unwrap();

    assert_eq!(kinship(&one.store, shared.path()), Kin::SameLineage);
}

#[test]
fn a_document_written_on_the_other_machine_lands_on_the_first_sync_not_the_second() {
    let shared = tempfile::tempdir().unwrap();
    let one = machine("uno");
    let two = blank("dos");

    filed(&one, "uno-0001", "# Ortografia\n\nla n con virgulilla\n");
    carry(
        &one.data,
        &one.device,
        shared.path(),
        Way::Both,
        &["uno-0001".into()],
    )
    .unwrap();

    carry(&two.data, &two.device, shared.path(), Way::Both, &[]).unwrap();

    let landed = two.data.join(PAPERS).join("uno-0001.md");
    assert!(
        landed.is_file(),
        "el documento no llego en la primera vuelta"
    );
    assert_eq!(
        std::fs::read_to_string(landed).unwrap(),
        "# Ortografia\n\nla n con virgulilla\n"
    );
}

#[test]
fn a_document_the_other_machine_deleted_is_never_brought_back_by_the_new_reckoning() {
    let shared = tempfile::tempdir().unwrap();
    let one = machine("uno");
    let two = blank("dos");

    filed(&one, "uno-0001", "# Algo\n");
    carry(
        &one.data,
        &one.device,
        shared.path(),
        Way::Both,
        &["uno-0001".into()],
    )
    .unwrap();

    let mut held = signing(&one);
    let id = tisty_core::State::replay(&tisty_core::store::read_all(&one.store).unwrap())
        .docs
        .values()
        .next()
        .unwrap()
        .id;
    held.append(Op::DocDelete { id }).unwrap();
    drop(held);
    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

    carry(&two.data, &two.device, shared.path(), Way::Both, &[]).unwrap();

    assert!(!two.data.join(PAPERS).join("uno-0001.md").exists());
}

fn signing(who: &Machine) -> Store {
    let whose = DeviceId(who.device.clone());
    let paths = tisty_core::Paths::new(who.data.clone(), who.data.join("config"));
    let key = tisty_core::signing::mine(&paths, &whose);
    if let Some(shown) = key.as_ref().map(tisty_core::signing::shown) {
        std::fs::create_dir_all(&who.data).unwrap();
        tisty_core::vouched::confirm(&who.data, &whose, &shown);
    }
    Store::open(&who.store, whose.clone())
        .unwrap()
        .signing_with(key)
}

fn wrote(who: &Machine, title: String) {
    let mut held = signing(who);
    held.append(Op::TaskAdd {
        id: Ulid::generate(),
        d: TaskAdd::new(title, "a0"),
    })
    .unwrap();
}

fn joined(who: &Machine, shared: &Path) {
    carry(&who.data, &who.device, shared, Way::Pull, &[]).unwrap();
}

fn titles(store: &Path) -> Vec<String> {
    tisty_core::State::replay(&tisty_core::store::read_all(store).unwrap())
        .tasks
        .values()
        .map(|task| task.title.clone())
        .collect()
}

fn says(who: &Machine, op: Op) {
    let mut held = signing(who);
    held.append(op).unwrap();
}

fn signs(who: &Machine, alias: &str) {
    says(
        who,
        Op::Signed {
            d: tisty_core::event::Signature {
                alias: Some(alias.to_string()),
                name: None,
                email: None,
            },
        },
    );
}

#[test]
fn a_folder_says_it_stirred_without_being_read() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    let still = stirring(shared.path());
    assert_eq!(still, stirring(shared.path()), "a folder at rest changed");

    wrote(&one, "lo que vino despues".into());
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    assert_ne!(
        still,
        stirring(shared.path()),
        "the folder grew and said nothing"
    );
}

#[test]
fn a_seat_nobody_ever_wrote_in_is_not_taken_home() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();
    std::fs::create_dir_all(shared.path().join(STORE).join("dev_ghost")).unwrap();

    let fresh = blank("dos");
    carry(&fresh.data, &fresh.device, shared.path(), Way::Pull, &[]).unwrap();

    assert!(
        !fresh.store.join("dev_ghost").exists(),
        "an empty device directory travelled as though it were a machine"
    );
}

#[test]
fn a_machine_that_wrote_nothing_takes_up_the_folder_it_is_pointed_at() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    let fresh = blank("dos");
    carry(&fresh.data, &fresh.device, shared.path(), Way::Both, &[]).unwrap();

    assert_eq!(titles(&fresh.store), vec!["lo de uno".to_string()]);
    assert_eq!(
        signed_at(shared.path()),
        None,
        "nobody signed, so there is no name to take"
    );
}

#[test]
fn signing_before_the_folder_is_chosen_makes_a_new_machine_look_like_another_history() {
    let one = machine("uno");
    signs(&one, "mario");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    let fresh = blank("dos");
    signs(&fresh, "mario");
    let outcome = carry(&fresh.data, &fresh.device, shared.path(), Way::Both, &[]);

    assert!(
        matches!(
            outcome,
            Err(Trouble::OtherStore { .. }) | Err(Trouble::WouldReset { .. })
        ),
        "a name of its own is a history of its own: {outcome:?}"
    );
    assert_eq!(
        signed_at(shared.path()).as_deref(),
        Some("mario"),
        "the folder carries the name, so asking for one first is asking twice"
    );
}

#[test]
fn a_store_with_no_list_yet_lets_everyone_write() {
    let one = machine("dev_a");
    let shared = tempfile::tempdir().unwrap();

    let moved = carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

    assert!(moved.sent > 0, "an older store must not be locked out");
}

#[test]
fn a_machine_on_the_list_writes_as_it_always_did() {
    let one = machine("dev_a");
    says(
        &one,
        Op::DeviceJoin {
            d: DeviceId("dev_a".into()),
            k: Some(tisty_core::DeviceKind::Machine),
            p: None,
        },
    );
    let shared = tempfile::tempdir().unwrap();

    let moved = carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

    assert!(moved.sent > 0);
}

fn planted(root: &Path, called: &str, body: &[u8]) -> String {
    let dir = tempfile::tempdir().unwrap();
    let at = dir.path().join(called);
    std::fs::write(&at, body).unwrap();
    tisty_core::attach::keep(&at, root, tisty_core::attach::COPIED_IN_DOC)
        .unwrap()
        .at
}

fn paper(who: &Machine, id: &str, body: &str) {
    let at = who.data.join("docs");
    std::fs::create_dir_all(&at).unwrap();
    std::fs::write(at.join(format!("{id}.md")), body).unwrap();
}

fn theirs(shared: &Path, id: &str, body: &str) {
    let at = shared.join("docs");
    std::fs::create_dir_all(&at).unwrap();
    std::fs::write(at.join(format!("{id}.md")), body).unwrap();
}

fn body(at: &Path, id: &str) -> String {
    std::fs::read_to_string(at.join("docs").join(format!("{id}.md"))).unwrap()
}

fn at_odds(one: &Machine, shared: &Path) -> Vec<String> {
    let alive = vec!["dev_a-0001".to_string()];
    paper(one, "dev_a-0001", "# Minuta");
    carry_papers(&one.data, shared, &alive).unwrap();
    paper(one, "dev_a-0001", "# Minuta\n\nlo mio");
    theirs(shared, "dev_a-0001", "# Minuta\n\nlo suyo");
    alive
}

#[test]
fn keeping_mine_leaves_the_folder_holding_mine_and_asks_no_more() {
    let one = machine("dev_a");
    let shared = tempfile::tempdir().unwrap();
    let alive = at_odds(&one, shared.path());

    let brought = settle(&one.data, shared.path(), "dev_a-0001", Keep::Mine).unwrap();

    assert_eq!(brought, None);
    assert_eq!(body(shared.path(), "dev_a-0001"), "# Minuta\n\nlo mio");
    let done = carry_papers(&one.data, shared.path(), &alive).unwrap();
    assert!(
        done.undecided.is_empty(),
        "it asked again about a settled one"
    );
}

#[test]
fn what_is_written_after_settling_travels_instead_of_being_asked_about_again() {
    let one = machine("dev_a");
    let shared = tempfile::tempdir().unwrap();
    let alive = at_odds(&one, shared.path());
    settle(&one.data, shared.path(), "dev_a-0001", Keep::Mine).unwrap();

    paper(&one, "dev_a-0001", "# Minuta\n\nlo mio, y algo mas");
    let done = carry_papers(&one.data, shared.path(), &alive).unwrap();

    assert!(
        done.undecided.is_empty(),
        "settling it did not become the new common ground"
    );
    assert_eq!(done.sent, 1);
    assert_eq!(
        body(shared.path(), "dev_a-0001"),
        "# Minuta\n\nlo mio, y algo mas"
    );
}

#[test]
fn keeping_theirs_takes_it_home_and_asks_no_more() {
    let one = machine("dev_a");
    let shared = tempfile::tempdir().unwrap();
    let alive = at_odds(&one, shared.path());

    settle(&one.data, shared.path(), "dev_a-0001", Keep::Theirs).unwrap();

    assert_eq!(body(&one.data, "dev_a-0001"), "# Minuta\n\nlo suyo");
    let done = carry_papers(&one.data, shared.path(), &alive).unwrap();
    assert!(
        done.undecided.is_empty(),
        "it asked again about a settled one"
    );
}

#[test]
fn keeping_both_hands_back_the_other_body_before_anything_is_overwritten() {
    let one = machine("dev_a");
    let shared = tempfile::tempdir().unwrap();
    let alive = at_odds(&one, shared.path());

    let brought = settle(&one.data, shared.path(), "dev_a-0001", Keep::Both).unwrap();

    assert_eq!(brought.as_deref(), Some("# Minuta\n\nlo suyo"));
    assert_eq!(
        body(shared.path(), "dev_a-0001"),
        "# Minuta\n\nlo suyo",
        "it overwrote the other version before it was anywhere safe"
    );
    assert_eq!(body(&one.data, "dev_a-0001"), "# Minuta\n\nlo mio");
    let done = carry_papers(&one.data, shared.path(), &alive).unwrap();
    assert_eq!(done.undecided.len(), 1, "it settled before being told to");
}

#[test]
fn keeping_both_settles_once_the_other_version_is_safe() {
    let one = machine("dev_a");
    let shared = tempfile::tempdir().unwrap();
    let alive = at_odds(&one, shared.path());
    settle(&one.data, shared.path(), "dev_a-0001", Keep::Both).unwrap();

    settle(&one.data, shared.path(), "dev_a-0001", Keep::Mine).unwrap();

    assert_eq!(body(shared.path(), "dev_a-0001"), "# Minuta\n\nlo mio");
    let done = carry_papers(&one.data, shared.path(), &alive).unwrap();
    assert!(done.undecided.is_empty());
}

#[cfg(unix)]
#[test]
fn a_meeting_place_that_points_somewhere_else_is_refused_before_anything_leaves() {
    let one = machine("dev_a");
    let shared = tempfile::tempdir().unwrap();
    let elsewhere = tempfile::tempdir().unwrap();
    std::os::unix::fs::symlink(elsewhere.path(), shared.path().join(STORE)).unwrap();

    let why = carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap_err();

    assert!(
        matches!(why, Trouble::Refused(_)),
        "it followed the link out of the folder: {why:?}"
    );
    assert!(
        std::fs::read_dir(elsewhere.path())
            .unwrap()
            .next()
            .is_none(),
        "the log was copied where the link pointed"
    );
}

#[cfg(unix)]
#[test]
fn a_documents_folder_that_points_somewhere_else_never_receives_a_body() {
    let one = machine("dev_a");
    let shared = tempfile::tempdir().unwrap();
    let elsewhere = tempfile::tempdir().unwrap();
    std::os::unix::fs::symlink(elsewhere.path(), shared.path().join(PAPERS)).unwrap();
    paper(&one, "dev_a-0001", "# Lo que dije");

    let why = carry_papers(&one.data, shared.path(), &["dev_a-0001".into()]).unwrap_err();

    assert!(matches!(why, Trouble::Refused(_)), "{why:?}");
    assert!(
        std::fs::read_dir(elsewhere.path())
            .unwrap()
            .next()
            .is_none(),
        "the body was written where the link pointed"
    );
}

#[test]
fn a_body_the_reader_would_refuse_never_replaces_the_one_that_is_here() {
    let one = blank("dev_a");
    let shared = tempfile::tempdir().unwrap();
    let alive = ["dev_a-0001".to_string()];
    paper(&one, "dev_a-0001", "# Lo mio, breve");
    theirs(shared.path(), "dev_a-0001", &"x".repeat(600 * 1024));

    let done = carry_papers(&one.data, shared.path(), &alive);

    assert!(done.is_err() || done.unwrap().brought == 0);
    assert_eq!(
        body(&one.data, "dev_a-0001"),
        "# Lo mio, breve",
        "a body nobody can open replaced one that could be read"
    );
}

#[test]
fn an_attachment_whose_bytes_were_swapped_never_reaches_this_machine() {
    let one = machine("dev_a");
    let shared = tempfile::tempdir().unwrap();
    let (_src, file) = {
        let dir = tempfile::tempdir().unwrap();
        let at = dir.path().join("contrato.pdf");
        std::fs::write(&at, b"what the person really attached").unwrap();
        (dir, at)
    };
    let kept =
        tisty_core::attach::keep(&file, &one.data, tisty_core::attach::COPIED_UP_TO).unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    let theirs = shared.path().join(&kept.at);
    std::fs::write(&theirs, b"a different file wearing the same name").unwrap();
    std::fs::remove_file(one.data.join(&kept.at)).unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Pull, &[]).unwrap();

    assert!(
        !one.data.join(&kept.at).exists(),
        "bytes nobody vouched for were taken in under a trusted name"
    );
}

#[test]
fn what_we_already_kept_is_not_replaced_by_something_the_name_alone_allows() {
    let one = machine("dev_a");
    let shared = tempfile::tempdir().unwrap();
    let kept = planted(
        shared.path(),
        "contrato.pdf",
        b"the bytes whose name this is",
    );

    let mine = one.data.join(&kept);
    std::fs::create_dir_all(mine.parent().unwrap()).unwrap();
    std::fs::write(&mine, b"what we kept, of another length").unwrap();
    std::fs::write(
        one.data.join("attachments.jsonl"),
        format!(
            "{{\"at\":\"{kept}\",\"sha256\":\"{}\",\"bytes\":31}}\n",
            "0".repeat(64)
        ),
    )
    .unwrap();

    carry(&one.data, &one.device, shared.path(), Way::Pull, &[]).unwrap();

    assert_eq!(
        std::fs::read(&mine).unwrap(),
        b"what we kept, of another length",
        "the name alone was enough to replace what we had written down"
    );
}

#[test]
fn an_attachment_that_is_what_it_says_it_is_comes_home() {
    let one = machine("dev_a");
    let other = blank("dev_b");
    let shared = tempfile::tempdir().unwrap();
    let (_src, file) = {
        let dir = tempfile::tempdir().unwrap();
        let at = dir.path().join("contrato.pdf");
        std::fs::write(&at, b"what the person really attached").unwrap();
        (dir, at)
    };
    let kept =
        tisty_core::attach::keep(&file, &one.data, tisty_core::attach::COPIED_UP_TO).unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    carry(&other.data, &other.device, shared.path(), Way::Pull, &[]).unwrap();

    assert_eq!(
        std::fs::read(other.data.join(&kept.at)).unwrap(),
        b"what the person really attached",
        "an honest attachment was turned away"
    );
}

#[test]
fn what_the_folder_offers_that_is_not_shaped_like_an_attachment_stays_there() {
    let one = machine("dev_a");
    let shared = tempfile::tempdir().unwrap();
    let real = planted(shared.path(), "contrato.pdf", b"a real one");
    let under = real.split('/').nth(1).unwrap().to_string();
    let shelf = shared.path().join(HELD).join(&under);
    std::fs::write(shelf.join("factura.exe"), b"not yours").unwrap();
    std::fs::write(shelf.join("nota.command"), b"nor this").unwrap();
    let odd = shared.path().join(HELD).join("not-a-shelf");
    std::fs::create_dir_all(&odd).unwrap();
    std::fs::write(odd.join("mapa-91f2ab00.svg"), b"wrong shelf").unwrap();

    carry(&one.data, &one.device, shared.path(), Way::Pull, &[]).unwrap();

    let here = one.data.join(HELD).join(&under);
    assert!(one.data.join(&real).exists(), "it kept nothing");
    assert!(
        !here.join("factura.exe").exists(),
        "an executable was let in"
    );
    assert!(!here.join("nota.command").exists());
    assert!(!one.data.join(HELD).join("not-a-shelf").exists());
}

#[test]
fn a_document_deleted_here_stops_being_readable_in_the_shared_folder() {
    let one = machine("dev_a");
    let shared = tempfile::tempdir().unwrap();
    paper(&one, "dev_a-0001", "# Lo que dije");
    carry_papers(&one.data, shared.path(), &["dev_a-0001".into()]).unwrap();

    forget_paper(shared.path(), "dev_a-0001");

    assert!(
        !shared.path().join("docs").join("dev_a-0001.md").exists(),
        "the body stayed legible in someone else's cloud folder"
    );
}

#[test]
fn forgetting_a_paper_can_never_name_its_way_out_of_the_folder() {
    let shared = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(shared.path().join(PAPERS)).unwrap();
    let loot = shared.path().join("loot.md");
    std::fs::write(&loot, "no es tuyo").unwrap();

    forget_paper(shared.path(), "../loot");

    assert!(loot.exists(), "it deleted a file outside the folder");
}

#[test]
fn a_body_travels_even_when_only_one_direction_was_asked_for() {
    let one = machine("dev_a");
    let shared = tempfile::tempdir().unwrap();
    paper(&one, "dev_a-0001", "# Lo que dije");

    carry(
        &one.data,
        &one.device,
        shared.path(),
        Way::Push,
        &["dev_a-0001".to_string()],
    )
    .unwrap();

    assert_eq!(
        body(shared.path(), "dev_a-0001"),
        "# Lo que dije",
        "a document written here waited for a full round to leave"
    );
}

#[test]
fn nothing_moving_still_leaves_the_two_sides_on_common_ground() {
    let one = machine("dev_a");
    let shared = tempfile::tempdir().unwrap();
    let alive = vec!["dev_a-0001".to_string()];
    paper(&one, "dev_a-0001", "# Iguales");
    theirs(shared.path(), "dev_a-0001", "# Iguales");

    let done = carry_papers(&one.data, shared.path(), &alive).unwrap();
    assert_eq!(done.sent + done.brought, 0);

    paper(&one, "dev_a-0001", "# Iguales, y algo mas");
    let after = carry_papers(&one.data, shared.path(), &alive).unwrap();

    assert!(
        after.undecided.is_empty(),
        "agreeing was not written down, so the next edit looked like a quarrel"
    );
    assert_eq!(after.sent, 1);
}

#[test]
fn a_name_no_document_could_have_never_reaches_the_disk() {
    let one = machine("dev_a");
    let shared = tempfile::tempdir().unwrap();
    let loot = shared.path().join("loot.md");
    std::fs::write(&loot, "no es tuyo").unwrap();

    let done = carry_papers(
        &one.data,
        shared.path(),
        &["../loot".to_string(), "../../loot".to_string()],
    )
    .unwrap();

    assert_eq!(done.sent + done.brought, 0, "it walked out of the store");
    assert_eq!(std::fs::read_to_string(&loot).unwrap(), "no es tuyo");
    let said = std::fs::read_to_string(one.data.join("carried.json")).unwrap_or_default();
    assert!(
        !said.contains("loot"),
        "the ledger learned a name it must not know"
    );
}

#[test]
fn settling_a_name_no_document_could_have_is_refused() {
    let one = machine("dev_a");
    let shared = tempfile::tempdir().unwrap();

    let why = settle(&one.data, shared.path(), "../../loot", Keep::Theirs);

    assert!(why.is_err(), "it settled a document that cannot exist");
}

#[test]
fn a_document_written_here_lands_in_the_folder() {
    let one = machine("dev_a");
    let shared = tempfile::tempdir().unwrap();
    paper(&one, "dev_a-0001", "# Minuta\n\nlo que dije");

    let done = carry_papers(&one.data, shared.path(), &["dev_a-0001".into()]).unwrap();

    assert_eq!(done.sent, 1);
    assert_eq!(body(shared.path(), "dev_a-0001"), "# Minuta\n\nlo que dije");
}

#[test]
fn a_locked_document_is_not_written_over_by_a_round() {
    let one = machine("dev_a");
    let shared = tempfile::tempdir().unwrap();
    let alive = vec!["dev_a-0001".to_string()];
    let shut = alive.clone();
    paper(&one, "dev_a-0001", "# Minuta");
    carry_papers_holding(&one.data, shared.path(), &alive, &shut).unwrap();
    theirs(
        shared.path(),
        "dev_a-0001",
        "# Minuta

lo suyo",
    );

    let done = carry_papers_holding(&one.data, shared.path(), &alive, &shut).unwrap();

    assert_eq!(body(&one.data, "dev_a-0001"), "# Minuta");
    assert_eq!(done.brought, 0, "nothing was written over the lock");
    assert_eq!(done.undecided_ids(), vec!["dev_a-0001".to_string()]);
}

#[test]
fn a_locked_document_written_on_both_sides_waits_instead_of_joining() {
    let one = machine("dev_a");
    let shared = tempfile::tempdir().unwrap();
    let alive = at_odds(&one, shared.path());

    let done = carry_papers_holding(&one.data, shared.path(), &alive, &alive).unwrap();

    assert_eq!(
        body(&one.data, "dev_a-0001"),
        "# Minuta

lo mio"
    );
    assert!(done.joined.is_empty(), "a lock is not joined away");
    assert_eq!(done.undecided_ids(), vec!["dev_a-0001".to_string()]);
}

#[test]
fn a_log_that_will_not_project_carries_no_body_either_way() {
    let one = machine("dev_a");
    let shared = tempfile::tempdir().unwrap();
    paper(&one, "dev_a-0001", "# Mio");
    theirs(shared.path(), "dev_a-0001", "# Suyo");
    let mine = one.store.join(&one.device).join("active.tisty");
    std::fs::create_dir_all(mine.parent().unwrap()).unwrap();
    std::fs::write(
        &mine,
        "{\"v\":99,\"ts\":\"2026-01-01T00:00:00Z\"}
",
    )
    .unwrap();

    let done = carry(
        &one.data,
        &one.device,
        shared.path(),
        Way::Pull,
        &["dev_a-0001".to_string()],
    )
    .unwrap();

    assert_eq!(
        body(&one.data, "dev_a-0001"),
        "# Mio",
        "not knowing what is locked is not knowing what may be written"
    );
    assert_eq!(done.brought, 0);
    assert_eq!(done.astray, vec!["dev_a-0001".to_string()]);
    assert!(
        done.unprojected,
        "a log nobody could read was told as a document nobody could read"
    );
}

#[test]
fn a_locked_document_still_goes_out_to_the_folder() {
    let one = machine("dev_a");
    let shared = tempfile::tempdir().unwrap();
    paper(&one, "dev_a-0001", "# Lo que guardo");

    let done = carry_papers_holding(
        &one.data,
        shared.path(),
        &["dev_a-0001".into()],
        &["dev_a-0001".into()],
    )
    .unwrap();

    assert_eq!(done.sent, 1, "the protected copy is the one that travels");
    assert_eq!(body(shared.path(), "dev_a-0001"), "# Lo que guardo");
}

#[test]
fn a_document_only_the_folder_has_comes_home() {
    let one = machine("dev_a");
    let shared = tempfile::tempdir().unwrap();
    theirs(shared.path(), "dev_b-0001", "# Suya");

    let done = carry_papers(&one.data, shared.path(), &["dev_b-0001".into()]).unwrap();

    assert_eq!(done.brought, 1);
    assert_eq!(body(&one.data, "dev_b-0001"), "# Suya");
}

#[test]
fn only_the_side_that_changed_travels_the_second_time() {
    let one = machine("dev_a");
    let shared = tempfile::tempdir().unwrap();
    let alive = ["dev_a-0001".to_string()];
    paper(&one, "dev_a-0001", "# Minuta");
    carry_papers(&one.data, shared.path(), &alive).unwrap();

    theirs(shared.path(), "dev_a-0001", "# Minuta\n\ny algo mas");
    let done = carry_papers(&one.data, shared.path(), &alive).unwrap();

    assert_eq!(done.brought, 1, "what changed there did not come home");
    assert_eq!(done.sent, 0, "it pushed over what it had just been given");
    assert_eq!(body(&one.data, "dev_a-0001"), "# Minuta\n\ny algo mas");
}

#[test]
fn when_both_sides_moved_it_asks_instead_of_choosing() {
    let one = machine("dev_a");
    let shared = tempfile::tempdir().unwrap();
    let alive = ["dev_a-0001".to_string()];
    paper(&one, "dev_a-0001", "# Minuta");
    carry_papers(&one.data, shared.path(), &alive).unwrap();

    paper(&one, "dev_a-0001", "# Minuta\n\nlo mio");
    theirs(shared.path(), "dev_a-0001", "# Minuta\n\nlo suyo");
    let done = carry_papers(&one.data, shared.path(), &alive).unwrap();

    assert_eq!(done.undecided.len(), 1);
    assert_eq!(done.undecided[0].id, "dev_a-0001");
    assert_eq!(done.sent + done.brought, 0, "it moved something anyway");
    assert_eq!(body(&one.data, "dev_a-0001"), "# Minuta\n\nlo mio");
    assert_eq!(body(shared.path(), "dev_a-0001"), "# Minuta\n\nlo suyo");
}

#[test]
fn a_document_the_log_never_mentions_is_left_where_it_is() {
    let one = machine("dev_a");
    let shared = tempfile::tempdir().unwrap();
    theirs(shared.path(), "dev_b-0009", "# Nadie la nombra");

    let done = carry_papers(&one.data, shared.path(), &[]).unwrap();

    assert_eq!(done.brought, 0);
    assert!(!one.data.join("docs").join("dev_b-0009.md").exists());
}

#[test]
fn asking_twice_asks_twice_instead_of_deciding_the_second_time() {
    let one = machine("dev_a");
    let shared = tempfile::tempdir().unwrap();
    let alive = ["dev_a-0001".to_string()];
    paper(&one, "dev_a-0001", "# Minuta");
    carry_papers(&one.data, shared.path(), &alive).unwrap();
    paper(&one, "dev_a-0001", "# Minuta\n\nlo mio");
    theirs(shared.path(), "dev_a-0001", "# Minuta\n\nlo suyo");
    carry_papers(&one.data, shared.path(), &alive).unwrap();

    let done = carry_papers(&one.data, shared.path(), &alive).unwrap();

    assert_eq!(done.undecided.len(), 1, "it settled it on its own");
}

#[test]
fn settling_with_theirs_also_lets_the_next_edit_travel_alone() {
    let one = machine("dev_a");
    let shared = tempfile::tempdir().unwrap();
    let alive = at_odds(&one, shared.path());
    settle(&one.data, shared.path(), "dev_a-0001", Keep::Theirs).unwrap();

    paper(&one, "dev_a-0001", "# Minuta\n\nlo suyo, y algo mas");
    let done = carry_papers(&one.data, shared.path(), &alive).unwrap();

    assert!(
        done.undecided.is_empty(),
        "settling with theirs did not become the new common ground"
    );
    assert_eq!(done.sent, 1);
    assert_eq!(
        body(shared.path(), "dev_a-0001"),
        "# Minuta\n\nlo suyo, y algo mas"
    );
}

#[test]
fn settling_with_both_lets_a_later_local_edit_travel_alone() {
    let one = machine("dev_a");
    let shared = tempfile::tempdir().unwrap();
    let alive = at_odds(&one, shared.path());
    settle(&one.data, shared.path(), "dev_a-0001", Keep::Both).unwrap();
    settle(&one.data, shared.path(), "dev_a-0001", Keep::Mine).unwrap();

    paper(&one, "dev_a-0001", "# Minuta\n\nlo mio, y algo mas");
    let done = carry_papers(&one.data, shared.path(), &alive).unwrap();

    assert!(
        done.undecided.is_empty(),
        "settling both did not become the new common ground"
    );
    assert_eq!(done.sent, 1);
}

#[test]
fn settling_with_both_lets_a_later_remote_edit_travel_alone() {
    let one = machine("dev_a");
    let shared = tempfile::tempdir().unwrap();
    let alive = at_odds(&one, shared.path());
    settle(&one.data, shared.path(), "dev_a-0001", Keep::Both).unwrap();
    settle(&one.data, shared.path(), "dev_a-0001", Keep::Mine).unwrap();

    theirs(
        shared.path(),
        "dev_a-0001",
        "# Minuta\n\nlo mio, visto desde el otro lado",
    );
    let done = carry_papers(&one.data, shared.path(), &alive).unwrap();

    assert!(
        done.undecided.is_empty(),
        "settling both did not become the new common ground on the other side"
    );
    assert_eq!(done.brought, 1);
}

#[test]
fn four_alternating_synchronizations_converge_without_losing_a_single_edit() {
    let one = machine("dev_a");
    let shared = tempfile::tempdir().unwrap();
    let alive = ["dev_a-0001".to_string()];

    paper(&one, "dev_a-0001", "# Minuta\n\nversion uno");
    let first = carry_papers(&one.data, shared.path(), &alive).unwrap();
    assert_eq!(first.sent, 1, "the first version never left");

    theirs(shared.path(), "dev_a-0001", "# Minuta\n\nversion dos");
    let second = carry_papers(&one.data, shared.path(), &alive).unwrap();
    assert_eq!(second.brought, 1, "the second version did not come home");
    assert!(second.undecided.is_empty());
    assert_eq!(body(&one.data, "dev_a-0001"), "# Minuta\n\nversion dos");

    paper(&one, "dev_a-0001", "# Minuta\n\nversion tres");
    let third = carry_papers(&one.data, shared.path(), &alive).unwrap();
    assert_eq!(third.sent, 1, "the third version never left");
    assert!(third.undecided.is_empty());
    assert_eq!(
        body(shared.path(), "dev_a-0001"),
        "# Minuta\n\nversion tres"
    );

    theirs(shared.path(), "dev_a-0001", "# Minuta\n\nversion cuatro");
    let fourth = carry_papers(&one.data, shared.path(), &alive).unwrap();
    assert_eq!(fourth.brought, 1, "the fourth version did not come home");
    assert!(fourth.undecided.is_empty());

    assert_eq!(body(&one.data, "dev_a-0001"), "# Minuta\n\nversion cuatro");
    assert_eq!(
        body(shared.path(), "dev_a-0001"),
        "# Minuta\n\nversion cuatro"
    );
}

#[test]
fn a_body_missing_from_the_shared_folder_is_sent_back_not_left_absent() {
    let one = machine("dev_a");
    let shared = tempfile::tempdir().unwrap();
    let alive = ["dev_a-0001".to_string()];
    paper(&one, "dev_a-0001", "# Minuta\n\nlo que dije");
    carry_papers(&one.data, shared.path(), &alive).unwrap();

    std::fs::remove_file(shared.path().join("docs").join("dev_a-0001.md")).unwrap();
    let done = carry_papers(&one.data, shared.path(), &alive).unwrap();

    assert_eq!(
        done.sent, 1,
        "a body only the folder lost was not sent back"
    );
    assert_eq!(body(shared.path(), "dev_a-0001"), "# Minuta\n\nlo que dije");
}

#[test]
fn a_body_deleted_here_by_accident_comes_back_from_what_the_folder_still_has() {
    let one = machine("dev_a");
    let shared = tempfile::tempdir().unwrap();
    let alive = ["dev_a-0001".to_string()];
    paper(&one, "dev_a-0001", "# Minuta\n\nlo que dije");
    carry_papers(&one.data, shared.path(), &alive).unwrap();

    std::fs::remove_file(one.data.join("docs").join("dev_a-0001.md")).unwrap();
    let done = carry_papers(&one.data, shared.path(), &alive).unwrap();

    assert_eq!(
        done.brought, 1,
        "a body only lost here was not brought back"
    );
    assert_eq!(body(&one.data, "dev_a-0001"), "# Minuta\n\nlo que dije");
}

#[test]
fn a_body_gone_from_both_sides_is_not_forgotten_and_comes_back_clean_once_one_side_writes_again() {
    let one = machine("dev_a");
    let shared = tempfile::tempdir().unwrap();
    let alive = ["dev_a-0001".to_string()];
    paper(&one, "dev_a-0001", "# Minuta\n\nversion uno");
    carry_papers(&one.data, shared.path(), &alive).unwrap();

    std::fs::remove_file(one.data.join("docs").join("dev_a-0001.md")).unwrap();
    std::fs::remove_file(shared.path().join("docs").join("dev_a-0001.md")).unwrap();
    let vanished = carry_papers(&one.data, shared.path(), &alive).unwrap();
    assert_eq!(vanished.sent + vanished.brought, 0);
    assert!(vanished.undecided.is_empty());

    paper(&one, "dev_a-0001", "# Minuta\n\nvuelve distinta");
    let done = carry_papers(&one.data, shared.path(), &alive).unwrap();

    assert_eq!(
        done.sent, 1,
        "a body that came back was held against its old self"
    );
    assert!(done.undecided.is_empty());
    assert_eq!(
        body(shared.path(), "dev_a-0001"),
        "# Minuta\n\nvuelve distinta"
    );
}

#[test]
fn two_sides_that_reach_the_same_words_on_their_own_settle_without_being_asked() {
    let one = machine("dev_a");
    let shared = tempfile::tempdir().unwrap();
    let alive = at_odds(&one, shared.path());
    let conflicted = carry_papers(&one.data, shared.path(), &alive).unwrap();
    assert_eq!(conflicted.undecided.len(), 1);

    paper(&one, "dev_a-0001", "# Minuta\n\nlo mismo al fin");
    theirs(shared.path(), "dev_a-0001", "# Minuta\n\nlo mismo al fin");
    let done = carry_papers(&one.data, shared.path(), &alive).unwrap();

    assert!(
        done.undecided.is_empty(),
        "it kept asking after both sides said the same thing"
    );
    assert_eq!(
        done.sent + done.brought,
        0,
        "it moved something nobody had changed anywhere"
    );
}

#[test]
fn leaning_on_the_cache_still_sees_what_was_retired_a_moment_ago() {
    let one = machine("uno");
    let kept = planted(&one.data, "foto.png", b"una fotografia retirada");
    let shared = tempfile::tempdir().unwrap();
    let cache = tempfile::tempdir().unwrap();
    let at = cache.path().to_path_buf();

    carry_leaning_on(
        &one.data,
        Some(&at),
        &one.device,
        shared.path(),
        Way::Both,
        &[],
    )
    .unwrap();
    std::fs::remove_file(shared.path().join(&kept)).unwrap();

    says(&one, Op::AttachRetire { d: kept.clone() });
    carry_leaning_on(
        &one.data,
        Some(&at),
        &one.device,
        shared.path(),
        Way::Both,
        &[],
    )
    .unwrap();

    assert!(
        !shared.path().join(&kept).exists(),
        "la cache sirvio un estado viejo y lo retirado volvio a subir"
    );
}

#[test]
fn leaning_on_the_cache_lands_exactly_where_reading_it_all_lands() {
    let run = |aside: bool| -> (Vec<String>, usize) {
        let one = machine("dev_a");
        let two = blank("dev_b");
        let shared = tempfile::tempdir().unwrap();
        let cache = tempfile::tempdir().unwrap();
        let at = aside.then(|| cache.path().to_path_buf());

        carry_leaning_on(
            &one.data,
            at.as_deref(),
            &one.device,
            shared.path(),
            Way::Both,
            &[],
        )
        .unwrap();
        carry_leaning_on(
            &two.data,
            at.as_deref(),
            &two.device,
            shared.path(),
            Way::Both,
            &[],
        )
        .unwrap();

        for _ in 0..3 {
            carry_leaning_on(
                &one.data,
                at.as_deref(),
                &one.device,
                shared.path(),
                Way::Both,
                &[],
            )
            .unwrap();
            carry_leaning_on(
                &two.data,
                at.as_deref(),
                &two.device,
                shared.path(),
                Way::Both,
                &[],
            )
            .unwrap();
            wrote(&one, "algo mas de a".to_string());
            wrote(&two, "algo mas de b".to_string());
        }
        carry_leaning_on(
            &one.data,
            at.as_deref(),
            &one.device,
            shared.path(),
            Way::Both,
            &[],
        )
        .unwrap();

        let told = tisty_core::store::read_all(&one.store).unwrap();
        let state = tisty_core::State::replay(&told);
        let mut said: Vec<String> = state.tasks.values().map(|one| one.title.clone()).collect();
        said.sort();
        (said, told.len())
    };

    let plain = run(false);
    let leaned = run(true);

    assert_eq!(
        leaned, plain,
        "la cache llevo la sincronizacion a otro sitio"
    );
    assert!(plain.1 > 0, "el sorteo no llego a mover nada");
}

#[test]
fn after_a_decision_by_hand_the_base_on_disk_is_what_the_ledger_says_it_is() {
    let one = blank("dev_a");
    let shared = tempfile::tempdir().unwrap();
    let alive = ["dev_a-0001".to_string()];
    let base = "# Minuta\n\nparrafo uno\n\nparrafo dos\n";
    paper(&one, "dev_a-0001", base);
    carry_papers(&one.data, shared.path(), &alive).unwrap();

    paper(
        &one,
        "dev_a-0001",
        "# Minuta\n\nparrafo uno del mac\n\nparrafo dos\n",
    );
    theirs(
        shared.path(),
        "dev_a-0001",
        "# Minuta\n\nparrafo uno de windows\n\nparrafo dos\n",
    );
    let done = carry_papers(&one.data, shared.path(), &alive).unwrap();
    assert_eq!(done.undecided.len(), 1, "no llego a preguntar");

    settle(&one.data, shared.path(), "dev_a-0001", Keep::Mine).unwrap();
    carry_papers(&one.data, shared.path(), &alive).unwrap();
    carry_papers(&one.data, shared.path(), &alive).unwrap();

    let said = tisty_core::docs::Carried::read(&one.data);
    assert_eq!(
        tisty_core::docs::carried_print(&one.data, "dev_a-0001").as_deref(),
        said.of("dev_a-0001"),
        "la base en disco dejo de ser la que el ledger dice"
    );
}

#[test]
fn a_stale_base_left_by_a_decision_does_not_turn_a_silent_merge_into_a_question() {
    let one = blank("dev_a");
    let shared = tempfile::tempdir().unwrap();
    let alive = ["dev_a-0001".to_string()];
    paper(
        &one,
        "dev_a-0001",
        "# Minuta\n\nparrafo uno\n\nparrafo dos\n",
    );
    carry_papers(&one.data, shared.path(), &alive).unwrap();

    paper(
        &one,
        "dev_a-0001",
        "# Minuta\n\nparrafo uno del mac\n\nparrafo dos\n",
    );
    theirs(
        shared.path(),
        "dev_a-0001",
        "# Minuta\n\nparrafo uno de windows\n\nparrafo dos\n",
    );
    carry_papers(&one.data, shared.path(), &alive).unwrap();
    settle(&one.data, shared.path(), "dev_a-0001", Keep::Mine).unwrap();
    carry_papers(&one.data, shared.path(), &alive).unwrap();

    paper(
        &one,
        "dev_a-0001",
        "# Minuta del mac\n\nparrafo uno del mac\n\nparrafo dos\n",
    );
    theirs(
        shared.path(),
        "dev_a-0001",
        "# Minuta\n\nparrafo uno del mac\n\nparrafo dos\n\nparrafo tres\n",
    );
    let done = carry_papers(&one.data, shared.path(), &alive).unwrap();

    assert_eq!(
        done.undecided.len(),
        0,
        "volvio a preguntar algo que se juntaba solo"
    );
    assert_eq!(
        body(&one.data, "dev_a-0001"),
        "# Minuta del mac\n\nparrafo uno del mac\n\nparrafo dos\n\nparrafo tres\n"
    );
}

#[test]
fn the_attachment_ledger_never_lands_in_the_shared_folder() {
    let one = machine("dev_a");
    let shared = tempfile::tempdir().unwrap();
    planted(&one.data, "foto.png", b"una fotografia cualquiera");

    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

    assert!(
        !shared.path().join("attachments.jsonl").exists(),
        "el registro es local: en la carpeta compartida lo escriben todas y la nube saca copias de conflicto"
    );
    assert!(one.data.join("attachments.jsonl").exists());
}

#[test]
fn an_attachment_that_came_from_elsewhere_is_written_down_like_our_own() {
    let one = machine("uno");
    let two = blank("dos");
    let shared = tempfile::tempdir().unwrap();
    let kept = planted(&one.data, "foto.png", b"una fotografia que viaja");
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    carry(&two.data, &two.device, shared.path(), Way::Pull, &[]).unwrap();

    let written = tisty_core::attach::digests(&two.data);
    assert!(
        written.contains_key(&kept),
        "lo que llega de fuera queda sin huella larga, solo con la corta del nombre"
    );
}

#[test]
fn what_lands_in_the_shared_folder_is_dated_now_so_a_cloud_client_notices_it() {
    let one = machine("dev_a");
    let shared = tempfile::tempdir().unwrap();
    let old = std::time::SystemTime::now() - std::time::Duration::from_secs(60 * 60 * 24 * 3);
    let mine = one.store.join(&one.device).join("active.tisty");
    std::fs::File::options()
        .write(true)
        .open(&mine)
        .unwrap()
        .set_modified(old)
        .unwrap();

    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    let landed = shared
        .path()
        .join(STORE)
        .join(&one.device)
        .join("active.tisty");
    let when = std::fs::metadata(&landed)
        .and_then(|m| m.modified())
        .unwrap();
    let apart = std::time::SystemTime::now().duration_since(when).unwrap();
    assert!(
        apart < std::time::Duration::from_secs(60),
        "aterrizo con fecha de hace {apart:?}: un cliente de nube no lo ve como cambio"
    );
}

#[test]
fn a_round_that_changed_nothing_still_does_not_copy_again() {
    let one = machine("dev_a");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();
    let landed = shared
        .path()
        .join(STORE)
        .join(&one.device)
        .join("active.tisty");
    let was = std::fs::metadata(&landed)
        .and_then(|m| m.modified())
        .unwrap();

    std::thread::sleep(std::time::Duration::from_millis(20));
    let done = carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    assert_eq!(done.sent, 0, "volvio a copiar lo que no habia cambiado");
    assert_eq!(
        std::fs::metadata(&landed)
            .and_then(|m| m.modified())
            .unwrap(),
        was,
        "lo reescribio sin motivo"
    );
}

#[test]
fn asking_again_writes_what_a_plain_round_leaves_alone() {
    let one = machine("dev_a");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();
    let landed = shared
        .path()
        .join(STORE)
        .join(&one.device)
        .join("active.tisty");
    let stuck = std::time::SystemTime::now() - std::time::Duration::from_secs(60 * 60 * 24 * 3);
    std::fs::File::options()
        .write(true)
        .open(&landed)
        .unwrap()
        .set_modified(stuck)
        .unwrap();

    let plain = carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();
    assert_eq!(plain.sent, 0);
    assert_eq!(
        std::fs::metadata(&landed)
            .and_then(|m| m.modified())
            .unwrap(),
        stuck,
        "una ronda normal deberia dejar quieto lo que ya tiene los mismos bytes"
    );

    let forced = carry(&one.data, &one.device, shared.path(), Way::Again, &[]).unwrap();

    assert_eq!(forced.sent, 1, "pedirlo otra vez no reenvio el segmento");
    let when = std::fs::metadata(&landed)
        .and_then(|m| m.modified())
        .unwrap();
    let apart = std::time::SystemTime::now().duration_since(when).unwrap();
    assert!(
        apart < std::time::Duration::from_secs(60),
        "aterrizo con fecha de hace {apart:?}: sigue atascado"
    );
}

#[test]
fn two_bodies_of_one_size_but_different_content_are_not_taken_for_equal() {
    let dir = tempfile::tempdir().unwrap();
    let a = dir.path().join("a");
    let b = dir.path().join("b");
    let body = b"una linea igual de larga\n";
    let other = b"otra linea igual de larga";
    assert_eq!(body.len(), other.len());
    std::fs::write(&a, body).unwrap();
    std::fs::write(&b, other).unwrap();

    assert!(!same(&a, &b), "la fecha decidia, no el contenido");
    std::fs::write(&b, body).unwrap();
    assert!(same(&a, &b));
}

#[test]
fn two_bodies_of_one_size_that_differ_before_the_tail_are_not_taken_for_equal() {
    let dir = tempfile::tempdir().unwrap();
    let a = dir.path().join("a");
    let b = dir.path().join("b");
    let mut body = vec![b'x'; 4096];
    std::fs::write(&a, &body).unwrap();
    body[0] = b'z';
    std::fs::write(&b, &body).unwrap();

    assert!(!same(&a, &b));
}

#[test]
fn a_difference_inside_the_tail_is_seen_even_in_a_body_far_longer_than_the_window() {
    let dir = tempfile::tempdir().unwrap();
    let a = dir.path().join("a");
    let b = dir.path().join("b");
    let mut body = vec![b'x'; 2000];
    std::fs::write(&a, &body).unwrap();
    body[1900] = b'z';
    std::fs::write(&b, &body).unwrap();

    assert_eq!(std::fs::metadata(&a).unwrap().len(), 2000);
    assert!(
        !same(&a, &b),
        "difieren a 100 bytes del final y pasaron por iguales: la ventana es corta o mira al principio"
    );
}

#[test]
fn the_window_reads_the_end_of_the_file_and_not_the_beginning() {
    let dir = tempfile::tempdir().unwrap();
    let a = dir.path().join("a");
    let b = dir.path().join("b");
    let mut body = vec![b'x'; 2000];
    body[0] = b'a';
    std::fs::write(&a, &body).unwrap();
    body[1999] = b'z';
    std::fs::write(&b, &body).unwrap();

    assert!(
        !same(&a, &b),
        "solo se distinguen por el ultimo byte, asi que leer el principio los daria por iguales"
    );
}

#[test]
fn a_round_that_moved_nothing_leaves_the_marker_untouched() {
    let one = machine("dev_a");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();
    let at = shared.path().join(STORE).join(MARKER);
    let was = std::fs::metadata(&at).and_then(|m| m.modified()).unwrap();

    std::thread::sleep(std::time::Duration::from_millis(20));
    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

    let now = std::fs::metadata(&at).and_then(|m| m.modified()).unwrap();
    assert_eq!(now, was, "la carpeta de nube ve un cambio donde no lo hay");
}

#[test]
fn a_round_where_nothing_moved_writes_nothing_at_all() {
    let one = blank("dev_a");
    let shared = tempfile::tempdir().unwrap();
    let alive = ["dev_a-0001".to_string()];
    paper(&one, "dev_a-0001", "# Notas\n\nun cuerpo cualquiera");

    carry_papers(&one.data, shared.path(), &alive).unwrap();

    let watched = [
        one.data.join("carried").join("dev_a-0001.md"),
        one.data.join("carried.json"),
    ];
    let before: Vec<_> = watched
        .iter()
        .map(|at| std::fs::metadata(at).and_then(|m| m.modified()).unwrap())
        .collect();

    std::thread::sleep(std::time::Duration::from_millis(20));
    let still = carry_papers(&one.data, shared.path(), &alive).unwrap();

    assert_eq!(still.sent + still.brought, 0);
    for (at, was) in watched.iter().zip(before) {
        let now = std::fs::metadata(at).and_then(|m| m.modified()).unwrap();
        assert_eq!(now, was, "se reescribio sin que nada cambiara: {at:?}");
    }
}

#[test]
fn a_base_wiped_from_under_us_is_written_again_on_the_next_round() {
    let one = blank("dev_a");
    let shared = tempfile::tempdir().unwrap();
    let alive = ["dev_a-0001".to_string()];
    paper(&one, "dev_a-0001", "# Notas\n\nun cuerpo cualquiera");

    carry_papers(&one.data, shared.path(), &alive).unwrap();
    let at = one.data.join("carried").join("dev_a-0001.md");
    std::fs::remove_file(&at).unwrap();

    carry_papers(&one.data, shared.path(), &alive).unwrap();

    assert!(at.exists(), "la base para fusionar quedo perdida");
}

#[test]
fn a_body_kept_beside_a_document_is_no_ancestor_unless_the_ledger_names_it() {
    let one = blank("dev_a");
    let shared = tempfile::tempdir().unwrap();
    let alive = ["dev_a-0001".to_string()];
    paper(&one, "dev_a-0001", "# Minuta\n\nlo que los dos vieron\n");

    carry_papers(&one.data, shared.path(), &alive).unwrap();
    std::fs::write(
        one.data.join("carried").join("dev_a-0001.md"),
        "# Minuta\n\nuno\n\ndos\n",
    )
    .unwrap();
    let mine = "# Minuta\n\nuno-mio\n\ndos\n";
    paper(&one, "dev_a-0001", mine);
    theirs(shared.path(), "dev_a-0001", "# Minuta\n\nuno\n\ndos-suyo\n");

    let done = carry_papers(&one.data, shared.path(), &alive).unwrap();

    assert!(
        done.joined.is_empty(),
        "a body nobody ever settled on was woven in as what the two sides came from"
    );
    assert_eq!(done.undecided_ids(), alive);
    assert_eq!(body(&one.data, "dev_a-0001"), mine);
}

#[test]
fn a_base_written_before_the_folder_was_named_still_stands_after_an_update_in_place() {
    let one = machine("dev_a");
    let kept = tempfile::tempdir().unwrap();
    let aside = Some(kept.path());
    let shared = tempfile::tempdir().unwrap();
    filed(
        &one,
        "dev_a-0001",
        "# Minuta

lo que los dos vieron
",
    );
    carry_leaning_on(&one.data, aside, &one.device, shared.path(), Way::Both, &[]).unwrap();

    let print = tisty_core::docs::carried_print(&one.data, "dev_a-0001").expect("a base");
    std::fs::write(
        one.data.join("carried.json"),
        format!("{{\"dev_a-0001\":\"{print}\"}}"),
    )
    .unwrap();
    tisty_core::docs::write(
        &one.data.join(PAPERS),
        "dev_a-0001",
        "# Minuta

lo que los dos vieron
lo mio
",
    )
    .unwrap();

    let done =
        carry_leaning_on(&one.data, aside, &one.device, shared.path(), Way::Both, &[]).unwrap();

    assert!(
        done.undecided.is_empty(),
        "updating in place was read as another folder, so it asked about a document only this side touched"
    );
    assert_eq!(done.sent, 1);
    assert_eq!(
        body(shared.path(), "dev_a-0001"),
        "# Minuta

lo que los dos vieron
lo mio
"
    );
}

#[test]
fn a_base_from_one_folder_is_not_leaned_on_in_another() {
    let one = blank("dev_a");
    let first = tempfile::tempdir().unwrap();
    let second = tempfile::tempdir().unwrap();
    let alive = ["dev_a-0001".to_string()];
    let mine = "# Minuta\n\nlo que la primera carpeta vio\n";
    paper(&one, "dev_a-0001", mine);

    carry_papers(&one.data, first.path(), &alive).unwrap();
    theirs(
        second.path(),
        "dev_a-0001",
        "# Minuta\n\notra rama entera\n",
    );

    let done = carry_papers(&one.data, second.path(), &alive).unwrap();

    assert_eq!(
        body(&one.data, "dev_a-0001"),
        mine,
        "a folder we had never compared against was taken for the newer side"
    );
    assert_eq!(done.brought, 0);
    assert_eq!(done.undecided_ids(), alive);
    assert!(
        !one.data.join("carried").join("dev_a-0001.md").exists(),
        "the body the first folder settled on stayed as the ancestor of a history it never had"
    );
}

#[test]
fn taking_up_another_folder_asks_only_about_what_the_two_sides_do_not_already_hold_alike() {
    let one = blank("dev_a");
    let first = tempfile::tempdir().unwrap();
    let second = tempfile::tempdir().unwrap();
    let alive = ["dev_a-0001".to_string(), "dev_a-0002".to_string()];
    paper(&one, "dev_a-0001", "# Igual\n");
    paper(&one, "dev_a-0002", "# Distinto\n");

    carry_papers(&one.data, first.path(), &alive).unwrap();
    theirs(second.path(), "dev_a-0001", "# Igual\n");
    theirs(second.path(), "dev_a-0002", "# Distinto\n\notra rama\n");

    let done = carry_papers(&one.data, second.path(), &alive).unwrap();

    assert_eq!(
        done.undecided_ids(),
        ["dev_a-0002".to_string()],
        "losing the base turned a folder that agrees into a folder that argues"
    );
    assert_eq!(body(&one.data, "dev_a-0001"), "# Igual\n");
    assert_eq!(
        tisty_core::docs::carried_print(&one.data, "dev_a-0001"),
        tisty_core::docs::print_of(&one.data.join("docs").join("dev_a-0001.md")).unwrap(),
        "the base for the folder it faces now was not written"
    );
}

#[test]
fn content_no_editor_would_be_proud_of_still_crosses_byte_for_byte() {
    let long = "y".repeat(400 * 1024);
    let cases: [(&str, &str); 6] = [
        ("empty", ""),
        ("blank", "   \n\t  \n  "),
        ("long, but still within what the editor will open", &long),
        ("accented", "café ñandú 日本語 🎉 texto con acentós"),
        ("windows line endings", "una linea\r\notra linea\r\n"),
        ("a byte order mark up front", "\u{FEFF}# Titulo\n\ncuerpo"),
    ];

    for (label, first) in cases {
        let one = blank("dev_a");
        let shared = tempfile::tempdir().unwrap();
        let alive = ["dev_a-0001".to_string()];
        paper(&one, "dev_a-0001", first);

        let sent = carry_papers(&one.data, shared.path(), &alive).unwrap();
        assert_eq!(sent.sent, 1, "did not travel on: {label}");
        assert_eq!(
            body(shared.path(), "dev_a-0001"),
            first,
            "bytes changed in transit on: {label}"
        );

        let still = carry_papers(&one.data, shared.path(), &alive).unwrap();
        assert_eq!(
            still.sent + still.brought,
            0,
            "moved again with nothing changed on: {label}"
        );

        let edited = format!("{first}\nmas");
        theirs(shared.path(), "dev_a-0001", &edited);
        let round = carry_papers(&one.data, shared.path(), &alive).unwrap();
        assert_eq!(
            round.brought, 1,
            "the edit on top did not come home on: {label}"
        );
        assert_eq!(
            body(&one.data, "dev_a-0001"),
            edited,
            "bytes changed coming home on: {label}"
        );
    }
}

#[test]
fn a_machine_nobody_ever_named_is_not_locked_out_by_someone_elses_list() {
    let one = machine("dev_a");
    says(
        &one,
        Op::DeviceJoin {
            d: DeviceId("dev_b".into()),
            k: Some(tisty_core::DeviceKind::Machine),
            p: None,
        },
    );
    let shared = tempfile::tempdir().unwrap();

    let moved = carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

    assert!(
        moved.sent > 0,
        "the first machine to join shut the door on the rest"
    );
}

#[test]
fn being_named_once_and_dropped_is_not_the_same_as_never_being_named() {
    let said = tisty_core::store::Ledger {
        allowed: [DeviceId("dev_b".into())].into(),
        named: [DeviceId("dev_a".into()), DeviceId("dev_b".into())].into(),
        keys: Default::default(),
        vouched: Default::default(),
    };

    assert!(!said.may_write(&DeviceId("dev_a".into())));
    assert!(said.may_write(&DeviceId("dev_b".into())));
    assert!(said.may_write(&DeviceId("dev_c".into())));
    assert!(said.was_removed(&DeviceId("dev_a".into())));
    assert!(!said.was_removed(&DeviceId("dev_c".into())));
}

#[test]
fn two_machines_removing_each_other_at_once_do_not_brick_the_store() {
    let said = tisty_core::store::Ledger {
        allowed: Default::default(),
        named: [DeviceId("dev_a".into()), DeviceId("dev_b".into())].into(),
        keys: Default::default(),
        vouched: Default::default(),
    };

    assert!(
        said.may_write(&DeviceId("dev_a".into())),
        "nobody could ever write here again"
    );
    assert!(said.may_write(&DeviceId("dev_b".into())));
}

#[test]
fn a_machine_that_was_removed_writes_nothing_at_all() {
    let one = machine("dev_a");
    says(
        &one,
        Op::DeviceJoin {
            d: DeviceId("dev_a".into()),
            k: Some(tisty_core::DeviceKind::Machine),
            p: None,
        },
    );
    says(
        &one,
        Op::DeviceJoin {
            d: DeviceId("dev_b".into()),
            k: Some(tisty_core::DeviceKind::Machine),
            p: None,
        },
    );
    says(
        &one,
        Op::DeviceRemove {
            d: DeviceId("dev_a".into()),
        },
    );
    let shared = tempfile::tempdir().unwrap();

    let why = carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap_err();

    assert!(
        matches!(why, Trouble::NotAllowed(_)),
        "it went through: {why:?}"
    );
    assert!(
        !shared.path().join(STORE).join("dev_a").exists(),
        "it wrote before refusing"
    );
}

#[test]
fn a_machine_that_was_removed_still_brings_what_is_there() {
    let other = machine("dev_b");
    let shared = tempfile::tempdir().unwrap();
    carry(&other.data, &other.device, shared.path(), Way::Push, &[]).unwrap();

    let one = blank("dev_a");
    joined(&one, shared.path());
    says(
        &one,
        Op::DeviceJoin {
            d: DeviceId("dev_b".into()),
            k: Some(tisty_core::DeviceKind::Machine),
            p: None,
        },
    );
    says(
        &one,
        Op::DeviceRemove {
            d: DeviceId("dev_a".into()),
        },
    );

    wrote(&other, "lo dicho despues".into());
    carry(&other.data, &other.device, shared.path(), Way::Push, &[]).unwrap();
    let moved = carry(&one.data, &one.device, shared.path(), Way::Pull, &[]).unwrap();

    assert!(moved.brought > 0, "being removed is not being cut off");
    assert!(
        titles(&one.store).contains(&"lo dicho despues".to_string()),
        "{:?}",
        titles(&one.store)
    );
}

#[test]
fn the_word_that_removes_it_is_read_before_it_writes() {
    let other = machine("dev_b");
    let shared = tempfile::tempdir().unwrap();
    carry(&other.data, &other.device, shared.path(), Way::Push, &[]).unwrap();

    let one = blank("dev_a");
    joined(&one, shared.path());

    says(
        &other,
        Op::DeviceJoin {
            d: DeviceId("dev_b".into()),
            k: Some(tisty_core::DeviceKind::Machine),
            p: None,
        },
    );
    says(
        &other,
        Op::DeviceRemove {
            d: DeviceId("dev_a".into()),
        },
    );
    carry(&other.data, &other.device, shared.path(), Way::Push, &[]).unwrap();

    let why = carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap_err();

    assert!(
        matches!(why, Trouble::NotAllowed(_)),
        "it pushed on stale news: {why:?}"
    );
}

#[test]
fn what_one_machine_leaves_the_other_takes_home() {
    let one = machine("dev_a");
    let other = blank("dev_b");
    let shared = tempfile::tempdir().unwrap();

    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();
    joined(&other, shared.path());
    wrote(&other, "lo de dev_b".into());
    carry(&other.data, &other.device, shared.path(), Way::Both, &[]).unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

    let mine = titles(&one.store);
    assert!(mine.contains(&"lo de dev_a".to_string()), "{mine:?}");
    assert!(mine.contains(&"lo de dev_b".to_string()), "{mine:?}");
    assert_eq!(titles(&other.store).len(), 2);
}

#[test]
fn nobody_ever_writes_over_their_own_directory() {
    let one = machine("dev_a");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

    std::fs::write(shared.path().join("store/dev_a/active.tisty"), b"").unwrap();

    let _ = carry(&one.data, &one.device, shared.path(), Way::Pull, &[]);
    assert_eq!(titles(&one.store).len(), 1, "the emptied copy came home");
}

#[test]
fn a_directory_that_differs_only_in_case_is_still_our_own() {
    let one = machine("dev_a");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    let theirs = shared.path().join("store/DEV_A");
    std::fs::create_dir_all(&theirs).unwrap();
    std::fs::write(theirs.join("active.tisty"), b"").unwrap();

    let _ = carry(&one.data, &one.device, shared.path(), Way::Pull, &[]);
    assert_eq!(titles(&one.store).len(), 1, "our own log was overwritten");
}

#[test]
fn what_is_left_behind_is_never_removed() {
    let one = machine("dev_a");
    let shared = tempfile::tempdir().unwrap();
    let stranger = shared.path().join("store/dev_z");
    std::fs::create_dir_all(&stranger).unwrap();
    std::fs::write(stranger.join("keep.txt"), b"not ours").unwrap();

    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();
    assert!(stranger.join("keep.txt").exists());
}

#[test]
fn a_folder_of_another_store_is_refused_before_anything_moves() {
    let one = machine("dev_a");
    std::fs::write(one.store.join(MARKER), b"01OURS00000000000000000000").unwrap();
    let shared = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(shared.path().join("store")).unwrap();
    std::fs::write(
        shared.path().join("store").join(MARKER),
        b"01THEIRS000000000000000000",
    )
    .unwrap();

    let Err(Trouble::OtherStore { theirs }) =
        carry(&one.data, &one.device, shared.path(), Way::Both, &[])
    else {
        panic!("two histories were about to be merged");
    };
    assert_eq!(theirs, "01THEIRS000000000000000000");
    assert!(
        !shared.path().join("store/dev_a").exists(),
        "something moved"
    );
}

#[test]
fn two_histories_are_never_joined_at_all() {
    let one = machine("dev_a");
    std::fs::remove_file(one.store.join(MARKER)).ok();
    let other = machine("dev_b");
    let shared = tempfile::tempdir().unwrap();
    carry(&other.data, &other.device, shared.path(), Way::Push, &[]).unwrap();

    let Err(Trouble::WouldReset { .. }) =
        carry(&one.data, &one.device, shared.path(), Way::Both, &[])
    else {
        panic!("two histories were joined");
    };

    assert_eq!(titles(&one.store), vec!["lo de dev_a".to_string()]);
    assert!(
        !shared.path().join("store/dev_a").exists(),
        "something moved"
    );

    let again = carry(&one.data, &one.device, shared.path(), Way::Both, &[]);
    assert!(
        matches!(again, Err(Trouble::WouldReset { .. })),
        "asking twice is not consent: {again:?}"
    );
    assert_eq!(titles(&one.store).len(), 1, "it joined them anyway");
}

#[test]
fn a_store_with_history_and_no_marker_is_not_adopted() {
    let one = machine("dev_a");
    std::fs::remove_file(one.store.join(MARKER)).ok();
    let other = machine("dev_b");
    let shared = tempfile::tempdir().unwrap();
    carry(&other.data, &other.device, shared.path(), Way::Push, &[]).unwrap();

    let Err(Trouble::WouldReset { .. }) =
        carry(&one.data, &one.device, shared.path(), Way::Both, &[])
    else {
        panic!("an unmarked store merged into a stranger's history");
    };
    assert_eq!(titles(&one.store), vec!["lo de dev_a".to_string()]);
}

#[test]
fn a_folder_full_of_history_with_no_marker_is_refused() {
    let one = machine("dev_a");
    let other = machine("dev_b");
    let shared = tempfile::tempdir().unwrap();
    carry(&other.data, &other.device, shared.path(), Way::Push, &[]).unwrap();
    std::fs::remove_file(shared.path().join("store").join(MARKER)).unwrap();

    let Err(Trouble::WouldReset { .. }) =
        carry(&one.data, &one.device, shared.path(), Way::Both, &[])
    else {
        panic!("a folder with history and no marker was treated as empty");
    };
    assert_eq!(titles(&one.store), vec!["lo de dev_a".to_string()]);
}

#[test]
fn a_meeting_place_that_is_not_there_says_so() {
    let one = machine("dev_a");
    let gone = one.store.join("unplugged");

    assert!(matches!(
        carry(&one.data, &one.device, &gone, Way::Both, &[]),
        Err(Trouble::NotThere(_))
    ));
}

#[test]
fn one_direction_only_does_one_direction() {
    let one = blank("dev_a");
    let other = machine("dev_b");
    let shared = tempfile::tempdir().unwrap();
    carry(&other.data, &other.device, shared.path(), Way::Push, &[]).unwrap();

    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();
    assert!(
        titles(&one.store).is_empty(),
        "a push brought something back"
    );

    carry(&one.data, &one.device, shared.path(), Way::Pull, &[]).unwrap();
    assert_eq!(titles(&one.store).len(), 1);
}

#[test]
fn a_machine_meeting_the_folder_for_the_first_time_adopts_its_name() {
    let one = machine("dev_a");
    let other = machine("dev_b");
    std::fs::remove_file(other.store.join(MARKER)).ok();
    let shared = tempfile::tempdir().unwrap();

    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();
    std::fs::remove_dir_all(other.store.join(&other.device)).unwrap();
    carry(&other.data, &other.device, shared.path(), Way::Both, &[]).unwrap();

    assert_eq!(
        tisty_core::store::peek_identity(&other.store),
        tisty_core::store::peek_identity(&one.store)
    );
}

#[test]
fn syncing_twice_over_carries_nothing_the_second_time() {
    let one = machine("dev_a");
    let shared = tempfile::tempdir().unwrap();

    let first = carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();
    let again = carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    assert!(first.sent > 0);
    assert_eq!(again.sent, 0, "it copied what was already identical");
}

#[test]
fn a_gap_in_what_is_offered_is_refused_without_importing_it() {
    let one = blank("dev_a");
    let shared = tempfile::tempdir().unwrap();
    let theirs = shared.path().join("store/dev_b");
    std::fs::create_dir_all(&theirs).unwrap();
    std::fs::write(theirs.join("000002.tisty"), b"").unwrap();
    std::fs::write(
        shared.path().join("store").join(MARKER),
        b"01M0THEIRSTORE000000000000",
    )
    .unwrap();

    let moved = carry(&one.data, &one.device, shared.path(), Way::Pull, &[]).unwrap();

    assert_eq!(moved.unreadable, vec!["dev_b".to_string()]);
    assert!(!one.store.join("dev_b").exists(), "it landed anyway");
    assert!(titles(&one.store).is_empty(), "our own store still reads");
}

#[test]
fn one_unreadable_machine_in_the_folder_never_stops_our_own_work_going_out() {
    let one = machine("dev_a");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();
    let theirs = shared.path().join("store/dev_b");
    std::fs::create_dir_all(&theirs).unwrap();
    std::fs::write(theirs.join("000002.tisty"), b"").unwrap();

    wrote(&one, "lo que tiene que salir igual".into());
    let moved = carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

    assert_eq!(moved.unreadable, vec!["dev_b".to_string()]);
    assert!(moved.sent > 0, "lo nuestro se quedo sin salir");
    assert_eq!(
        tisty_core::store::distinct_in(&shared.path().join(STORE).join(&one.device)).unwrap(),
        2,
        "la carpeta no recibio lo que escribimos"
    );
}

#[test]
fn a_machine_writing_a_newer_schema_stops_the_sync_instead_of_carrying_half() {
    let one = machine("dev_a");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();
    let before =
        tisty_core::store::distinct_in(&shared.path().join(STORE).join(&one.device)).unwrap();

    let theirs = shared.path().join("store/dev_b");
    std::fs::create_dir_all(&theirs).unwrap();
    std::fs::write(
        theirs.join("active.tisty"),
        b"{\"v\":99,\"ts\":\"2026-08-26T10:00:00Z\",\"by\":\"dev_b\",\"op\":\"task.add\",\"id\":\"01M0ZX62YMRXMABJ6Q4FEF69WT\",\"d\":{\"title\":\"from the future\",\"order\":\"V\"}}
",
    )
    .unwrap();

    wrote(&one, "esto no debe salir a medias".into());
    let stopped = carry(&one.data, &one.device, shared.path(), Way::Both, &[]);

    assert!(
        matches!(stopped, Err(Trouble::Newer(ref who)) if who == "dev_b"),
        "a newer schema has to stop the sync and name the machine: {stopped:?}"
    );
    assert_eq!(
        tisty_core::store::distinct_in(&shared.path().join(STORE).join(&one.device)).unwrap(),
        before,
        "nothing of ours may go out while we cannot read theirs"
    );
}

#[test]
fn a_push_is_stopped_by_a_newer_schema_too_instead_of_writing_first_and_asking_later() {
    let one = machine("dev_a");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();
    let before =
        tisty_core::store::distinct_in(&shared.path().join(STORE).join(&one.device)).unwrap();

    let theirs = shared.path().join("store/dev_b");
    std::fs::create_dir_all(&theirs).unwrap();
    std::fs::write(
        theirs.join("active.tisty"),
        b"{\"v\":99,\"ts\":\"2026-08-26T10:00:00Z\",\"by\":\"dev_b\",\"op\":\"task.add\",\"id\":\"01M0ZX62YMRXMABJ6Q4FEF69WT\",\"d\":{\"title\":\"from the future\",\"order\":\"V\"}}
",
    )
    .unwrap();

    wrote(&one, "lo que escribo mientras no puedo leerles".into());
    let stopped = carry(&one.data, &one.device, shared.path(), Way::Push, &[]);

    assert!(
        matches!(stopped, Err(Trouble::Newer(ref who)) if who == "dev_b"),
        "a push is what every edit fires, and it asked nothing: {stopped:?}"
    );
    assert_eq!(
        tisty_core::store::distinct_in(&shared.path().join(STORE).join(&one.device)).unwrap(),
        before,
        "it wrote our history beside a machine we cannot read"
    );
}

#[test]
fn what_proves_a_store_is_its_own_never_reaches_the_meeting_place() {
    fn every_file(at: &Path, found: &mut Vec<PathBuf>) {
        let Ok(entries) = std::fs::read_dir(at) else {
            return;
        };
        for one in entries.filter_map(|e| e.ok()) {
            let at = one.path();
            match at.is_dir() {
                true => every_file(&at, found),
                false => found.push(at),
            }
        }
    }

    let one = machine("dev_a");
    std::fs::write(one.store.join(tisty_core::store::KEEP), [5u8; 32]).unwrap();
    let shared = tempfile::tempdir().unwrap();

    for way in [Way::Push, Way::Pull, Way::Both, Way::Again] {
        carry(&one.data, &one.device, shared.path(), way, &[]).unwrap();
    }

    let mut found = Vec::new();
    every_file(shared.path(), &mut found);
    assert!(
        !found.iter().any(|at| at.ends_with(tisty_core::store::KEEP)),
        "the key reached the meeting place: {found:?}"
    );
}

#[test]
fn a_conflict_copy_is_not_a_segment() {
    let one = machine("dev_a");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    let mine = shared.path().join("store/dev_a");
    let held = std::fs::read(mine.join("active.tisty")).unwrap();
    std::fs::write(mine.join("active (conflicted copy).tisty"), &held).unwrap();

    let other = blank("dev_b");
    carry(&other.data, &other.device, shared.path(), Way::Pull, &[]).unwrap();

    assert_eq!(
        titles(&other.store).len(),
        1,
        "the conflict copy was read as history"
    );
}

#[test]
fn a_pull_with_nothing_to_bring_does_not_reread_everything() {
    let one = blank("dev_a");
    let other = machine("dev_b");
    let shared = tempfile::tempdir().unwrap();
    carry(&other.data, &other.device, shared.path(), Way::Push, &[]).unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Pull, &[]).unwrap();

    std::fs::write(
        shared.path().join("store/dev_b/000001.tisty"),
        b"not json at all",
    )
    .unwrap();
    std::fs::write(shared.path().join("store/dev_b/000001.count"), b"1").unwrap();

    let again = carry(&one.data, &one.device, shared.path(), Way::Pull, &[]).unwrap();
    assert_eq!(again.unreadable, vec!["dev_b".to_string()]);
    assert_eq!(titles(&one.store).len(), 1, "our store still reads");
}

fn hostile_ids() -> Vec<String> {
    vec![
        "../secret".to_string(),
        "../../secret".to_string(),
        "..\\secret".to_string(),
        "/etc/passwd".to_string(),
        "C:\\Windows\\System32\\loot".to_string(),
        "c:loot".to_string(),
        "\\\\server\\share\\loot".to_string(),
        "CON".to_string(),
        "a3f1\0-0001".to_string(),
        "a3f1-0001 ".to_string(),
        "a".repeat(300),
        "a".repeat(5000),
        "dispositivo_caf\u{e9}-0001".to_string(),
        "dispositivo_cafe\u{301}-0001".to_string(),
        "\u{202e}evil-0001".to_string(),
        String::new(),
    ]
}

#[test]
fn a_hostile_identifier_in_the_alive_list_never_reaches_a_file_outside_docs() {
    let one = machine("dev_a");
    let shared = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(one.data.join(PAPERS)).unwrap();
    std::fs::create_dir_all(shared.path().join(PAPERS)).unwrap();
    let local_bystander = one.data.join("secret.md");
    std::fs::write(&local_bystander, "no es tuyo").unwrap();
    let shared_bystander = shared.path().join("secret.md");
    std::fs::write(&shared_bystander, "tampoco es tuyo").unwrap();

    let attacks = hostile_ids();
    let done = carry_papers(&one.data, shared.path(), &attacks).unwrap();

    assert_eq!(
        done.sent + done.brought,
        0,
        "a hostile identifier moved something"
    );
    assert_eq!(
        std::fs::read_to_string(&local_bystander).unwrap(),
        "no es tuyo"
    );
    assert_eq!(
        std::fs::read_to_string(&shared_bystander).unwrap(),
        "tampoco es tuyo"
    );
    let said = std::fs::read_to_string(one.data.join("carried.json")).unwrap_or_default();
    assert!(
        !said.contains("secret"),
        "the ledger learned a name it must not know"
    );
}

#[test]
fn settling_any_hostile_identifier_is_always_refused_before_anything_moves() {
    let one = machine("dev_a");
    let shared = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(one.data.join(PAPERS)).unwrap();
    std::fs::create_dir_all(shared.path().join(PAPERS)).unwrap();

    for id in hostile_ids() {
        assert!(
            settle(&one.data, shared.path(), &id, Keep::Theirs).is_err(),
            "{id:?} settled with theirs"
        );
        assert!(
            settle(&one.data, shared.path(), &id, Keep::Mine).is_err(),
            "{id:?} settled with mine"
        );
        assert!(
            settle(&one.data, shared.path(), &id, Keep::Both).is_err(),
            "{id:?} settled with both"
        );
    }
}

#[test]
fn forgetting_any_hostile_identifier_never_deletes_a_file_outside_docs() {
    let shared = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(shared.path().join(PAPERS)).unwrap();
    let bystander = shared.path().join("secret.md");
    std::fs::write(&bystander, "no es tuyo").unwrap();

    for id in hostile_ids() {
        forget_paper(shared.path(), &id);
    }

    assert_eq!(std::fs::read_to_string(&bystander).unwrap(), "no es tuyo");
}

#[test]
fn a_round_does_not_read_back_what_it_just_wrote() {
    let one = machine("dev_a");
    let big: Vec<u8> = (0..(tisty_core::attach::COPIED_UP_TO as usize + 1024))
        .map(|at| (at % 251) as u8)
        .collect();
    let heavy = planted(&one.data, "charla.mp4", &big);
    let shared = tempfile::tempdir().unwrap();
    carry_holding(
        &one.data,
        None,
        &one.device,
        shared.path(),
        Way::Both,
        &[],
        Holds::Shared,
    )
    .unwrap();
    assert!(!one.data.join(&heavy).exists(), "it went up and let go");

    // What did not verify stays here, and a round that carried nothing tries nothing.
    let again = carry_holding(
        &one.data,
        None,
        &one.device,
        shared.path(),
        Way::Both,
        &[],
        Holds::Shared,
    )
    .unwrap();

    assert_eq!(again.freed, 0, "a second round has nothing to free");
    assert_eq!(again.sent, 0, "and nothing to send either");
}

#[test]
fn letting_go_frees_only_what_the_shared_folder_really_holds() {
    let one = machine("dev_a");
    let heavy = planted(&one.data, "charla.mp4", &vec![3u8; 4000]);
    let light = planted(&one.data, "nota.txt", b"lo apuntado");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    let done = let_go_telling(&one.data, shared.path(), 1000, &|_| false, &mut |_| true).unwrap();

    assert_eq!(done.gone, 1, "only the big one");
    assert_eq!(done.freed, 4000);
    assert!(!one.data.join(&heavy).exists(), "it went");
    assert!(one.data.join(&light).is_file(), "the small one stayed");
    assert!(shared.path().join(&heavy).is_file(), "and it is up there");
}

#[test]
fn nothing_is_freed_when_what_is_up_there_is_not_the_same_file() {
    let one = machine("dev_a");
    let heavy = planted(&one.data, "charla.mp4", &vec![3u8; 4000]);
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();
    std::fs::write(shared.path().join(&heavy), vec![9u8; 4000]).unwrap();

    let done = let_go_telling(&one.data, shared.path(), 1000, &|_| false, &mut |_| true).unwrap();

    assert_eq!(done.gone, 0);
    assert_eq!(done.kept, vec![heavy.clone()]);
    assert!(
        one.data.join(&heavy).is_file(),
        "a copy that does not vouch for itself keeps the local one alive"
    );
}

#[cfg(windows)]
#[test]
fn nothing_is_freed_when_the_copy_up_there_is_a_hole_its_keeper_has_not_filled() {
    use std::io::Write;
    use std::os::windows::fs::OpenOptionsExt;

    const OFFLINE: u32 = 0x0000_1000;

    let one = machine("dev_a");
    let heavy = planted(&one.data, "charla.mp4", &vec![3u8; 4000]);
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    let up = shared.path().join(&heavy);
    let body = std::fs::read(&up).unwrap();
    std::fs::remove_file(&up).unwrap();
    let mut marked = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .attributes(OFFLINE)
        .open(&up)
        .unwrap();
    marked.write_all(&body).unwrap();
    drop(marked);

    let done = let_go_telling(&one.data, shared.path(), 1000, &|_| false, &mut |_| true).unwrap();

    assert_eq!(done.gone, 0, "nothing is let go of");
    assert_eq!(done.kept, vec![heavy.clone()]);
    assert!(
        one.data.join(&heavy).is_file(),
        "a copy nobody can answer for without fetching it keeps the local one alive"
    );
}

#[cfg(windows)]
#[test]
fn what_the_round_just_wrote_counts_as_landed_even_when_its_keeper_took_the_body() {
    use std::io::Write;
    use std::os::windows::fs::OpenOptionsExt;

    const OFFLINE: u32 = 0x0000_1000;
    const SHARED_WITH_NOBODY: u32 = 0;

    let room = tempfile::tempdir().unwrap();
    let there = room.path().join("charla-d5d43135.mp4");
    let mut marked = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .attributes(OFFLINE)
        .open(&there)
        .unwrap();
    marked.write_all(b"lo grabado").unwrap();
    drop(marked);
    let (sha256, bytes) = tisty_core::attach::hashed(&there).unwrap();

    let shut = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(SHARED_WITH_NOBODY)
        .open(&there)
        .unwrap();
    let told = super::held::landed_whole(
        &there,
        Some(&(sha256, bytes)),
        bytes,
        "attachments/cd/charla-d5d43135.mp4",
    );
    drop(shut);

    assert!(
        told,
        "the round wrote and hashed it on the way up, so nobody has to read it back"
    );
}

#[cfg(windows)]
#[test]
fn a_hole_another_machine_says_it_holds_is_let_go_of_without_reading_it() {
    use std::io::Write;
    use std::os::windows::fs::OpenOptionsExt;

    const OFFLINE: u32 = 0x0000_1000;
    const SHARED_WITH_NOBODY: u32 = 0;

    let one = machine("dev_a");
    let heavy = planted(&one.data, "charla.mp4", &vec![3u8; 4000]);
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    let up = shared.path().join(&heavy);
    let body = std::fs::read(&up).unwrap();
    std::fs::remove_file(&up).unwrap();
    let mut marked = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .attributes(OFFLINE)
        .open(&up)
        .unwrap();
    marked.write_all(&body).unwrap();
    drop(marked);

    let shut = std::fs::OpenOptions::new()
        .read(true)
        .share_mode(SHARED_WITH_NOBODY)
        .open(&up)
        .unwrap();
    let done = let_go_telling(&one.data, shared.path(), 1000, &|_| true, &mut |_| true).unwrap();
    drop(shut);

    assert_eq!(done.gone, 1, "the log said another machine has it");
    assert_eq!(
        done.let_go,
        vec![heavy.clone()],
        "it did not say what it let go of"
    );
    assert!(
        !one.data.join(&heavy).exists(),
        "the local copy stayed although nothing had to be read to be sure"
    );
}

#[test]
fn a_body_that_contradicts_what_the_log_says_it_holds_is_not_taken_in() {
    let one = machine("dev_a");
    let body = b"lo grabado";
    let heavy = planted(&one.data, "charla.mp4", body);
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    let brought_to = |avowed: &std::collections::BTreeMap<String, (String, u64)>| {
        let other = machine("dev_b");
        copy_held(
            &shared.path().join(HELD),
            &other.data.join(HELD),
            &Default::default(),
            false,
            Some(&other.data),
            None,
            None,
            None,
            avowed,
            &mut |_| {},
        )
        .unwrap();
        other.data.join(&heavy).is_file()
    };

    let lying = std::collections::BTreeMap::from([(
        heavy.clone(),
        (tisty_core::attach::printed(b"otra cosa"), body.len() as u64),
    )]);
    assert!(
        !brought_to(&lying),
        "a body the log says holds something else was taken in"
    );

    let telling = std::collections::BTreeMap::from([(
        heavy.clone(),
        (tisty_core::attach::printed(body), body.len() as u64),
    )]);
    assert!(
        brought_to(&telling),
        "a body that holds what the log says was turned away"
    );
}

#[test]
fn a_machine_that_takes_a_body_in_is_told_what_it_now_holds() {
    let one = machine("dev_a");
    let body = b"lo grabado";
    let heavy = planted(&one.data, "charla.mp4", body);
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    let two = machine("dev_b");
    stitch(&two.data, &two.device, shared.path(), None).unwrap();
    let moved = carry(&two.data, &two.device, shared.path(), Way::Pull, &[]).unwrap();

    assert!(
        two.data.join(&heavy).is_file(),
        "the body never reached the second machine"
    );
    assert_eq!(
        moved.took_in,
        vec![(
            heavy.clone(),
            tisty_core::attach::printed(body),
            body.len() as u64
        )],
        "the round said nothing about what this machine now holds"
    );
}

#[test]
fn nothing_is_freed_when_the_shared_folder_never_saw_it() {
    let one = machine("dev_a");
    let heavy = planted(&one.data, "charla.mp4", &vec![3u8; 4000]);
    let shared = tempfile::tempdir().unwrap();

    let done = let_go_telling(&one.data, shared.path(), 1000, &|_| false, &mut |_| true).unwrap();

    assert_eq!(done.gone, 0);
    assert!(one.data.join(&heavy).is_file());
}

#[test]
fn a_round_on_a_machine_that_shares_them_does_not_bring_the_big_ones_home() {
    let one = machine("dev_a");
    let big: Vec<u8> = (0..(tisty_core::attach::COPIED_UP_TO as usize + 1024))
        .map(|at| (at % 251) as u8)
        .collect();
    let heavy = planted(&one.data, "charla.mp4", &big);
    let light = planted(&one.data, "nota.txt", b"lo apuntado");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    let other = blank("dev_b");
    carry_holding(
        &other.data,
        None,
        &other.device,
        shared.path(),
        Way::Both,
        &[],
        Holds::Shared,
    )
    .unwrap();

    assert!(other.data.join(&light).is_file(), "the small one came");
    assert!(!other.data.join(&heavy).exists(), "the big one did not");
    assert!(
        shared.path().join(&heavy).is_file(),
        "and it waits up there"
    );
}

#[test]
fn a_round_lets_go_of_what_it_just_pushed_when_that_is_the_setting() {
    let one = machine("dev_a");
    let big: Vec<u8> = (0..(tisty_core::attach::COPIED_UP_TO as usize + 1024))
        .map(|at| (at % 251) as u8)
        .collect();
    let heavy = planted(&one.data, "charla.mp4", &big);
    let shared = tempfile::tempdir().unwrap();

    let moved = carry_holding(
        &one.data,
        None,
        &one.device,
        shared.path(),
        Way::Both,
        &[],
        Holds::Shared,
    )
    .unwrap();

    assert!(shared.path().join(&heavy).is_file(), "it went up");
    assert!(!one.data.join(&heavy).exists(), "and it is no longer here");
    assert!(moved.freed > 0, "and the round says how much it freed");
}

#[test]
fn a_history_this_machine_inherited_reaches_the_folder_too() {
    let gone = machine("dev_old");
    let shared = tempfile::tempdir().unwrap();
    carry(&gone.data, &gone.device, shared.path(), Way::Push, &[]).unwrap();

    let heir = blank("dev_heir");
    carry(&heir.data, &heir.device, shared.path(), Way::Pull, &[]).unwrap();
    wrote(&heir, "lo del heredero".into());

    let later = tempfile::tempdir().unwrap();
    carry(&heir.data, &heir.device, later.path(), Way::Push, &[]).unwrap();

    let fresh = blank("dev_fresh");
    carry(&fresh.data, &fresh.device, later.path(), Way::Pull, &[]).unwrap();

    let said = titles(&fresh.store);
    assert!(
        said.contains(&"lo de dev_old".to_string()),
        "the inherited history never reached the new folder: {said:?}"
    );
    assert!(said.contains(&"lo del heredero".to_string()), "{said:?}");
}

#[test]
fn handing_one_on_never_shortens_what_its_own_machine_wrote() {
    let two = machine("dos");
    let shared = tempfile::tempdir().unwrap();
    carry(&two.data, &two.device, shared.path(), Way::Push, &[]).unwrap();

    let one = blank("uno");
    carry(&one.data, &one.device, shared.path(), Way::Pull, &[]).unwrap();
    wrote(&two, "lo que dos siguio escribiendo".into());
    carry(&two.data, &two.device, shared.path(), Way::Push, &[]).unwrap();

    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    let said = titles(&shared.path().join(STORE));
    assert!(
        said.contains(&"lo que dos siguio escribiendo".to_string()),
        "a stale copy wrote over what that machine had already put there: {said:?}"
    );
}

#[test]
fn a_machine_taken_off_the_list_leaves_no_document_the_folder_cannot_name() {
    let gone = machine("dev_gone");
    filed(&gone, "dev_gone-0001", "# lo de la retirada\n");
    let shared = tempfile::tempdir().unwrap();
    carry(&gone.data, &gone.device, shared.path(), Way::Both, &[]).unwrap();

    let heir = blank("dev_heir");
    carry(&heir.data, &heir.device, shared.path(), Way::Pull, &[]).unwrap();
    says(
        &heir,
        Op::DeviceRemove {
            d: DeviceId(gone.device.clone()),
        },
    );

    let later = tempfile::tempdir().unwrap();
    carry(&heir.data, &heir.device, later.path(), Way::Both, &[]).unwrap();

    assert_eq!(
        unclaimed(later.path()),
        Holding::Whole,
        "the folder holds a document no history there can name"
    );
    assert!(later.path().join(STORE).join(&gone.device).is_dir());
}

#[test]
fn a_fresh_machine_can_open_every_document_the_folder_holds() {
    let gone = machine("dev_gone");
    filed(&gone, "dev_gone-0001", "# la primera\n");
    filed(&gone, "dev_gone-0002", "# la segunda\n");
    let first = tempfile::tempdir().unwrap();
    carry(&gone.data, &gone.device, first.path(), Way::Both, &[]).unwrap();

    let heir = blank("dev_heir");
    carry(&heir.data, &heir.device, first.path(), Way::Pull, &[]).unwrap();
    says(
        &heir,
        Op::DeviceRemove {
            d: DeviceId(gone.device.clone()),
        },
    );
    filed(&heir, "dev_heir-0001", "# la del heredero\n");

    let later = tempfile::tempdir().unwrap();
    carry(&heir.data, &heir.device, later.path(), Way::Both, &[]).unwrap();
    let fresh = blank("dev_fresh");
    carry(&fresh.data, &fresh.device, later.path(), Way::Pull, &[]).unwrap();

    let named = tisty_core::State::replay(&tisty_core::store::read_all(&fresh.store).unwrap())
        .docs
        .len();
    assert_eq!(
        named,
        tisty_core::docs::names(&later.path().join(PAPERS)).len(),
        "the folder holds a document its own log cannot name"
    );
}

#[test]
fn a_folder_whose_history_went_missing_says_how_much_it_is_holding() {
    let one = machine("dev_a");
    filed(&one, "dev_a-0001", "# Lo escrito\n");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();
    assert_eq!(
        unclaimed(shared.path()),
        Holding::Whole,
        "a whole folder claims its own"
    );

    std::fs::remove_dir_all(shared.path().join(STORE).join(&one.device)).unwrap();

    assert_eq!(unclaimed(shared.path()), Holding::Strays(1));
}

#[test]
fn a_folder_nobody_has_written_to_is_holding_nothing() {
    let empty = tempfile::tempdir().unwrap();

    assert_eq!(unclaimed(empty.path()), Holding::Whole);
}

#[cfg(unix)]
#[test]
fn the_documents_are_home_before_a_single_attachment_is_fetched() {
    let one = machine("uno");
    filed(&one, "uno-0001", "# Lo que se lee\n");
    let kept = planted(&one.data, "foto.png", b"unos bytes cualesquiera");
    let shelf = kept.split('/').nth(1).unwrap().to_string();
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

    let two = blank("dos");
    let here = two.data.join(HELD);
    std::fs::create_dir_all(&here).unwrap();
    let elsewhere = tempfile::tempdir().unwrap();
    std::os::unix::fs::symlink(elsewhere.path(), here.join(&shelf)).unwrap();

    let outcome = carry(&two.data, &two.device, shared.path(), Way::Pull, &[]);

    assert!(matches!(outcome, Err(Trouble::Refused(_))), "{outcome:?}");
    assert!(
        two.data.join(PAPERS).join("uno-0001.md").is_file(),
        "the shelf held up the reading: a document waited on bytes nobody asked for"
    );
}

#[test]
fn a_round_says_the_log_is_home_before_it_says_the_documents_are() {
    let one = machine("uno");
    filed(&one, "uno-0001", "# Lo que se lee\n");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

    let two = blank("dos");
    let mut heard = Vec::new();
    carry_telling(
        &two.data,
        None,
        &two.device,
        shared.path(),
        Way::Pull,
        &[],
        Holds::Everywhere,
        &mut |far| heard.push(far),
    )
    .unwrap();

    heard.retain(|far| !matches!(far, Reached::Along { .. }));
    assert_eq!(heard, vec![Reached::Log, Reached::Papers]);
}

#[test]
fn a_round_that_carried_nothing_says_nothing() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

    let mut heard = Vec::new();
    carry_telling(
        &one.data,
        None,
        &one.device,
        shared.path(),
        Way::Both,
        &[],
        Holds::Everywhere,
        &mut |far| heard.push(far),
    )
    .unwrap();

    heard.retain(|far| !matches!(far, Reached::Along { .. }));
    assert!(heard.is_empty(), "{heard:?}");
}

#[test]
fn a_document_the_folder_is_holding_is_told_apart_from_one_nobody_has() {
    let one = machine("uno");
    filed(&one, "uno-0001", "# Lo que se lee\n");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

    assert!(paper_waiting(shared.path(), "uno-0001"));
    assert!(!paper_waiting(shared.path(), "uno-0002"));
}

#[test]
fn a_machine_that_lost_its_own_history_takes_it_back_from_the_folder() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

    std::fs::remove_dir_all(one.store.join(&one.device)).unwrap();

    carry(&one.data, &one.device, shared.path(), Way::Pull, &[]).unwrap();

    assert_eq!(titles(&one.store), vec!["lo de uno".to_string()]);
}

#[test]
fn what_this_machine_wrote_and_never_sent_is_never_written_over() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();
    wrote(&one, "lo que aun no ha viajado".into());

    carry(&one.data, &one.device, shared.path(), Way::Pull, &[]).unwrap();

    let said = titles(&one.store);
    assert!(
        said.contains(&"lo que aun no ha viajado".to_string()),
        "{said:?}"
    );
    assert_eq!(said.len(), 2, "{said:?}");
}

#[test]
fn a_history_that_went_another_way_is_never_taken_for_our_own() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();
    let theirs = shared.path().join(STORE).join(&one.device);
    for at in tisty_core::store::segments_in(&theirs).unwrap() {
        let mut said = std::fs::read(&at).unwrap();
        said.extend(said.clone());
        std::fs::write(&at, said).unwrap();
    }
    std::fs::remove_dir_all(one.store.join(&one.device)).unwrap();
    wrote(&one, "lo que esta maquina siguio por su lado".into());

    carry(&one.data, &one.device, shared.path(), Way::Pull, &[]).unwrap();

    let said = titles(&one.store);
    assert_eq!(
        said,
        vec!["lo que esta maquina siguio por su lado".to_string()],
        "a history that diverged was taken as our own"
    );
}

fn held_in(at: &Path) -> Vec<(String, Vec<u8>)> {
    let mut found: Vec<(String, Vec<u8>)> = std::fs::read_dir(at)
        .unwrap()
        .filter_map(|one| one.ok())
        .filter(|one| one.path().is_file())
        .map(|one| {
            (
                one.file_name().to_string_lossy().into_owned(),
                std::fs::read(one.path()).unwrap(),
            )
        })
        .collect();
    found.sort();
    found
}

fn forked(at: &Path, keeping: usize) {
    let active = at.join("active.tisty");
    let whole = std::fs::read_to_string(&active).unwrap();
    let kept: Vec<&str> = whole.lines().take(keeping).collect();
    std::fs::write(&active, format!("{}\n", kept.join("\n"))).unwrap();
}

#[test]
fn a_history_that_went_another_way_is_never_handed_on_over_its_own_machine() {
    let three = machine("dev_c");
    for n in 1..8 {
        wrote(&three, format!("c {n}"));
    }
    let shared = tempfile::tempdir().unwrap();
    carry(&three.data, &three.device, shared.path(), Way::Both, &[]).unwrap();

    let one = blank("dev_a");
    carry(&one.data, &one.device, shared.path(), Way::Pull, &[]).unwrap();
    wrote(&one, "de a".into());
    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

    forked(&three.store.join(&three.device), 3);
    for n in 1..4 {
        wrote(&three, format!("c despues del restore {n}"));
    }
    carry(&three.data, &three.device, shared.path(), Way::Both, &[]).unwrap();
    let published = titles(&shared.path().join(STORE));

    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

    let now = titles(&shared.path().join(STORE));
    let gone: Vec<&String> = published.iter().filter(|one| !now.contains(one)).collect();
    assert!(
        gone.is_empty(),
        "events published by another machine were destroyed: {gone:?}"
    );
}

#[test]
fn two_machines_handing_one_history_on_stop_instead_of_taking_turns() {
    let three = machine("dev_c");
    for n in 1..8 {
        wrote(&three, format!("c {n}"));
    }
    let shared = tempfile::tempdir().unwrap();
    carry(&three.data, &three.device, shared.path(), Way::Both, &[]).unwrap();
    let one = blank("dev_a");
    carry(&one.data, &one.device, shared.path(), Way::Pull, &[]).unwrap();
    forked(&three.store.join(&three.device), 3);
    for n in 1..4 {
        wrote(&three, format!("c otra vez {n}"));
    }
    carry(&three.data, &three.device, shared.path(), Way::Both, &[]).unwrap();

    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();
    carry(&three.data, &three.device, shared.path(), Way::Both, &[]).unwrap();
    let last = carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

    assert_eq!(
        last.sent, 0,
        "two machines are re-uploading a whole segment at each other every round"
    );
}

#[test]
fn handing_a_history_on_never_writes_over_one_we_cannot_read_whole() {
    let three = machine("dev_c");
    for n in 1..6 {
        wrote(&three, format!("c {n}"));
    }
    let shared = tempfile::tempdir().unwrap();
    carry(&three.data, &three.device, shared.path(), Way::Both, &[]).unwrap();
    let two = blank("dev_b");
    carry(&two.data, &two.device, shared.path(), Way::Pull, &[]).unwrap();
    for n in 1..5 {
        wrote(&three, format!("c mas {n}"));
    }
    carry(&three.data, &three.device, shared.path(), Way::Both, &[]).unwrap();

    let theirs = shared.path().join(STORE).join(&three.device);
    let closed = theirs.join("000001.tisty");
    let whole = std::fs::read_to_string(theirs.join("active.tisty")).unwrap();
    std::fs::remove_file(theirs.join("active.tisty")).unwrap();
    std::fs::write(
        theirs.join("000001.count"),
        whole.lines().count().to_string(),
    )
    .unwrap();
    let cut: Vec<&str> = whole.lines().take(4).collect();
    std::fs::write(&closed, format!("{}\n", cut.join("\n"))).unwrap();
    assert_eq!(tisty_core::store::distinct_in(&theirs).unwrap(), 4);
    assert!(
        tisty_core::store::check_device(&theirs).is_err(),
        "the setup is not torn"
    );
    let was = held_in(&theirs);

    wrote(&two, "de b".into());
    carry(&two.data, &two.device, shared.path(), Way::Push, &[]).unwrap();

    assert_eq!(
        held_in(&theirs),
        was,
        "a history nobody can read whole was written into anyway"
    );
    assert!(closed.is_file());
}

#[test]
fn a_folder_whose_history_cannot_be_read_is_not_called_whole() {
    let one = machine("dev_a");
    filed(&one, "dev_a-0001", "# lo escrito\n");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();
    assert_eq!(unclaimed(shared.path()), Holding::Whole);

    let theirs = shared.path().join(STORE).join(&one.device);
    let whole = std::fs::read_to_string(theirs.join("active.tisty")).unwrap();
    std::fs::rename(theirs.join("active.tisty"), theirs.join("000001.tisty")).unwrap();
    std::fs::write(
        theirs.join("000001.count"),
        (whole.lines().count() + 1).to_string(),
    )
    .unwrap();

    assert_eq!(
        unclaimed(shared.path()),
        Holding::Unreadable,
        "a folder nobody can read was called whole"
    );
}

#[test]
fn a_document_the_log_deleted_is_not_called_a_stray() {
    let one = machine("dev_b");
    filed(&one, "dev_b-0001", "# lo escrito\n");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();
    assert_eq!(unclaimed(shared.path()), Holding::Whole);

    let id = tisty_core::State::replay(&tisty_core::store::read_all(&one.store).unwrap())
        .docs
        .values()
        .find(|one| one.file == "dev_b-0001")
        .map(|one| one.id)
        .unwrap();
    says(&one, Op::DocDelete { id });
    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

    assert_eq!(
        unclaimed(shared.path()),
        Holding::Whole,
        "a document the log itself deleted was reported as belonging to no history"
    );
}

#[test]
fn a_document_one_machine_deleted_leaves_the_other_machine_too() {
    let one = machine("dev_a");
    filed(
        &one,
        "dev_a-0001",
        "# the first
",
    );
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    let two = blank("dev_b");
    joined(&two, shared.path());

    let here = |who: &Machine| {
        tisty_core::State::replay(&tisty_core::store::read_all(&who.store).unwrap())
            .docs
            .values()
            .any(|d| d.file == "dev_a-0001")
    };
    assert!(
        here(&two),
        "the second machine has it before anything is deleted"
    );

    let id = tisty_core::State::replay(&tisty_core::store::read_all(&one.store).unwrap())
        .docs
        .values()
        .find(|d| d.file == "dev_a-0001")
        .map(|d| d.id)
        .unwrap();
    says(&one, Op::DocDelete { id });
    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();
    carry(&two.data, &two.device, shared.path(), Way::Both, &[]).unwrap();

    assert!(
        !here(&two),
        "the delete reached the second machine, so the document is not left showing an error"
    );
    assert!(
        !two.data.join(PAPERS).join("dev_a-0001.md").exists(),
        "and its file went with it"
    );
}

#[test]
fn a_file_that_went_before_its_delete_did_stops_showing_an_error_once_it_arrives() {
    let one = machine("dev_a");
    filed(
        &one,
        "dev_a-0001",
        "# the first
",
    );
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    let two = blank("dev_b");
    joined(&two, shared.path());

    let told = |who: &Machine| {
        tisty_core::State::replay(&tisty_core::store::read_all(&who.store).unwrap())
    };
    let here = |who: &Machine| told(who).docs.values().any(|d| d.file == "dev_a-0001");
    let id = told(&one)
        .docs
        .values()
        .find(|d| d.file == "dev_a-0001")
        .map(|d| d.id)
        .unwrap();

    forget_paper(shared.path(), "dev_a-0001");
    carry(&two.data, &two.device, shared.path(), Way::Both, &[]).unwrap();
    assert!(
        here(&two),
        "the file is gone but nothing said to forget it, so the document still stands"
    );

    says(&one, Op::DocDelete { id });
    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();
    carry(&two.data, &two.device, shared.path(), Way::Both, &[]).unwrap();

    assert!(
        !here(&two),
        "once the delete arrives the document goes, however late it came"
    );
}

#[test]
fn a_folder_that_lost_its_history_says_how_much_it_is_holding() {
    let one = machine("dev_a");
    filed(&one, "dev_a-0001", "# la primera\n");
    filed(&one, "dev_a-0002", "# la segunda\n");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

    std::fs::remove_dir_all(shared.path().join(STORE)).unwrap();

    assert_eq!(unclaimed(shared.path()), Holding::Strays(2));
}

#[test]
fn taking_our_own_history_back_never_writes_over_a_live_writer() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();
    let mine = one.store.join(&one.device);
    std::fs::remove_dir_all(&mine).unwrap();

    let writing = tisty_core::store::alone(&mine).expect("nobody else holds it");
    carry(&one.data, &one.device, shared.path(), Way::Pull, &[]).unwrap();

    assert!(
        !mine.join("active.tisty").is_file(),
        "a live writer's history was renamed out from under it"
    );
    drop(writing);
    carry(&one.data, &one.device, shared.path(), Way::Pull, &[]).unwrap();
    assert!(
        mine.join("active.tisty").is_file(),
        "and it never came back after"
    );
}

#[cfg(unix)]
#[test]
fn an_interrupted_push_never_leaves_a_document_ahead_of_the_bytes_it_names() {
    let one = machine("uno");
    let kept = planted(&one.data, "foto.png", b"unos bytes cualesquiera");
    let shelf = kept.split('/').nth(1).unwrap().to_string();
    filed(&one, "uno-0001", &format!("![foto]({kept})\n"));
    let shared = tempfile::tempdir().unwrap();
    let there = shared.path().join(HELD);
    std::fs::create_dir_all(&there).unwrap();
    let elsewhere = tempfile::tempdir().unwrap();
    std::os::unix::fs::symlink(elsewhere.path(), there.join(&shelf)).unwrap();

    let outcome = carry(&one.data, &one.device, shared.path(), Way::Both, &[]);

    assert!(matches!(outcome, Err(Trouble::Refused(_))), "{outcome:?}");
    assert!(
        !shared.path().join(PAPERS).join("uno-0001.md").exists(),
        "a document naming bytes that never went up was published anyway"
    );
}

#[test]
fn a_finished_round_puts_the_bytes_up_before_the_document_that_names_them() {
    let one = machine("uno");
    let kept = planted(&one.data, "foto.png", b"unos bytes cualesquiera");
    filed(&one, "uno-0001", &format!("![foto]({kept})\n"));
    let shared = tempfile::tempdir().unwrap();

    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

    assert!(
        shared.path().join(&kept).is_file(),
        "the bytes never went up"
    );
    assert!(shared.path().join(PAPERS).join("uno-0001.md").is_file());
}

fn wrote_id(who: &Machine, title: String) -> Ulid {
    let id = Ulid::generate();
    says(
        who,
        Op::TaskAdd {
            id,
            d: TaskAdd::new(title, "a0"),
        },
    );
    id
}

fn seen(store: &Path) -> tisty_core::State {
    tisty_core::State::replay(&tisty_core::store::read_all(store).unwrap())
}

fn met(one: &Machine, two: &Machine, shared: &Path) {
    carry(&one.data, &one.device, shared, Way::Both, &[]).unwrap();
    carry(&two.data, &two.device, shared, Way::Both, &[]).unwrap();
    carry(&one.data, &one.device, shared, Way::Both, &[]).unwrap();
}

fn both_know(one: &Machine, two: &Machine, shared: &Path) -> Ulid {
    carry(&one.data, &one.device, shared, Way::Push, &[]).unwrap();
    carry(&two.data, &two.device, shared, Way::Both, &[]).unwrap();
    let id = Ulid::generate();
    says(
        one,
        Op::TaskAdd {
            id,
            d: TaskAdd::new("la disputada", "a1"),
        },
    );
    met(one, two, shared);
    id
}

#[test]
fn a_task_done_on_one_machine_and_deleted_on_the_other_ends_up_gone_on_both() {
    let one = machine("uno");
    let two = blank("dos");
    let shared = tempfile::tempdir().unwrap();
    let id = both_know(&one, &two, shared.path());

    says(&one, Op::TaskDone { id, filled: false });
    says(&two, Op::TaskDelete { id });
    met(&two, &one, shared.path());
    carry(&two.data, &two.device, shared.path(), Way::Both, &[]).unwrap();

    for who in [&one, &two] {
        assert!(
            !seen(&who.store).tasks.contains_key(&id),
            "a deletion did not travel to {}",
            who.device
        );
    }
}

#[test]
fn a_title_written_two_ways_at_once_reads_the_same_on_both_machines() {
    let one = machine("uno");
    let two = blank("dos");
    let shared = tempfile::tempdir().unwrap();
    let id = both_know(&one, &two, shared.path());

    for (who, said) in [(&one, "puesto por uno"), (&two, "puesto por dos")] {
        says(
            who,
            Op::TaskUpdate {
                id,
                d: tisty_core::event::TaskPatch {
                    title: Some(said.into()),
                    ..Default::default()
                },
            },
        );
    }
    met(&one, &two, shared.path());

    assert_eq!(
        seen(&one.store).tasks[&id].title,
        seen(&two.store).tasks[&id].title,
        "the two machines disagree about the title"
    );
}

#[test]
fn a_tag_added_on_one_machine_and_taken_off_on_the_other_settles_the_same_way() {
    let one = machine("uno");
    let two = blank("dos");
    let shared = tempfile::tempdir().unwrap();
    let id = both_know(&one, &two, shared.path());

    says(
        &one,
        Op::TaskUpdate {
            id,
            d: tisty_core::event::TaskPatch {
                tags: Some(vec![tisty_core::Tag::new("urgente").unwrap()]),
                ..Default::default()
            },
        },
    );
    says(
        &two,
        Op::TaskUpdate {
            id,
            d: tisty_core::event::TaskPatch {
                tags: Some(Vec::new()),
                ..Default::default()
            },
        },
    );
    met(&one, &two, shared.path());

    assert_eq!(
        seen(&one.store).tasks[&id].tags,
        seen(&two.store).tasks[&id].tags
    );
}

#[test]
fn a_document_deleted_here_is_not_brought_back_by_an_edit_made_over_there() {
    let one = machine("uno");
    filed(&one, "uno-0001", "# Kit\n\nbase\n");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();
    let two = blank("dos");
    carry(&two.data, &two.device, shared.path(), Way::Both, &[]).unwrap();

    let id = seen(&one.store)
        .docs
        .values()
        .find(|one| one.file == "uno-0001")
        .map(|one| one.id)
        .unwrap();
    says(&one, Op::DocDelete { id });
    let _ = std::fs::remove_file(one.data.join(PAPERS).join("uno-0001.md"));

    wrote_body(
        &two.data.join(PAPERS),
        "uno-0001",
        "# Kit\n\nescrito por dos\n",
    );
    carry(&two.data, &two.device, shared.path(), Way::Both, &[]).unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

    assert!(
        !one.data.join(PAPERS).join("uno-0001.md").exists(),
        "an edit made elsewhere brought a deleted document back"
    );
}

#[test]
fn a_third_machine_that_was_away_for_many_rounds_arrives_to_all_of_it() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();
    let two = blank("dos");
    carry(&two.data, &two.device, shared.path(), Way::Both, &[]).unwrap();

    let mut marked = None;
    for n in 1..6 {
        let id = wrote_id(&one, format!("de uno ronda {n}"));
        wrote(&two, format!("de dos ronda {n}"));
        match n {
            2 => says(&one, Op::TaskDone { id, filled: false }),
            4 => {
                says(&one, Op::TaskDelete { id });
                marked = Some(id);
            }
            _ => {}
        }
        carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();
        carry(&two.data, &two.device, shared.path(), Way::Both, &[]).unwrap();
    }

    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

    let three = blank("tres");
    carry(&three.data, &three.device, shared.path(), Way::Both, &[]).unwrap();

    assert_eq!(titles(&three.store), titles(&one.store));
    assert!(!seen(&three.store).tasks.contains_key(&marked.unwrap()));
}

#[test]
fn a_history_that_rotated_while_the_other_was_away_lands_whole_on_its_return() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();
    let two = blank("dos");
    carry(&two.data, &two.device, shared.path(), Way::Both, &[]).unwrap();

    for n in 1..5 {
        wrote(&one, format!("antes de rotar {n}"));
    }
    let done = wrote_id(&one, "una que se cierra".into());
    says(
        &one,
        Op::TaskDone {
            id: done,
            filled: false,
        },
    );
    rotated(&one);
    for n in 1..5 {
        wrote(&one, format!("despues de rotar {n}"));
    }
    let gone = wrote_id(&one, "una que se borra".into());
    says(&one, Op::TaskDelete { id: gone });
    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

    carry(&two.data, &two.device, shared.path(), Way::Both, &[]).unwrap();

    assert_eq!(titles(&two.store), titles(&one.store));
    assert_eq!(
        tisty_core::store::distinct_in(&two.store.join(&one.device)).unwrap(),
        tisty_core::store::distinct_in(&one.store.join(&one.device)).unwrap(),
    );
}

#[test]
fn a_segment_put_right_without_changing_its_length_still_reaches_the_folder() {
    let one = machine("uno");
    let active = one.store.join(&one.device).join("active.tisty");
    let mut n = 0;
    while std::fs::metadata(&active)
        .map(|told| told.len())
        .unwrap_or(0)
        <= 600
    {
        n += 1;
        wrote(
            &one,
            format!("tarea numero {n} con un titulo largo de veras"),
        );
    }
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    let whole = std::fs::read_to_string(&active).unwrap();
    let said = whole
        .replacen("titulo largo", "titulo corto", 1)
        .into_bytes();
    assert_eq!(said.len(), whole.len());
    assert_ne!(said, whole.as_bytes());
    std::fs::write(&active, &said).unwrap();

    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    assert_eq!(
        std::fs::read(
            shared
                .path()
                .join(STORE)
                .join(&one.device)
                .join("active.tisty")
        )
        .unwrap(),
        said,
        "a segment that changed before its tail was taken for one already up there"
    );
}

#[test]
fn an_attachment_put_aside_and_then_named_again_travels_like_any_other() {
    let one = machine("uno");
    let kept = planted(&one.data, "contrato.pdf", b"lo que se adjunto");
    says(&one, Op::AttachRetire { d: kept.clone() });
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();
    assert!(
        !shared.path().join(&kept).exists(),
        "a retired attachment went up anyway"
    );

    filed(&one, "uno-0001", &format!("![contrato]({kept})\n"));
    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

    assert!(
        shared.path().join(&kept).is_file(),
        "a document names it again and it still could not travel"
    );
}

#[test]
fn a_document_arrives_with_its_name_before_its_body_is_read() {
    let one = machine("uno");
    let id = Ulid::generate();
    says(
        &one,
        Op::DocAdd {
            id,
            d: tisty_core::event::DocAdd {
                wrote: None,
                guest: false,
                made: None,
                by: None,
                said: None,
                file: "uno-0001".into(),
                order: "a0".into(),
                folder: None,
                page_of: None,
            },
        },
    );
    says(
        &one,
        Op::DocSaid {
            id,
            d: tisty_core::event::Said {
                title: "Cómo funciona esto".into(),
                bytes: Some(42),
                tags: Some(vec![tisty_core::Tag::new("casa").unwrap()]),
                by: None,
                print: None,
            },
        },
    );
    tisty_core::docs::write(&one.data.join(PAPERS), "uno-0001", "# Cómo funciona esto\n").unwrap();
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

    let two = blank("dos");
    carry(&two.data, &two.device, shared.path(), Way::Both, &[]).unwrap();

    let kept = seen(&two.store);
    let paper = kept
        .docs
        .get(&id)
        .expect("the log never named the document");
    assert_eq!(paper.title.as_deref(), Some("Cómo funciona esto"));
    assert_eq!(paper.bytes, Some(42));
    assert_eq!(paper.tags.len(), 1, "the tags did not travel in the log");
    assert_eq!(
        body(&two.data, "uno-0001"),
        "# Cómo funciona esto\n",
        "the body did not travel"
    );

    std::fs::remove_file(two.data.join(PAPERS).join("uno-0001.md")).unwrap();
    let still = seen(&two.store);
    assert_eq!(
        still
            .docs
            .get(&id)
            .and_then(|one| one.title.clone())
            .as_deref(),
        Some("Cómo funciona esto"),
        "the name needs the body to be read"
    );
    assert!(
        paper_waiting(shared.path(), "uno-0001"),
        "a document the folder holds was not told apart from one that is lost"
    );
}

#[test]
fn a_folder_that_cannot_name_its_documents_hands_over_no_file_nothing_reaches() {
    let one = machine("uno");
    let kept = planted(&one.data, "contrato.pdf", b"lo que nadie nombra");
    filed(&one, "uno-0001", "# sin adjuntos\n");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();
    assert!(shared.path().join(&kept).is_file());

    std::fs::remove_dir_all(shared.path().join(STORE).join(&one.device)).unwrap();
    assert!(matches!(unclaimed(shared.path()), Holding::Strays(_)));

    let two = blank("dos");
    carry(&two.data, &two.device, shared.path(), Way::Pull, &[]).unwrap();

    assert!(
        !two.data.join(&kept).exists(),
        "a fresh machine fetched bytes no document up there will ever name"
    );
    assert!(
        shared.path().join(&kept).is_file(),
        "and they stay where they were"
    );
}

#[test]
fn a_whole_folder_still_hands_over_every_file_it_holds() {
    let one = machine("uno");
    let kept = planted(&one.data, "contrato.pdf", b"lo que nadie nombra");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();
    assert_eq!(unclaimed(shared.path()), Holding::Whole);

    let two = blank("dos");
    carry(&two.data, &two.device, shared.path(), Way::Pull, &[]).unwrap();

    assert!(
        two.data.join(&kept).is_file(),
        "a whole folder stopped handing over what it holds"
    );
}

#[test]
fn a_round_that_only_takes_does_not_hold_back_what_its_own_documents_left_up_there() {
    let one = machine("uno");
    let kept = planted(&one.data, "contrato.pdf", b"lo que nadie nombra");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

    let two = blank("dos");
    carry(&two.data, &two.device, shared.path(), Way::Both, &[]).unwrap();
    std::fs::remove_file(two.data.join(&kept)).unwrap();
    filed(&two, "dos-0001", "# lo que dos escribio\n");

    carry(&two.data, &two.device, shared.path(), Way::Pull, &[]).unwrap();

    assert!(
        two.data.join(&kept).is_file(),
        "a round that only takes published a document and then held files back over it"
    );
}

#[test]
fn a_history_we_cannot_read_whole_is_never_taken_for_our_own() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();
    for said in ["dos", "tres"] {
        wrote(&one, said.into());
    }
    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

    let theirs = shared.path().join(STORE).join(&one.device);
    let whole = std::fs::read_to_string(theirs.join("active.tisty")).unwrap();
    std::fs::remove_file(theirs.join("active.tisty")).unwrap();
    std::fs::write(
        theirs.join("000001.count"),
        (whole.lines().count() + 3).to_string(),
    )
    .unwrap();
    std::fs::write(theirs.join("000001.tisty"), &whole).unwrap();
    std::fs::remove_dir_all(one.store.join(&one.device)).unwrap();

    carry(&one.data, &one.device, shared.path(), Way::Pull, &[]).unwrap();

    assert!(
        !one.store.join(&one.device).join("000001.tisty").is_file(),
        "a history whose own tally disagrees was taken back as ours"
    );
}

#[test]
fn a_longer_history_that_went_another_way_is_never_taken_for_our_own() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();
    let mine = one.store.join(&one.device);
    let ours = std::fs::read_to_string(mine.join("active.tisty")).unwrap();

    let theirs = shared.path().join(STORE).join(&one.device);
    let two = machine("uno");
    for n in 1..6 {
        wrote(&two, format!("por otro camino {n}"));
    }
    let forked = std::fs::read_to_string(two.store.join(&two.device).join("active.tisty")).unwrap();
    std::fs::write(theirs.join("active.tisty"), &forked).unwrap();
    assert!(
        forked.lines().count() > ours.lines().count(),
        "the setup is not longer"
    );

    carry(&one.data, &one.device, shared.path(), Way::Pull, &[]).unwrap();

    assert_eq!(
        std::fs::read_to_string(mine.join("active.tisty")).unwrap(),
        ours,
        "a longer history that never grew from ours was taken back over it"
    );
}

fn papered(who: &Machine, file: &str, guest: bool, by: Option<&str>) -> Ulid {
    let id = Ulid::generate();
    says(
        who,
        Op::DocAdd {
            id,
            d: tisty_core::event::DocAdd {
                wrote: None,
                guest,
                made: None,
                by: by.map(str::to_string),
                said: None,
                file: file.to_string(),
                order: "a0".into(),
                folder: None,
                page_of: None,
            },
        },
    );
    tisty_core::docs::write(&who.data.join(PAPERS), file, "# lo escrito\n").unwrap();
    id
}

#[test]
fn who_wrote_what_travels_whole_even_through_a_history_somebody_else_hands_on() {
    let old = machine("dev_old");
    says(
        &old,
        Op::Signed {
            d: tisty_core::event::Signature {
                alias: Some("mario".into()),
                name: None,
                email: None,
            },
        },
    );
    let mine = papered(&old, "dev_old-0001", false, Some("mario"));
    let theirs = papered(&old, "dev_old-0002", true, Some("otra persona"));
    let unsigned = papered(&old, "dev_old-0003", false, None);
    says(
        &old,
        Op::DocSigned {
            id: unsigned,
            d: "mario".into(),
        },
    );
    let first = tempfile::tempdir().unwrap();
    carry(&old.data, &old.device, first.path(), Way::Both, &[]).unwrap();

    let heir = blank("dev_heir");
    carry(&heir.data, &heir.device, first.path(), Way::Pull, &[]).unwrap();
    let later = tempfile::tempdir().unwrap();
    carry(&heir.data, &heir.device, later.path(), Way::Both, &[]).unwrap();

    let fresh = blank("dev_fresh");
    carry(&fresh.data, &fresh.device, later.path(), Way::Pull, &[]).unwrap();

    let told = seen(&fresh.store);
    assert_eq!(told.docs[&mine].by.as_deref(), Some("mario"));
    assert!(!told.docs[&mine].guest);
    assert_eq!(told.docs[&theirs].by.as_deref(), Some("otra persona"));
    assert!(
        told.docs[&theirs].guest,
        "a guest document lost that it was one"
    );
    assert_eq!(
        told.docs[&unsigned].by.as_deref(),
        Some("mario"),
        "a signature written after the fact did not travel"
    );
    assert!(!told.docs[&unsigned].guest);
    for file in ["dev_old-0001", "dev_old-0002", "dev_old-0003"] {
        assert_eq!(
            body(&fresh.data, file),
            "# lo escrito\n",
            "{file} arrived without its body"
        );
    }
}

#[test]
fn a_guest_document_is_carried_like_any_other_when_the_folder_is_adrift() {
    let one = machine("uno");
    let kept = planted(&one.data, "contrato.pdf", b"lo adjunto");
    papered(&one, "uno-0001", true, Some("quien lo escribio"));
    tisty_core::docs::write(
        &one.data.join(PAPERS),
        "uno-0001",
        &format!("# de otra persona\n\n![contrato]({kept})\n"),
    )
    .unwrap();
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

    theirs(
        shared.path(),
        "suelto-0001",
        "# lo que ninguna historia nombra\n",
    );
    assert!(matches!(unclaimed(shared.path()), Holding::Strays(_)));

    let two = blank("dos");
    carry(&two.data, &two.device, shared.path(), Way::Pull, &[]).unwrap();

    let told = seen(&two.store);
    let paper = told
        .docs
        .values()
        .find(|one| one.file == "uno-0001")
        .unwrap();
    assert!(paper.guest, "a guest document lost that it was one");
    assert_eq!(paper.by.as_deref(), Some("quien lo escribio"));
    assert!(
        two.data.join(&kept).is_file(),
        "an adrift folder held back a file a guest document names"
    );
}

#[test]
fn a_history_handed_on_reaches_the_folder_byte_for_byte() {
    let old = machine("dev_old");
    says(
        &old,
        Op::Signed {
            d: tisty_core::event::Signature {
                alias: Some("mario".into()),
                name: Some("Mario".into()),
                email: Some("mario@example.com".into()),
            },
        },
    );
    let id = papered(&old, "dev_old-0001", true, Some("otra persona"));
    says(
        &old,
        Op::DocSaid {
            id,
            d: tisty_core::event::Said {
                title: "Lo que trajo alguien".into(),
                bytes: Some(13),
                tags: Some(vec![tisty_core::Tag::new("casa").unwrap()]),
                by: Some("otra persona".into()),
                print: None,
            },
        },
    );
    let first = tempfile::tempdir().unwrap();
    carry(&old.data, &old.device, first.path(), Way::Both, &[]).unwrap();
    let written = std::fs::read(
        first
            .path()
            .join(STORE)
            .join(&old.device)
            .join("active.tisty"),
    )
    .unwrap();

    let heir = blank("dev_heir");
    carry(&heir.data, &heir.device, first.path(), Way::Pull, &[]).unwrap();
    let later = tempfile::tempdir().unwrap();
    carry(&heir.data, &heir.device, later.path(), Way::Push, &[]).unwrap();

    let handed = std::fs::read(
        later
            .path()
            .join(STORE)
            .join(&old.device)
            .join("active.tisty"),
    )
    .unwrap();
    assert_eq!(
        handed, written,
        "a history handed on came out different from the one that was written"
    );
    let said = String::from_utf8(handed).unwrap();
    let stamped = format!("\"v\":{}", tisty_core::event::SCHEMA_VERSION);
    for kept in [
        stamped.as_str(),
        "person.signed",
        "mario@example.com",
        "\"guest\":true",
        "otra persona",
        "casa",
    ] {
        assert!(
            said.contains(kept),
            "{kept} did not survive being handed on"
        );
    }
}

#[test]
fn what_a_machine_leaves_behind_depends_on_how_it_holds_them() {
    assert_eq!(left_behind(Holds::Everywhere), None);
    assert_eq!(
        left_behind(Holds::Mine),
        Some(tisty_core::attach::COPIED_UP_TO)
    );
    assert_eq!(
        left_behind(Holds::Shared),
        Some(tisty_core::attach::COPIED_UP_TO)
    );
}

#[test]
fn a_machine_that_leaves_the_big_ones_behind_still_takes_the_small() {
    let one = machine("dev_a");
    let heavy = planted(&one.data, "charla.mp4", &vec![7u8; 3000]);
    let light = planted(&one.data, "nota.txt", b"lo apuntado");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();
    let other = blank("dev_b");

    copy_held(
        &shared.path().join(HELD),
        &other.data.join(HELD),
        &Default::default(),
        false,
        Some(&other.data),
        Some(1000),
        None,
        None,
        &Default::default(),
        &mut |_| {},
    )
    .unwrap();

    assert!(
        other.data.join(&light).is_file(),
        "the small one travels as it always did"
    );
    assert!(
        !other.data.join(&heavy).exists(),
        "the big one waits in the shared folder"
    );
    assert!(
        shared.path().join(&heavy).is_file(),
        "and it is there to be fetched"
    );
}

#[test]
fn a_big_attachment_travels_whole_and_leaves_no_half() {
    let one = machine("dev_a");
    let bytes: Vec<u8> = (0..3_000_000).map(|at| (at % 251) as u8).collect();
    let kept = planted(&one.data, "charla.mp4", &bytes);

    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();
    let other = blank("dev_b");
    carry(&other.data, &other.device, shared.path(), Way::Pull, &[]).unwrap();

    assert_eq!(std::fs::read(other.data.join(&kept)).unwrap(), bytes);
    let shelf = std::path::Path::new(&kept).parent().unwrap();
    for at in [shared.path().join(shelf), other.data.join(shelf)] {
        let left: Vec<_> = std::fs::read_dir(&at)
            .unwrap()
            .filter_map(|one| one.ok())
            .map(|one| one.file_name().to_string_lossy().into_owned())
            .filter(|name| name.contains(".part"))
            .collect();
        assert!(left.is_empty(), "half a copy stayed at {at:?}: {left:?}");
    }
}

#[test]
fn attachments_travel_with_the_tasks_that_name_them() {
    let one = machine("dev_a");
    let kept = planted(&one.data, "foto.png", b"a picture");

    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    let other = blank("dev_b");
    carry(&other.data, &other.device, shared.path(), Way::Pull, &[]).unwrap();

    assert_eq!(std::fs::read(other.data.join(&kept)).unwrap(), b"a picture");
}

#[test]
fn merging_two_unrelated_histories_loses_nothing_from_either_side() {
    let one = machine("uno");
    tisty_core::store::identity(&one.store).unwrap();
    let two = machine("dos");
    let shared = tempfile::tempdir().unwrap();
    carry(&two.data, &two.device, shared.path(), Way::Push, &[]).unwrap();

    let merged = stitch(&one.data, &one.device, shared.path(), None).unwrap();
    assert_eq!(merged.kin, Kin::Strangers);
    says(
        &one,
        Op::StoresJoined {
            d: merged.stitch.unwrap(),
        },
    );

    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();
    carry(&two.data, &two.device, shared.path(), Way::Both, &[]).unwrap();

    for who in [titles(&one.store), titles(&two.store)] {
        assert!(who.contains(&"lo de uno".to_string()), "{who:?}");
        assert!(who.contains(&"lo de dos".to_string()), "{who:?}");
    }
}

#[test]
fn merging_adopts_the_folders_store_id_never_the_local_one_nor_a_new_one() {
    let one = machine("uno");
    let local_before = tisty_core::store::identity(&one.store).unwrap();
    let two = machine("dos");
    let shared = tempfile::tempdir().unwrap();
    carry(&two.data, &two.device, shared.path(), Way::Push, &[]).unwrap();
    let folder_id = super::theirs(shared.path()).unwrap();

    stitch(&one.data, &one.device, shared.path(), None).unwrap();

    let adopted = tisty_core::store::peek_identity(&one.store).unwrap();
    assert_eq!(adopted, folder_id);
    assert_ne!(adopted, local_before);
}

#[test]
fn the_other_machine_of_the_surviving_history_syncs_after_a_merge_without_being_asked() {
    let one = machine("uno");
    tisty_core::store::identity(&one.store).unwrap();
    let two = machine("dos");
    let shared = tempfile::tempdir().unwrap();
    carry(&two.data, &two.device, shared.path(), Way::Push, &[]).unwrap();

    let seam = stitch(&one.data, &one.device, shared.path(), None)
        .unwrap()
        .stitch
        .unwrap();
    says(&one, Op::StoresJoined { d: seam });
    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

    let done = carry(&two.data, &two.device, shared.path(), Way::Both, &[]).unwrap();

    assert!(
        done.brought > 0,
        "the survivor's own machine was not offered the merge"
    );
    assert!(titles(&two.store).contains(&"lo de uno".to_string()));
}

#[test]
fn a_straggler_with_the_old_identity_is_recognized_as_the_same_lineage() {
    let one = machine("uno");
    let old_shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, old_shared.path(), Way::Push, &[]).unwrap();
    let straggler = blank("tres");
    carry(
        &straggler.data,
        &straggler.device,
        old_shared.path(),
        Way::Pull,
        &[],
    )
    .unwrap();

    let two = machine("dos");
    let shared = tempfile::tempdir().unwrap();
    carry(&two.data, &two.device, shared.path(), Way::Push, &[]).unwrap();
    let seam = stitch(&one.data, &one.device, shared.path(), None)
        .unwrap()
        .stitch
        .unwrap();
    says(&one, Op::StoresJoined { d: seam });
    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

    assert_eq!(kinship(&straggler.store, shared.path()), Kin::SameLineage);
}

#[test]
fn a_straggler_adopts_the_merged_identity_while_keeping_its_unsent_tail() {
    let one = machine("uno");
    let old_shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, old_shared.path(), Way::Push, &[]).unwrap();
    let straggler = blank("tres");
    carry(
        &straggler.data,
        &straggler.device,
        old_shared.path(),
        Way::Pull,
        &[],
    )
    .unwrap();

    let two = machine("dos");
    let shared = tempfile::tempdir().unwrap();
    carry(&two.data, &two.device, shared.path(), Way::Push, &[]).unwrap();
    let seam = stitch(&one.data, &one.device, shared.path(), None)
        .unwrap()
        .stitch
        .unwrap();
    says(&one, Op::StoresJoined { d: seam });
    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

    wrote(&straggler, "lo que tres no habia mandado".into());

    let merged = stitch(&straggler.data, &straggler.device, shared.path(), None).unwrap();
    assert_eq!(merged.kin, Kin::SameLineage);
    assert!(merged.stitch.is_none());

    let moved = carry(
        &straggler.data,
        &straggler.device,
        shared.path(),
        Way::Both,
        &[],
    )
    .unwrap();

    assert!(moved.sent > 0, "the straggler's unsent tail never left");
    assert!(shared.path().join(STORE).join(&straggler.device).exists());
    let mine = titles(&straggler.store);
    assert!(mine.contains(&"lo que tres no habia mandado".to_string()));
    assert!(mine.contains(&"lo de dos".to_string()));
}

#[test]
fn two_machines_that_happen_to_share_a_device_name_but_wrote_different_things_are_a_clash() {
    let one = blank("portable");
    wrote(&one, "lo de la primera portable".into());
    let two = blank("portable");
    wrote(&two, "lo de la otra portable".into());
    let shared = tempfile::tempdir().unwrap();
    carry(&two.data, &two.device, shared.path(), Way::Push, &[]).unwrap();

    assert_eq!(
        kinship(&one.store, shared.path()),
        Kin::Clash("portable".to_string())
    );
}

#[test]
fn merging_refuses_with_same_name_when_two_machines_clash_under_one_device_name() {
    let one = blank("portable");
    wrote(&one, "lo de la primera portable".into());
    let two = blank("portable");
    wrote(&two, "lo de la otra portable".into());
    let shared = tempfile::tempdir().unwrap();
    carry(&two.data, &two.device, shared.path(), Way::Push, &[]).unwrap();

    let Err(why) = stitch(&one.data, &one.device, shared.path(), None) else {
        panic!("two machines clashing under one name were merged");
    };

    assert_eq!(why, Trouble::SameName("portable".to_string()));
}

#[test]
fn merging_leaves_the_local_identity_untouched_when_it_refuses_a_clash() {
    let one = blank("portable");
    wrote(&one, "lo de la primera portable".into());
    let before = tisty_core::store::identity(&one.store).unwrap();
    let two = blank("portable");
    wrote(&two, "lo de la otra portable".into());
    let shared = tempfile::tempdir().unwrap();
    carry(&two.data, &two.device, shared.path(), Way::Push, &[]).unwrap();

    let _ = stitch(&one.data, &one.device, shared.path(), None);

    assert_eq!(
        tisty_core::store::peek_identity(&one.store).unwrap(),
        before
    );
}

#[test]
fn an_empty_segment_proves_no_kinship_so_a_shared_name_is_refused() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    let mine = one.store.join(&one.device);
    let file = tisty_core::store::segments_in(&mine).unwrap().remove(0);
    std::fs::write(&file, b"").unwrap();

    assert_eq!(
        kinship(&one.store, shared.path()),
        Kin::Clash(one.device.clone())
    );
}

#[test]
fn a_placeholder_the_cloud_has_not_filled_in_cannot_pass_for_the_same_history() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    let theirs = shared.path().join(STORE).join(&one.device);
    for at in tisty_core::store::segments_in(&theirs).unwrap() {
        std::fs::write(&at, b"").unwrap();
    }

    assert_eq!(
        kinship(&one.store, shared.path()),
        Kin::Clash(one.device.clone())
    );
}

#[test]
fn a_name_that_only_a_removal_remembers_still_counts_as_shared() {
    let one = machine("uno");
    let two = machine("dos");
    says(
        &one,
        Op::DeviceRemove {
            d: DeviceId(two.device.clone()),
        },
    );
    let shared = tempfile::tempdir().unwrap();
    carry(&two.data, &two.device, shared.path(), Way::Push, &[]).unwrap();
    std::fs::remove_dir_all(one.store.join(&two.device)).ok();

    assert_eq!(
        kinship(&one.store, shared.path()),
        Kin::Clash(two.device.clone()),
        "un nombre que solo vive en un evento paso desapercibido"
    );
}

#[test]
fn two_histories_with_no_name_in_common_are_still_strangers() {
    let one = machine("uno");
    let two = machine("dos");
    let shared = tempfile::tempdir().unwrap();
    carry(&two.data, &two.device, shared.path(), Way::Push, &[]).unwrap();

    assert_eq!(kinship(&one.store, shared.path()), Kin::Strangers);
}

#[test]
fn a_side_that_rotated_into_a_second_segment_is_still_the_same_lineage() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    let mine = one.store.join(&one.device);
    std::fs::rename(mine.join("active.tisty"), mine.join("000001.tisty")).unwrap();
    std::fs::write(mine.join("000001.count"), b"1").unwrap();
    std::fs::write(mine.join("active.tisty"), b"").unwrap();

    assert_eq!(kinship(&one.store, shared.path()), Kin::SameLineage);
}

#[test]
fn a_directory_present_on_only_one_side_never_turns_a_shared_directory_into_a_clash() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    let mut solo_local = Store::open(&one.store, DeviceId("solo-local".into())).unwrap();
    solo_local
        .append(Op::TaskAdd {
            id: Ulid::generate(),
            d: TaskAdd::new("solo aqui", "a0"),
        })
        .unwrap();

    let mut solo_remote =
        Store::open(shared.path().join(STORE), DeviceId("solo-remoto".into())).unwrap();
    solo_remote
        .append(Op::TaskAdd {
            id: Ulid::generate(),
            d: TaskAdd::new("solo alla", "a0"),
        })
        .unwrap();

    assert_eq!(kinship(&one.store, shared.path()), Kin::SameLineage);
}

#[test]
fn an_empty_store_directory_is_a_stranger_not_the_same_lineage() {
    let empty = blank("solitario");
    std::fs::create_dir_all(&empty.store).unwrap();
    let two = machine("dos");
    let shared = tempfile::tempdir().unwrap();
    carry(&two.data, &two.device, shared.path(), Way::Push, &[]).unwrap();

    assert_eq!(kinship(&empty.store, shared.path()), Kin::Strangers);
}

#[test]
fn merging_two_histories_lands_the_documents_of_both_sides_under_their_own_names() {
    let one = machine("uno");
    tisty_core::store::identity(&one.store).unwrap();
    filed(&one, "uno-0001", "# Lo de uno");
    let two = machine("dos");
    filed(&two, "dos-0001", "# Lo de dos");
    let shared = tempfile::tempdir().unwrap();
    carry(
        &two.data,
        &two.device,
        shared.path(),
        Way::Both,
        &["dos-0001".into()],
    )
    .unwrap();

    let seam = stitch(&one.data, &one.device, shared.path(), None)
        .unwrap()
        .stitch
        .unwrap();
    says(&one, Op::StoresJoined { d: seam });
    carry(
        &one.data,
        &one.device,
        shared.path(),
        Way::Both,
        &["uno-0001".into(), "dos-0001".into()],
    )
    .unwrap();

    assert_eq!(body(shared.path(), "uno-0001"), "# Lo de uno\n");
    assert_eq!(body(shared.path(), "dos-0001"), "# Lo de dos\n");
    assert_eq!(body(&one.data, "dos-0001"), "# Lo de dos\n");

    carry(
        &two.data,
        &two.device,
        shared.path(),
        Way::Both,
        &["uno-0001".into(), "dos-0001".into()],
    )
    .unwrap();
    assert_eq!(body(&two.data, "uno-0001"), "# Lo de uno\n");
}

#[test]
fn the_same_attachment_kept_independently_on_both_sides_of_a_merge_is_not_duplicated() {
    let one = machine("uno");
    tisty_core::store::identity(&one.store).unwrap();
    let two = machine("dos");
    let bytes: &[u8] = b"la misma fotografia, bit por bit";
    let kept_one = planted(&one.data, "foto.png", bytes);
    let kept_two = planted(&two.data, "foto.png", bytes);
    assert_eq!(
        kept_one, kept_two,
        "identical bytes under the same name must land at the same address"
    );

    let shared = tempfile::tempdir().unwrap();
    carry(&two.data, &two.device, shared.path(), Way::Both, &[]).unwrap();

    let seam = stitch(&one.data, &one.device, shared.path(), None)
        .unwrap()
        .stitch
        .unwrap();
    says(&one, Op::StoresJoined { d: seam });
    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

    let shelf = shared
        .path()
        .join(&kept_one)
        .parent()
        .unwrap()
        .to_path_buf();
    let landed = std::fs::read_dir(&shelf).unwrap().count();
    assert_eq!(
        landed, 1,
        "the same picture landed twice under different names"
    );
    assert_eq!(std::fs::read(shared.path().join(&kept_one)).unwrap(), bytes);
}

#[test]
fn a_document_deleted_in_one_history_never_comes_back_once_the_histories_are_merged() {
    let one = machine("uno");
    tisty_core::store::identity(&one.store).unwrap();
    filed(&one, "uno-0001", "# Efimero");
    let two = machine("dos");
    let shared = tempfile::tempdir().unwrap();
    carry(&two.data, &two.device, shared.path(), Way::Push, &[]).unwrap();

    let seam = stitch(&one.data, &one.device, shared.path(), None)
        .unwrap()
        .stitch
        .unwrap();
    says(&one, Op::StoresJoined { d: seam });
    carry(
        &one.data,
        &one.device,
        shared.path(),
        Way::Both,
        &["uno-0001".into()],
    )
    .unwrap();

    let mut held = signing(&one);
    let id = tisty_core::State::replay(&tisty_core::store::read_all(&one.store).unwrap())
        .docs
        .values()
        .next()
        .unwrap()
        .id;
    held.append(Op::DocDelete { id }).unwrap();
    drop(held);
    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

    carry(&two.data, &two.device, shared.path(), Way::Both, &[]).unwrap();

    assert!(!two.data.join(PAPERS).join("uno-0001.md").exists());
}

#[test]
fn a_device_removed_before_a_merge_is_still_removed_after_it() {
    let one = machine("uno");
    tisty_core::store::identity(&one.store).unwrap();
    says(
        &one,
        Op::DeviceJoin {
            d: DeviceId("uno".into()),
            k: Some(tisty_core::DeviceKind::Machine),
            p: None,
        },
    );
    says(
        &one,
        Op::DeviceJoin {
            d: DeviceId("vieja".into()),
            k: Some(tisty_core::DeviceKind::Machine),
            p: None,
        },
    );
    says(
        &one,
        Op::DeviceRemove {
            d: DeviceId("vieja".into()),
        },
    );

    let two = machine("dos");
    let shared = tempfile::tempdir().unwrap();
    carry(&two.data, &two.device, shared.path(), Way::Push, &[]).unwrap();

    let seam = stitch(&one.data, &one.device, shared.path(), None)
        .unwrap()
        .stitch
        .unwrap();
    says(&one, Op::StoresJoined { d: seam });
    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

    let said = tisty_core::store::ledger(shared.path().join(STORE)).unwrap();
    assert!(said.was_removed(&DeviceId("vieja".into())));
    assert!(!said.may_write(&DeviceId("vieja".into())));

    let vieja = blank("vieja");
    let why = carry(&vieja.data, &vieja.device, shared.path(), Way::Both, &[]).unwrap_err();
    assert!(matches!(why, Trouble::NotAllowed(_)), "{why:?}");
}

#[test]
fn a_retired_attachment_does_not_come_back_from_the_folder_once_it_is_swept() {
    let one = machine("uno");
    let kept = planted(&one.data, "foto.png", b"una fotografia retirada");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    says(&one, Op::AttachRetire { d: kept.clone() });
    let retired: std::collections::BTreeSet<String> = [kept.clone()].into();
    tisty_core::attach::sweep(&one.data, &retired, &Default::default());
    assert!(!one.data.join(&kept).exists(), "sweep did not remove it");

    carry(&one.data, &one.device, shared.path(), Way::Pull, &[]).unwrap();

    assert!(
        !one.data.join(&kept).exists(),
        "a retired attachment came back from the shared folder"
    );
}

#[test]
fn a_store_nobody_can_read_is_no_kin_at_all() {
    let shared = tempfile::tempdir().unwrap();
    let one = machine("uno");
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    assert_eq!(
        kinship(&one.store.join("nowhere"), shared.path()),
        Kin::Unsure(String::new())
    );
}

#[test]
fn a_store_it_cannot_read_stops_the_stitch_instead_of_joining_blindly() {
    let shared = tempfile::tempdir().unwrap();
    let one = machine("uno");
    let two = machine("dos");
    carry(&two.data, &two.device, shared.path(), Way::Push, &[]).unwrap();

    let blind = one.data.join("gone");

    assert!(matches!(
        stitch(&blind, &one.device, shared.path(), None),
        Err(Trouble::Unreadable(_))
    ));
}

#[test]
fn a_folder_with_no_name_of_its_own_takes_ours() {
    let shared = tempfile::tempdir().unwrap();
    let one = machine("uno");

    let said = settled(&one.store, shared.path(), false).unwrap();

    assert_eq!(said, tisty_core::store::identity(&one.store).unwrap());
}

#[test]
fn a_folder_we_just_emptied_is_refused_instead_of_named_again() {
    let shared = tempfile::tempdir().unwrap();
    let one = machine("uno");

    assert!(matches!(
        settled(&one.store, shared.path(), true),
        Err(Trouble::Emptied(_))
    ));
}

#[test]
fn two_sides_with_no_name_yet_take_the_one_this_machine_writes() {
    let shared = tempfile::tempdir().unwrap();
    let one = blank("uno");
    std::fs::create_dir_all(&one.store).unwrap();

    let said = settled(&one.store, shared.path(), false).unwrap();

    assert!(!said.is_empty());
    assert_eq!(said, tisty_core::store::identity(&one.store).unwrap());
}

fn fetched(shared: &Path, other: &Machine, most: Option<u64>, again: bool) -> usize {
    copy_held(
        &shared.join(HELD),
        &other.data.join(HELD),
        &Default::default(),
        again,
        Some(&other.data),
        most,
        None,
        None,
        &Default::default(),
        &mut |_| {},
    )
    .unwrap()
}

#[test]
fn fetching_says_how_many_files_it_took() {
    let one = machine("dev_a");
    planted(&one.data, "uno.txt", b"lo primero");
    planted(&one.data, "dos.txt", b"lo segundo");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();
    let other = blank("dev_b");

    assert_eq!(fetched(shared.path(), &other, None, false), 2);
}

#[test]
fn a_file_of_exactly_the_weight_allowed_still_travels() {
    let one = machine("dev_a");
    let just = planted(&one.data, "justo.bin", &vec![3u8; 1000]);
    let over = planted(&one.data, "pasado.bin", &vec![3u8; 1001]);
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();
    let other = blank("dev_b");

    assert_eq!(fetched(shared.path(), &other, Some(1000), false), 1);
    assert!(other.data.join(&just).is_file());
    assert!(!other.data.join(&over).exists());
}

#[test]
fn fetching_twice_does_not_carry_the_same_bytes_again() {
    let one = machine("dev_a");
    planted(&one.data, "uno.txt", b"lo primero");
    planted(&one.data, "dos.txt", b"lo segundo");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();
    let other = blank("dev_b");

    assert_eq!(fetched(shared.path(), &other, None, false), 2);
    assert_eq!(fetched(shared.path(), &other, None, false), 0);
    assert_eq!(fetched(shared.path(), &other, None, true), 2);
}

#[test]
fn both_sides_of_a_paper_come_back_word_for_word() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    tisty_core::docs::write(
        &one.data.join(PAPERS),
        "uno-0001",
        "# Lo mio\n\nlo que puse yo\n",
    )
    .unwrap();

    let there = shared.path().join(PAPERS);
    std::fs::create_dir_all(&there).unwrap();
    tisty_core::docs::write(&there, "uno-0001", "# Lo suyo\n\nlo que puso la otra\n").unwrap();

    let (mine, theirs) = both_papers(&one.data, shared.path(), "uno-0001").unwrap();

    assert_eq!(mine, "# Lo mio\n\nlo que puse yo\n");
    assert_eq!(theirs, "# Lo suyo\n\nlo que puso la otra\n");
}

#[test]
fn a_paper_only_one_side_holds_is_refused_instead_of_halved() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    tisty_core::docs::write(&one.data.join(PAPERS), "uno-0001", "# Lo mio\n").unwrap();
    std::fs::create_dir_all(shared.path().join(PAPERS)).unwrap();

    assert!(matches!(
        both_papers(&one.data, shared.path(), "uno-0001"),
        Err(Trouble::Refused(_)) | Err(Trouble::Unreadable(_))
    ));
}

#[test]
fn a_history_nobody_kept_for_us_does_not_reach_further() {
    let one = machine("uno");
    let blank_at = tempfile::tempdir().unwrap();

    assert!(!ours_reaches_further(
        &blank_at.path().join("nowhere"),
        &blank_at.path().join("neither")
    ));

    let mine = one.store.join(&one.device);
    assert!(ours_reaches_further(
        &mine,
        &blank_at.path().join("nothing")
    ));
}

#[test]
fn an_empty_history_reaches_no_further_than_an_absent_one() {
    let room = tempfile::tempdir().unwrap();
    let mine = room.path().join("mine");
    std::fs::create_dir_all(&mine).unwrap();

    assert!(!ours_reaches_further(&mine, &room.path().join("nothing")));
}

#[test]
fn two_histories_of_the_same_length_reach_the_same_distance() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    let mine = one.store.join(&one.device);
    let theirs = shared.path().join(STORE).join(&one.device);

    assert!(!ours_reaches_further(&mine, &theirs));
}

fn many_segments(who: &Machine, lots: usize) {
    let mut held = signing(who);
    for lot in 0..lots {
        let ops: Vec<Op> = (0..5_000)
            .map(|n| Op::TaskAdd {
                id: Ulid::generate(),
                d: TaskAdd::new(format!("t {lot}-{n}"), "a0"),
            })
            .collect();
        held.append_batch(ops).unwrap();
    }
}

#[test]
fn a_round_that_changes_nothing_opens_what_it_has_to_and_no_more() {
    let one = machine("dev_a");
    let shared = tempfile::tempdir().unwrap();
    let aside = tempfile::tempdir().unwrap();
    let aside = Some(aside.path());
    many_segments(&one, 3);

    let alive: Vec<String> = (0..40)
        .map(|n| {
            let id = format!("dev_a-{n:04}");
            filed(
                &one,
                &id,
                &format!(
                    "# doc {n}

{}",
                    "cuerpo de prueba
"
                    .repeat(30)
                ),
            );
            wrote_body(
                &one.data.join(PAPERS),
                &id,
                &format!(
                    "# doc {n}

{}",
                    "cuerpo de prueba
"
                    .repeat(30)
                ),
            );
            id
        })
        .collect();
    carry_leaning_on(
        &one.data,
        aside,
        &one.device,
        shared.path(),
        Way::Push,
        &alive,
    )
    .unwrap();

    let two = machine("dev_b");
    many_segments(&two, 3);
    let theirs = shared.path().join(STORE).join("dev_b");
    std::fs::create_dir_all(&theirs).unwrap();
    for at in std::fs::read_dir(two.store.join("dev_b")).unwrap() {
        let at = at.unwrap().path();
        std::fs::copy(&at, theirs.join(at.file_name().unwrap())).unwrap();
    }

    carry_leaning_on(
        &one.data,
        aside,
        &one.device,
        shared.path(),
        Way::Both,
        &alive,
    )
    .unwrap();
    carry_leaning_on(
        &one.data,
        aside,
        &one.device,
        shared.path(),
        Way::Both,
        &alive,
    )
    .unwrap();

    let before = tisty_core::counting::from_now();
    carry_leaning_on(
        &one.data,
        aside,
        &one.device,
        shared.path(),
        Way::Both,
        &alive,
    )
    .unwrap();
    let quiet = tisty_core::counting::from_now();

    assert!(
        quiet <= 65,
        "a round with nothing to carry read {quiet} files where 65 is what it takes, and the round before it read {before}"
    );

    let _ = tisty_core::counting::from_now();
    carry(&one.data, &one.device, shared.path(), Way::Both, &alive).unwrap();
    let alone = tisty_core::counting::from_now();

    assert!(
        alone <= 73,
        "the same round without a cache to lean on read {alone} files where 73 is what it takes"
    );
}

fn a_segment(at: &Path, named: &str, body: &str) {
    std::fs::create_dir_all(at).unwrap();
    std::fs::write(at.join(named), body).unwrap();
}

#[test]
fn what_was_compared_once_is_not_compared_again_until_something_moves() {
    let room = tempfile::tempdir().unwrap();
    let theirs = room.path().join("theirs");
    let mine = room.path().join("mine");
    a_segment(&theirs, "000001.tisty", "one way\n");
    a_segment(&mine, "000001.tisty", "another\n");

    let mut alike = Alike::default();
    assert!(alike.of("dev_b", &theirs, &mine).is_empty());

    std::fs::write(theirs.join("000001.tisty"), "another\n").unwrap();
    assert!(
        alike.of("dev_b", &theirs, &mine).is_empty(),
        "asking twice in one round is what this exists to avoid"
    );

    a_segment(&mine, "000002.tisty", "and more\n");
    let done = alike
        .carried("dev_b", &theirs, &mine, Toward::Folder, false)
        .unwrap();

    assert_eq!(done, 1);
    assert_eq!(
        alike.of("dev_b", &theirs, &mine).len(),
        2,
        "a copy moved something, so what was answered before is not the answer now"
    );
}

fn sown(store: &Path, device: &str, many: usize) {
    let whose = DeviceId(device.into());
    let paths = tisty_core::Paths::new(store.to_path_buf(), store.join("config"));
    let mut held = Store::open(store, whose.clone())
        .unwrap()
        .signing_with(tisty_core::signing::mine(&paths, &whose));
    for n in 0..many {
        held.append(Op::TaskAdd {
            id: Ulid::generate(),
            d: TaskAdd::new(format!("t{n}"), "a0"),
        })
        .unwrap();
    }
}

#[test]
fn a_machine_that_holds_the_same_history_hands_nothing_on() {
    let room = tempfile::tempdir().unwrap();
    let folder = room.path().join("folder");
    let store = room.path().join("store");
    sown(&folder, "dev_b", 1);
    std::fs::create_dir_all(store.join("dev_b")).unwrap();
    std::fs::copy(
        folder.join("dev_b").join("active.tisty"),
        store.join("dev_b").join("active.tisty"),
    )
    .unwrap();

    let theirs = folder.join("dev_b");
    let mine = store.join("dev_b");
    assert!(
        !ours_reaches_further(&mine, &theirs),
        "the same history is not further along"
    );

    sown(&store, "dev_b", 1);
    assert!(
        ours_reaches_further(&mine, &theirs),
        "one event more is further along, and it grew from what they hold"
    );
}

#[test]
fn a_machine_the_folder_has_never_heard_of_is_handed_on_only_if_it_wrote_something() {
    let room = tempfile::tempdir().unwrap();
    let theirs = room.path().join("folder").join("dev_b");
    let mine = room.path().join("store").join("dev_b");
    std::fs::create_dir_all(&mine).unwrap();

    assert!(
        !ours_reaches_further(&mine, &theirs),
        "holding an empty folder for somebody is nothing to hand on"
    );

    sown(&room.path().join("store"), "dev_b", 1);
    assert!(
        ours_reaches_further(&mine, &theirs),
        "what the folder never heard of is exactly what to hand on"
    );
}

#[test]
fn a_history_the_folder_holds_more_of_is_left_where_it_is() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    sown(&one.store, "dev_c", 1);
    let elsewhere = tempfile::tempdir().unwrap();
    sown(elsewhere.path(), "dev_c", 2);
    let theirs = shared.path().join(STORE).join("dev_c");
    std::fs::create_dir_all(&theirs).unwrap();
    std::fs::copy(
        elsewhere.path().join("dev_c").join("active.tisty"),
        theirs.join("active.tisty"),
    )
    .unwrap();

    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    assert_eq!(
        tisty_core::store::distinct_in(&theirs).unwrap(),
        2,
        "the folder knows more of that machine than we kept, so the little we kept is not put over it"
    );
}

#[test]
fn every_history_handed_on_is_counted() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    sown(&one.store, "dev_b", 1);
    sown(&one.store, "dev_c", 1);

    let moved = carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    assert_eq!(
        moved.sent, 3,
        "ours, and the two we were holding for others"
    );
    for who in ["dev_b", "dev_c"] {
        assert!(
            shared
                .path()
                .join(STORE)
                .join(who)
                .join("active.tisty")
                .is_file(),
            "{who} never reached the folder"
        );
    }
}

#[test]
fn handing_something_on_asks_again_what_the_two_sides_have_in_common() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    sown(&one.store, "dev_c", 1);
    let theirs = shared.path().join(STORE).join("dev_c");
    let mine = one.store.join("dev_c");

    let mut alike = Alike::default();
    assert!(
        alike.of("dev_c", &theirs, &mine).is_empty(),
        "the folder has never heard of that machine"
    );

    let sent = hand_on(&one.store, &one.device, shared.path(), false, &mut alike).unwrap();

    assert_eq!(sent, 1);
    assert_eq!(
        alike.of("dev_c", &theirs, &mine).len(),
        1,
        "the copy made them alike, and what was answered before is not the answer now"
    );
}

#[test]
fn a_folder_entry_that_is_not_a_directory_is_no_history_to_hand_on_to() {
    let room = tempfile::tempdir().unwrap();
    let store = room.path().join("store");
    sown(&store, "dev_c", 1);
    let theirs = room.path().join("folder").join("dev_c");
    std::fs::create_dir_all(theirs.parent().unwrap()).unwrap();
    std::fs::write(&theirs, "not a history\n").unwrap();

    assert!(
        !ours_reaches_further(&store.join("dev_c"), &theirs),
        "unreadable is not the same as never heard of"
    );
}

#[test]
fn a_history_of_ours_that_is_not_a_directory_is_not_one_that_went_missing() {
    let room = tempfile::tempdir().unwrap();
    let folder = room.path().join("folder");
    sown(&folder, "dev_c", 1);
    let mine = room.path().join("store").join("dev_c");
    std::fs::create_dir_all(mine.parent().unwrap()).unwrap();
    std::fs::write(&mine, "not a history\n").unwrap();

    assert!(
        !ours_went_missing(&mine, &folder.join("dev_c")),
        "a history we cannot read is not a history we lost"
    );
}

#[test]
fn the_same_history_on_both_sides_is_nothing_to_take_back() {
    let room = tempfile::tempdir().unwrap();
    let folder = room.path().join("folder");
    let store = room.path().join("store");
    sown(&folder, "dev_c", 1);
    std::fs::create_dir_all(store.join("dev_c")).unwrap();
    std::fs::copy(
        folder.join("dev_c").join("active.tisty"),
        store.join("dev_c").join("active.tisty"),
    )
    .unwrap();

    assert!(
        !ours_went_missing(&store.join("dev_c"), &folder.join("dev_c")),
        "the same count on both sides is nothing to take back"
    );
}

#[test]
fn a_history_brought_home_is_not_read_again_to_see_whether_it_should_go_back() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    let aside = tempfile::tempdir().unwrap();
    let aside = Some(aside.path());
    carry_leaning_on(&one.data, aside, &one.device, shared.path(), Way::Both, &[]).unwrap();

    let elsewhere = tempfile::tempdir().unwrap();
    sown(elsewhere.path(), "dev_c", 12_000);
    let theirs = shared.path().join(STORE).join("dev_c");
    std::fs::create_dir_all(&theirs).unwrap();
    for at in tisty_core::store::segments_in(&elsewhere.path().join("dev_c")).unwrap() {
        std::fs::copy(&at, theirs.join(at.file_name().unwrap())).unwrap();
        for beside in ["count", "sig"] {
            let one = at.with_extension(beside);
            if one.is_file() {
                std::fs::copy(&one, theirs.join(one.file_name().unwrap())).unwrap();
            }
        }
    }
    assert_eq!(tisty_core::store::segments_in(&theirs).unwrap().len(), 3);
    let whose = DeviceId("dev_c".into());
    let key = tisty_core::signing::mine(
        &tisty_core::Paths::new(
            elsewhere.path().to_path_buf(),
            elsewhere.path().join("config"),
        ),
        &whose,
    )
    .unwrap();
    assert!(tisty_core::vouched::confirm(
        &one.data,
        &whose,
        &tisty_core::signing::shown(&key)
    ));

    let _ = tisty_core::counting::from_now();
    let moved =
        carry_leaning_on(&one.data, aside, &one.device, shared.path(), Way::Both, &[]).unwrap();
    let opened = tisty_core::counting::from_now();

    assert_eq!(moved.brought, 3);
    assert_eq!(moved.sent, 0);
    assert!(
        opened <= 35,
        "bringing three segments home read {opened} files where 35 is what it takes: three for the segments, and one pass over a machine met for the first time to read what it says it signs with"
    );
}

#[test]
fn nothing_handed_on_keeps_what_was_already_compared() {
    let room = tempfile::tempdir().unwrap();
    let theirs = room.path().join("theirs");
    let mine = room.path().join("mine");
    a_segment(&theirs, "000001.tisty", "the same\n");
    a_segment(&mine, "000001.tisty", "the same\n");

    let mut alike = Alike::default();
    assert_eq!(alike.of("dev_b", &theirs, &mine).len(), 1);

    let done = alike
        .carried("dev_b", &theirs, &mine, Toward::Folder, false)
        .unwrap();
    assert_eq!(done, 0, "both sides hold the same bytes");

    std::fs::write(mine.join("000001.tisty"), "not any more\n").unwrap();
    assert_eq!(
        alike.of("dev_b", &theirs, &mine).len(),
        1,
        "nothing moved, so the round keeps the answer it already paid for"
    );
}

#[test]
fn only_the_segments_that_match_are_remembered() {
    let room = tempfile::tempdir().unwrap();
    let theirs = room.path().join("theirs");
    let mine = room.path().join("mine");
    a_segment(&theirs, "000001.tisty", "shared\n");
    a_segment(&theirs, "000002.tisty", "theirs alone\n");
    a_segment(&mine, "000001.tisty", "shared\n");
    a_segment(&mine, "000002.tisty", "mine alone\n");

    let mut alike = Alike::default();
    let known = alike.of("dev_b", &theirs, &mine);

    assert_eq!(known.len(), 1);
    assert!(known.contains(std::ffi::OsStr::new("000001.tisty")));
}

#[test]
fn an_empty_history_is_never_all_the_other_side_holds() {
    let room = tempfile::tempdir().unwrap();
    let theirs = room.path().join("theirs");
    let mine = room.path().join("mine");
    std::fs::create_dir_all(&theirs).unwrap();
    std::fs::create_dir_all(&mine).unwrap();

    let mut alike = Alike::default();
    assert!(!alike.settled("dev_b", &theirs, &mine, Toward::Home));
    assert!(
        !alike.settled("dev_c", &theirs, &mine, Toward::Folder),
        "holding nothing is not the same as holding everything they hold"
    );
}

#[test]
fn a_file_in_our_store_is_not_a_machine_to_hand_on() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();
    std::fs::write(one.store.join("notes.txt"), "not a machine\n").unwrap();

    let mut alike = Alike::default();
    let sent = hand_on(&one.store, &one.device, shared.path(), false, &mut alike).unwrap();

    assert_eq!(sent, 0);
    assert!(!shared.path().join(STORE).join("notes.txt").exists());
}

#[test]
fn a_forced_round_leaves_alike_histories_where_they_are() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();
    sown(&one.store, "dev_c", 1);

    let mut alike = Alike::default();
    let first = hand_on(&one.store, &one.device, shared.path(), true, &mut alike).unwrap();
    let mut alike = Alike::default();
    let second = hand_on(&one.store, &one.device, shared.path(), true, &mut alike).unwrap();

    assert_eq!(first, 1);
    assert_eq!(
        second, 0,
        "asking again sends our own history over, never one we only keep for somebody else"
    );
}

fn kept_for_another(one: &Machine, shared: &Path) -> (PathBuf, PathBuf) {
    carry(&one.data, &one.device, shared, Way::Push, &[]).unwrap();
    sown(&one.store, "dev_c", 1);
    let mut alike = Alike::default();
    hand_on(&one.store, &one.device, shared, false, &mut alike).unwrap();
    (one.store.join("dev_c"), shared.join(STORE).join("dev_c"))
}

#[test]
fn a_signature_missing_over_there_is_put_back_by_the_machine_that_keeps_the_history() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    let (mine, there) = kept_for_another(&one, shared.path());
    assert!(
        there.join("active.sig").is_file(),
        "the relay never sent it"
    );
    std::fs::remove_file(there.join("active.sig")).unwrap();

    let mut alike = Alike::default();
    let sent = hand_on(&one.store, &one.device, shared.path(), false, &mut alike).unwrap();

    assert_eq!(sent, 0, "no segment moved: only the signature was missing");
    assert_eq!(
        std::fs::read(there.join("active.sig")).unwrap(),
        std::fs::read(mine.join("active.sig")).unwrap(),
        "the signature was not put back"
    );
}

#[test]
fn a_signature_that_differs_over_there_is_never_replaced_by_a_relay() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    let (_, there) = kept_for_another(&one, shared.path());
    std::fs::write(there.join("active.sig"), b"the owner's newer one").unwrap();

    let mut alike = Alike::default();
    hand_on(&one.store, &one.device, shared.path(), false, &mut alike).unwrap();

    assert_eq!(
        std::fs::read(there.join("active.sig")).unwrap(),
        b"the owner's newer one",
        "a relay replaced what the owner wrote"
    );
}

#[test]
fn a_signature_is_not_put_back_beside_a_segment_that_is_not_the_one_we_hold() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();
    sown(&one.store, "dev_c", 1);
    sown(&shared.path().join(STORE), "dev_c", 2);
    let there = shared.path().join(STORE).join("dev_c");
    std::fs::remove_file(there.join("active.sig")).unwrap();

    let mut alike = Alike::default();
    hand_on(&one.store, &one.device, shared.path(), false, &mut alike).unwrap();

    assert!(
        !there.join("active.sig").exists(),
        "a signature was put beside bytes it does not answer for"
    );
}

#[test]
fn a_count_and_a_signature_missing_beside_a_closed_segment_are_both_put_back() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();
    let mine = one.store.join("dev_c");
    let there = shared.path().join(STORE).join("dev_c");
    for dir in [&mine, &there] {
        std::fs::create_dir_all(dir).unwrap();
        std::fs::write(dir.join("000001.tisty"), b"a closed segment\n").unwrap();
    }
    std::fs::write(mine.join("000001.sig"), b"its signature").unwrap();
    std::fs::write(mine.join("000001.count"), b"1").unwrap();

    let mut alike = Alike::default();
    let sent = hand_on(&one.store, &one.device, shared.path(), false, &mut alike).unwrap();

    assert_eq!(sent, 0, "no segment moved");
    assert_eq!(
        std::fs::read(there.join("000001.sig")).unwrap(),
        b"its signature"
    );
    assert_eq!(std::fs::read(there.join("000001.count")).unwrap(), b"1");
}

#[test]
fn copying_unless_there_fills_what_is_not_and_leaves_what_is() {
    let room = tempfile::tempdir().unwrap();
    let from = room.path().join("from");
    let absent = room.path().join("absent");
    let taken = room.path().join("taken");
    std::fs::write(&from, b"ours").unwrap();
    std::fs::write(&taken, b"theirs").unwrap();

    assert!(copy_unless_there(&from, &absent).unwrap());
    assert!(!copy_unless_there(&from, &taken).unwrap());

    assert_eq!(std::fs::read(&absent).unwrap(), b"ours");
    assert_eq!(std::fs::read(&taken).unwrap(), b"theirs");
    let left: Vec<_> = std::fs::read_dir(room.path())
        .unwrap()
        .filter_map(|one| one.ok())
        .map(|one| one.file_name())
        .collect();
    assert_eq!(left.len(), 3, "a part file was left behind: {left:?}");
}

#[test]
fn where_links_are_not_kept_a_name_is_claimed_before_it_is_filled() {
    let room = tempfile::tempdir().unwrap();
    let from = room.path().join("from");
    let absent = room.path().join("absent");
    let taken = room.path().join("taken");
    std::fs::write(&from, b"ours").unwrap();
    std::fs::write(&taken, b"theirs").unwrap();

    assert!(created_where_nothing_stands(&from, &absent).unwrap());
    assert!(!created_where_nothing_stands(&from, &taken).unwrap());

    assert_eq!(std::fs::read(&absent).unwrap(), b"ours");
    assert_eq!(std::fs::read(&taken).unwrap(), b"theirs");
}

#[test]
fn a_kind_of_sidecar_a_later_build_writes_is_put_back_like_the_rest() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    let (mine, there) = kept_for_another(&one, shared.path());
    std::fs::write(mine.join("active.later"), b"what a later build keeps").unwrap();

    let mut alike = Alike::default();
    hand_on(&one.store, &one.device, shared.path(), false, &mut alike).unwrap();

    assert_eq!(
        std::fs::read(there.join("active.later")).unwrap(),
        b"what a later build keeps"
    );
}

#[test]
fn a_signature_not_yet_brought_down_by_the_cloud_is_not_put_back_beside_its_placeholder() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    let (_, there) = kept_for_another(&one, shared.path());
    std::fs::remove_file(there.join("active.sig")).unwrap();
    std::fs::write(there.join(".active.sig.icloud"), b"placeholder").unwrap();

    let mut alike = Alike::default();
    hand_on(&one.store, &one.device, shared.path(), false, &mut alike).unwrap();

    assert!(
        !there.join("active.sig").exists(),
        "a real signature was put beside the placeholder of the owner's"
    );
}

#[test]
fn a_history_taken_back_is_counted_as_it_comes() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();
    let mine = one.store.join(&one.device);
    std::fs::remove_dir_all(&mine).unwrap();

    let moved = carry(&one.data, &one.device, shared.path(), Way::Pull, &[]).unwrap();

    assert_eq!(moved.brought, 1, "our own history came back uncounted");
    assert!(mine.join("active.tisty").is_file());
}

fn handed_on(paths: &tisty_core::paths::Paths, who: &str) -> Vec<String> {
    tisty_core::witness::recent(paths, 200)
        .into_iter()
        .filter(|line| line.contains("was holding for another was handed on") && line.contains(who))
        .collect()
}

#[test]
fn a_history_handed_on_is_written_down_and_a_round_that_moved_none_is_not() {
    let _alone = ALONE.lock().unwrap_or_else(|e| e.into_inner());
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();
    sown(&one.store, "dev_written_down", 1);

    let kept = tempfile::tempdir().unwrap();
    let paths = watching(kept.path());
    let mut alike = Alike::default();
    let sent = hand_on(&one.store, &one.device, shared.path(), false, &mut alike).unwrap();

    assert_eq!(sent, 1);
    assert_eq!(handed_on(&paths, "dev_written_down").len(), 1);

    let mut alike = Alike::default();
    hand_on(&one.store, &one.device, shared.path(), false, &mut alike).unwrap();

    assert_eq!(
        handed_on(&paths, "dev_written_down").len(),
        1,
        "the second round moved nothing, so it had nothing to say"
    );
}

#[test]
fn what_was_written_after_the_round_looked_still_goes_up() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    let mine = one.store.join(&one.device);
    let theirs = shared.path().join(STORE).join(&one.device);
    let mut alike = Alike::default();
    assert_eq!(
        alike.of(&one.device, &theirs, &mine).len(),
        1,
        "both sides hold the same segment when the round starts"
    );

    wrote(&one, "lo que se escribio a mitad de ronda".into());
    let sent = alike
        .carried(&one.device, &theirs, &mine, Toward::Folder, false)
        .unwrap();

    assert_eq!(
        sent, 1,
        "a segment somebody wrote to mid round is not alike any more"
    );
    assert_eq!(
        tisty_core::store::distinct_in(&theirs).unwrap(),
        tisty_core::store::distinct_in(&mine).unwrap(),
        "the folder is missing what was written while the round was looking elsewhere"
    );
}

fn half_a_copy(at: &Path, who: &str, ago: u64) -> PathBuf {
    let one = at.join(format!("active.{who}.0.part"));
    std::fs::write(&one, b"half of what another machine is writing").unwrap();
    let when = std::time::SystemTime::now() - std::time::Duration::from_secs(ago);
    std::fs::File::options()
        .write(true)
        .open(&one)
        .unwrap()
        .set_modified(when)
        .unwrap();
    one
}

#[test]
fn a_copy_another_machine_has_in_flight_outlives_our_round() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();
    let theirs = half_a_copy(
        &shared.path().join(STORE).join(&one.device),
        &std::process::id().to_string(),
        60,
    );

    wrote(&one, "algo mas".into());
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    assert!(
        theirs.is_file(),
        "two machines can be given one process number, and this round took a copy the other was making"
    );
}

#[test]
fn a_copy_nobody_came_back_for_is_swept_whoever_left_it() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();
    let theirs = half_a_copy(
        &shared.path().join(STORE).join(&one.device),
        "01ARZ3NDEKTSV4RRFFQ69G5FAV",
        25 * 60 * 60,
    );

    wrote(&one, "algo mas".into());
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    assert!(
        !theirs.exists(),
        "what a machine that is gone left in the folder stayed there for ever"
    );
}

#[test]
fn a_meeting_place_named_another_way_is_still_the_one_we_carried_to() {
    let one = machine("uno");
    let aside = tempfile::tempdir().unwrap();
    let aside = Some(aside.path());
    let shared = tempfile::tempdir().unwrap();
    carry_leaning_on(&one.data, aside, &one.device, shared.path(), Way::Both, &[]).unwrap();
    std::fs::remove_dir_all(shared.path().join(STORE)).unwrap();

    std::fs::create_dir_all(shared.path().join("a-way-round")).unwrap();
    let same = shared.path().join("a-way-round").join("..");
    let stopped = carry_leaning_on(&one.data, aside, &one.device, &same, Way::Both, &[]);

    assert!(
        matches!(stopped, Err(Trouble::Emptied(_))),
        "the same folder spelled another way was taken for a new one: {stopped:?}"
    );
}

#[test]
fn a_meeting_place_we_never_carried_to_is_taken_up_without_a_word() {
    let one = machine("uno");
    let aside = tempfile::tempdir().unwrap();
    let aside = Some(aside.path());
    let shared = tempfile::tempdir().unwrap();
    carry_leaning_on(&one.data, aside, &one.device, shared.path(), Way::Both, &[]).unwrap();

    let elsewhere = tempfile::tempdir().unwrap();
    carry_leaning_on(
        &one.data,
        aside,
        &one.device,
        elsewhere.path(),
        Way::Both,
        &[],
    )
    .unwrap();

    assert!(
        elsewhere.path().join(STORE).join(&one.device).is_dir(),
        "a folder we had never carried to was refused as if it had been emptied"
    );
}

#[test]
fn a_folder_that_answers_to_another_name_now_is_still_the_one_we_carried_to() {
    let one = machine("uno");
    let kept = tempfile::tempdir().unwrap();
    let aside = Some(kept.path());
    let shared = tempfile::tempdir().unwrap();
    carry_leaning_on(&one.data, aside, &one.device, shared.path(), Way::Both, &[]).unwrap();
    std::fs::remove_dir_all(shared.path().join(STORE)).unwrap();

    let mark = tisty_core::paths::told_apart(shared.path()).expect("the folder is there");
    let gone = shared.path().join("under-another-letter");
    std::fs::write(
        kept.path().join(super::place::CARRIED_TO),
        format!("{mark}\n{}", gone.display()),
    )
    .unwrap();

    let stopped = carry_leaning_on(&one.data, aside, &one.device, shared.path(), Way::Both, &[]);

    assert!(
        matches!(stopped, Err(Trouble::Emptied(_))),
        "the drive came back as another letter and the folder was taken for a new one: {stopped:?}"
    );
}

#[test]
fn a_note_of_where_we_carried_from_before_the_mark_is_still_read() {
    let one = machine("uno");
    let kept = tempfile::tempdir().unwrap();
    let aside = Some(kept.path());
    let shared = tempfile::tempdir().unwrap();
    carry_leaning_on(&one.data, aside, &one.device, shared.path(), Way::Both, &[]).unwrap();
    std::fs::remove_dir_all(shared.path().join(STORE)).unwrap();
    std::fs::write(
        kept.path().join(super::place::CARRIED_TO),
        format!("{}\n", shared.path().display()),
    )
    .unwrap();

    std::fs::create_dir_all(shared.path().join("a-way-round")).unwrap();
    let same = shared.path().join("a-way-round").join("..");
    let stopped = carry_leaning_on(&one.data, aside, &one.device, &same, Way::Both, &[]);

    assert!(
        matches!(stopped, Err(Trouble::Emptied(_))),
        "a note written before this machine knew how to tell folders apart stopped counting: {stopped:?}"
    );
}

#[test]
fn another_folder_that_took_the_letter_is_not_the_one_we_carried_to() {
    let one = machine("uno");
    let kept = tempfile::tempdir().unwrap();
    let aside = Some(kept.path());
    let shared = tempfile::tempdir().unwrap();
    carry_leaning_on(&one.data, aside, &one.device, shared.path(), Way::Both, &[]).unwrap();

    let elsewhere = tempfile::tempdir().unwrap();
    let theirs = tisty_core::paths::told_apart(elsewhere.path()).expect("it is there");
    std::fs::write(
        kept.path().join(super::place::CARRIED_TO),
        format!("{theirs}\n{}", shared.path().display()),
    )
    .unwrap();
    std::fs::remove_dir_all(shared.path().join(STORE)).unwrap();

    carry_leaning_on(&one.data, aside, &one.device, shared.path(), Way::Both, &[])
        .expect("another disk under the same name is a folder we have never carried to");

    assert!(shared.path().join(STORE).join(&one.device).is_dir());
}

#[test]
fn a_note_of_another_place_from_before_the_mark_leaves_this_one_alone() {
    let one = machine("uno");
    let kept = tempfile::tempdir().unwrap();
    let aside = Some(kept.path());
    let elsewhere = tempfile::tempdir().unwrap();
    std::fs::write(
        kept.path().join(super::place::CARRIED_TO),
        elsewhere.path().display().to_string(),
    )
    .unwrap();

    let shared = tempfile::tempdir().unwrap();
    carry_leaning_on(&one.data, aside, &one.device, shared.path(), Way::Both, &[])
        .expect("a folder the note never named is one we have never carried to");

    assert!(shared.path().join(STORE).join(&one.device).is_dir());
}

#[test]
fn a_body_that_arrived_empty_never_writes_over_the_one_we_have() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    filed(&one, "uno-0001", "# Notas\n\nlo que escribi\n");
    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

    let theirs = shared.path().join(PAPERS).join("uno-0001.md");
    std::fs::write(&theirs, b"").unwrap();

    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

    assert_eq!(
        std::fs::read_to_string(one.data.join(PAPERS).join("uno-0001.md")).unwrap(),
        "# Notas\n\nlo que escribi\n",
        "a failed download read as somebody emptying the document"
    );
}

#[test]
fn a_body_the_person_emptied_still_travels() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    filed(&one, "uno-0001", "# Notas\n\nlo que escribi\n");
    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

    wrote_body(&one.data.join(PAPERS), "uno-0001", "");
    let id = tisty_core::State::replay(&tisty_core::store::read_all(&one.store).unwrap())
        .docs
        .values()
        .find(|paper| paper.file == "uno-0001")
        .map(|paper| paper.id)
        .expect("the document is in the log");
    says(
        &one,
        Op::DocSaid {
            id,
            d: tisty_core::event::Said::of(""),
        },
    );

    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

    assert_eq!(
        std::fs::read_to_string(shared.path().join(PAPERS).join("uno-0001.md")).unwrap(),
        "",
        "the person emptied it on purpose and the folder kept the old one"
    );
}

#[test]
fn the_last_copy_here_is_kept_when_the_one_up_there_only_weighs_the_same() {
    let one = machine("dev_a");
    let big: Vec<u8> = (0..(tisty_core::attach::COPIED_UP_TO as usize + 1024))
        .map(|at| (at % 251) as u8)
        .collect();
    let heavy = planted(&one.data, "charla.mp4", &big);
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();
    assert!(shared.path().join(&heavy).is_file(), "it went up");

    let mut other = big.clone();
    other[7] ^= 0xff;
    std::fs::write(shared.path().join(&heavy), &other).unwrap();
    let carried = [(
        heavy.clone(),
        tisty_core::attach::printed(&big),
        big.len() as u64,
    )];

    let freed = super::let_go_of(
        &one.data,
        shared.path(),
        &carried,
        tisty_core::attach::COPIED_UP_TO,
    );

    assert_eq!(freed.0, 0);
    assert!(
        one.data.join(&heavy).is_file(),
        "the only copy left was let go of on the strength of its weight"
    );
}

#[test]
fn the_last_copy_here_goes_once_the_one_up_there_is_the_same_bytes() {
    let one = machine("dev_a");
    let big: Vec<u8> = (0..(tisty_core::attach::COPIED_UP_TO as usize + 1024))
        .map(|at| (at % 251) as u8)
        .collect();
    let heavy = planted(&one.data, "charla.mp4", &big);
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();
    let carried = [(
        heavy.clone(),
        tisty_core::attach::printed(&big),
        big.len() as u64,
    )];

    let freed = super::let_go_of(
        &one.data,
        shared.path(),
        &carried,
        tisty_core::attach::COPIED_UP_TO,
    );

    assert_eq!(freed.0, big.len() as u64);
    assert_eq!(freed.1.len(), 1, "what it let go of was not reported");
    assert!(!one.data.join(&heavy).exists());
}

#[test]
fn a_body_the_log_does_not_answer_for_is_left_for_the_person_to_decide() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    let body = "# Notas\n\nlo que escribi\n";
    filed(&one, "uno-0001", body);
    let id = tisty_core::State::replay(&tisty_core::store::read_all(&one.store).unwrap())
        .docs
        .values()
        .find(|paper| paper.file == "uno-0001")
        .map(|paper| paper.id)
        .expect("the document is in the log");
    says(
        &one,
        Op::DocSaid {
            id,
            d: tisty_core::event::Said::of(body),
        },
    );
    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

    std::fs::write(
        shared.path().join(PAPERS).join("uno-0001.md"),
        b"algo que nadie escribio\n",
    )
    .unwrap();
    waited_long(&one.data, "uno-0001");

    let moved = carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

    assert!(
        moved.undecided_ids().contains(&"uno-0001".to_string()),
        "a body nobody wrote down was taken in without a word"
    );
    assert!(
        moved.astray.is_empty(),
        "a document the person can still decide was filed as one nothing can be done about"
    );
    assert_eq!(
        std::fs::read_to_string(one.data.join(PAPERS).join("uno-0001.md")).unwrap(),
        body
    );
    assert_eq!(
        std::fs::read_to_string(shared.path().join(PAPERS).join("uno-0001.md")).unwrap(),
        "algo que nadie escribio\n",
        "the side the person has not seen yet was written over"
    );
}

fn waited_long(data: &Path, id: &str) {
    std::fs::write(data.join("awaited"), format!("{id} 0\n")).unwrap();
}

#[test]
fn a_body_that_lands_ahead_of_its_history_waits_for_it_and_then_comes_in() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    let body = "# Notas\n\nlo que escribi\n";
    filed(&one, "uno-0001", body);
    let id = tisty_core::State::replay(&tisty_core::store::read_all(&one.store).unwrap())
        .docs
        .values()
        .find(|paper| paper.file == "uno-0001")
        .map(|paper| paper.id)
        .expect("the document is in the log");
    says(
        &one,
        Op::DocSaid {
            id,
            d: tisty_core::event::Said::of(body),
        },
    );
    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();
    let theirs = "# Notas\n\nlo que escribio el otro\n";
    std::fs::write(shared.path().join(PAPERS).join("uno-0001.md"), theirs).unwrap();

    let early = carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

    assert!(
        early.undecided_ids().is_empty(),
        "the person was asked about a body whose history was still on its way"
    );
    assert!(early.coming.contains(&"uno-0001".to_string()), "{early:?}");
    assert_eq!(
        std::fs::read_to_string(one.data.join(PAPERS).join("uno-0001.md")).unwrap(),
        body
    );

    says(
        &one,
        Op::DocSaid {
            id,
            d: tisty_core::event::Said::of(theirs),
        },
    );
    let landed = carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

    assert!(landed.undecided_ids().is_empty(), "{landed:?}");
    assert_eq!(
        std::fs::read_to_string(one.data.join(PAPERS).join("uno-0001.md")).unwrap(),
        theirs,
        "the body did not come in once its history arrived"
    );
    assert!(
        !one.data.join("awaited").exists(),
        "what arrived is still remembered as awaited"
    );
}

#[test]
fn a_signature_travels_with_the_segment_it_answers_for() {
    let one = machine("dev_a");
    let paths = tisty_core::Paths::new(one.data.clone(), one.data.join("config"));
    let who = DeviceId(one.device.clone());
    let key = tisty_core::signing::mine(&paths, &who).expect("a key");
    let mut held = Store::open(&one.store, who.clone())
        .unwrap()
        .signing_with(Some(key.clone()));
    held.append(Op::TaskAdd {
        id: Ulid::generate(),
        d: tisty_core::event::TaskAdd::new("chase the invoice", "a0"),
    })
    .unwrap();
    drop(held);

    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    let there = shared.path().join("store").join(&one.device);
    let said = std::fs::read_to_string(there.join("active.sig"))
        .expect("the signature stayed behind, so nothing over there can answer for the segment");
    let tip = tisty_core::signing::holds(
        &key.verifying_key(),
        &tisty_core::signing::About {
            device: &one.device,
            segment: "active.tisty",
        },
        &said,
    )
    .covers()
    .expect("it does not answer");
    let arrived = std::fs::read(there.join("active.tisty")).unwrap();
    assert_eq!(
        tip.tip,
        tisty_core::signing::tip_of(tisty_core::signing::NOTHING_BEFORE, &arrived),
        "what arrived is not what the signature was made over"
    );
    assert_eq!(
        tip.at,
        arrived.len() as u64,
        "the signature answers for a different number of bytes than arrived"
    );
}

fn a_segment_with_its_siblings(dir: &std::path::Path, count: &str, sig: &str) {
    std::fs::create_dir_all(dir).unwrap();
    std::fs::write(dir.join("000001.tisty"), b"one line").unwrap();
    std::fs::write(dir.join("000001.count"), count.as_bytes()).unwrap();
    std::fs::write(dir.join("000001.sig"), sig.as_bytes()).unwrap();
}

#[test]
fn a_source_caught_without_a_count_does_not_take_the_one_already_there() {
    let room = tempfile::tempdir().unwrap();
    let from = room.path().join("from");
    let into = room.path().join("into");
    a_segment_with_its_siblings(&from, "1", "a signature");
    a_segment_with_its_siblings(&into, "1", "a signature");
    std::fs::remove_file(from.join("000001.count")).unwrap();

    crate::segments::copy_segments(&from, &into, false, &Default::default()).unwrap();

    assert!(
        into.join("000001.count").is_file(),
        "the guard against a truncated segment was removed at the far end"
    );
}

/// Whoever can write in that folder could delete a signature, so taking ours away for being
/// gone over there is the downgrade itself. A stale one answers for the wrong number of bytes,
/// which reads as a history that cannot be read through — and that heals when it is signed again.
#[test]
fn a_signature_is_never_taken_away_for_being_gone_at_the_far_end() {
    let room = tempfile::tempdir().unwrap();
    let from = room.path().join("from");
    let into = room.path().join("into");
    a_segment_with_its_siblings(&from, "1", "a signature");
    a_segment_with_its_siblings(&into, "1", "a signature");
    std::fs::remove_file(from.join("000001.sig")).unwrap();
    std::fs::write(from.join("000001.tisty"), b"another line entirely").unwrap();

    crate::segments::copy_segments(&from, &into, true, &Default::default()).unwrap();

    assert!(
        into.join("000001.sig").is_file(),
        "a signature went because somebody stopped carrying one, which is the whole attack"
    );
}

#[test]
fn a_signature_stays_when_the_segment_under_it_never_moved() {
    let room = tempfile::tempdir().unwrap();
    let from = room.path().join("from");
    let into = room.path().join("into");
    a_segment_with_its_siblings(&from, "1", "a signature");
    a_segment_with_its_siblings(&into, "1", "a signature");
    std::fs::remove_file(from.join("000001.sig")).unwrap();

    crate::segments::copy_segments(&from, &into, false, &Default::default()).unwrap();

    assert!(
        into.join("000001.sig").is_file(),
        "the only signature that segment will ever have was thrown away for nothing"
    );
}

#[test]
fn a_carry_that_moved_only_a_signature_is_not_reported_as_nothing() {
    let room = tempfile::tempdir().unwrap();
    let from = room.path().join("from");
    let into = room.path().join("into");
    a_segment_with_its_siblings(&from, "1", "a newer signature");
    a_segment_with_its_siblings(&into, "1", "a signature");

    let (done, beside) =
        crate::segments::copy_segments(&from, &into, false, &Default::default()).unwrap();

    assert_eq!(
        std::fs::read_to_string(into.join("000001.sig")).unwrap(),
        "a newer signature"
    );
    assert_eq!(done, 0, "a signature was counted as a segment brought home");
    assert!(
        beside > 0,
        "a signature moved and the round called it a no-op"
    );
}

#[test]
fn a_sibling_this_build_has_no_name_for_travels_with_its_segment() {
    let room = tempfile::tempdir().unwrap();
    let from = room.path().join("from");
    let into = room.path().join("into");
    a_segment_with_its_siblings(&from, "1", "a signature");
    std::fs::write(from.join("000001.whatever"), b"a later build wrote this").unwrap();
    std::fs::write(from.join("active.tisty"), b"still being written").unwrap();
    std::fs::write(from.join("active.torn"), b"a mend this machine made").unwrap();
    std::fs::write(from.join(".lock"), b"").unwrap();

    crate::segments::copy_segments(&from, &into, false, &Default::default()).unwrap();

    assert_eq!(
        std::fs::read_to_string(into.join("000001.whatever"))
            .ok()
            .as_deref(),
        Some("a later build wrote this"),
        "a sibling with no name this build knows was left behind"
    );
    assert!(
        !into.join("active.torn").exists(),
        "a mend belongs to the machine that made it and went to the far side"
    );
    assert!(!into.join(".lock").exists(), "the lock travelled");
}

#[test]
fn a_round_leaves_the_meeting_place_saying_what_shape_it_is_in() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();

    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

    let said = std::fs::read_to_string(shared.path().join("tisty.toml"))
        .expect("a round wrote a whole history and never said what shape it left");
    assert!(said.contains("shape = 1"), "{said}");
    assert!(said.contains("store"), "{said}");
}

#[test]
fn a_meeting_place_arranged_by_a_build_that_knows_more_stops_the_round() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();
    std::fs::write(
        shared.path().join("tisty.toml"),
        "shape = 99
",
    )
    .unwrap();

    let two = machine("dos");
    let stopped = carry(&two.data, &two.device, shared.path(), Way::Both, &[]);

    assert!(
        matches!(stopped, Err(Trouble::Shape(_))),
        "it wrote into a folder it does not understand: {stopped:?}"
    );
}

#[test]
fn a_history_changed_under_its_signature_never_becomes_state_on_the_next_machine() {
    let one = machine("dev_a");
    let paths = tisty_core::Paths::new(one.data.clone(), one.data.join("config"));
    let who = DeviceId(one.device.clone());
    let key = tisty_core::signing::mine(&paths, &who).expect("a key");
    let mut held = Store::open(&one.store, who.clone())
        .unwrap()
        .signing_with(Some(key));
    held.append(Op::DeviceJoin {
        d: who.clone(),
        k: Some(tisty_core::DeviceKind::Machine),
        p: tisty_core::signing::mine(&paths, &who)
            .as_ref()
            .map(tisty_core::signing::shown),
    })
    .unwrap();
    held.append(Op::TaskAdd {
        id: Ulid::generate(),
        d: tisty_core::event::TaskAdd::new("chase the invoice", "a0"),
    })
    .unwrap();
    drop(held);

    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    let two = blank("dev_b");
    carry(&two.data, &two.device, shared.path(), Way::Both, &[]).unwrap();
    assert!(
        two.store.join(&one.device).join("active.tisty").is_file(),
        "the first round brought nothing, so the second proves nothing"
    );

    let theirs = shared.path().join(STORE).join(&one.device);
    let whole = std::fs::read_to_string(theirs.join("active.tisty")).unwrap();
    std::fs::write(
        theirs.join("active.tisty"),
        whole.replace("chase the invoice", "chase the invoicf"),
    )
    .unwrap();

    let after = carry(&two.data, &two.device, shared.path(), Way::Both, &[]).unwrap();

    assert_eq!(
        after.disowned,
        vec![one.device.clone()],
        "a history changed under its own signature was not called out"
    );
    assert!(
        !std::fs::read_to_string(two.store.join(&one.device).join("active.tisty"))
            .unwrap()
            .contains("invoicf"),
        "what does not answer for itself became state anyway"
    );
}

/// Our own name is the one worth wearing: a line put into our directory in the shared folder
/// comes home under it, and the next thing we write would sign it as ours.
#[test]
fn a_line_put_into_our_own_history_over_there_does_not_come_home() {
    let one = machine("dev_a");
    let paths = tisty_core::Paths::new(one.data.clone(), one.data.join("config"));
    let who = DeviceId(one.device.clone());
    let key = tisty_core::signing::mine(&paths, &who).expect("a key");
    let mut held = Store::open(&one.store, who.clone())
        .unwrap()
        .signing_with(Some(key));
    held.append(Op::DeviceJoin {
        d: who.clone(),
        k: Some(tisty_core::DeviceKind::Machine),
        p: tisty_core::signing::mine(&paths, &who)
            .as_ref()
            .map(tisty_core::signing::shown),
    })
    .unwrap();
    held.append(Op::TaskAdd {
        id: Ulid::generate(),
        d: tisty_core::event::TaskAdd::new("chase the invoice", "a0"),
    })
    .unwrap();
    drop(held);

    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    let theirs = shared.path().join(STORE).join(&one.device);
    let whole = std::fs::read_to_string(theirs.join("active.tisty")).unwrap();
    let forged = whole
        .lines()
        .last()
        .unwrap()
        .replace("chase the", "pay the");
    std::fs::write(
        theirs.join("active.tisty"),
        format!(
            "{whole}{forged}
"
        ),
    )
    .unwrap();
    std::fs::remove_dir_all(one.store.join(&one.device)).unwrap();

    let after = carry(&one.data, &one.device, shared.path(), Way::Pull, &[]).unwrap();

    let home = one.store.join(&one.device).join("active.tisty");
    let came = std::fs::read_to_string(&home).unwrap_or_default();
    assert!(
        !came.contains("pay the"),
        "a line somebody put under our own name came home:
{came}"
    );
    assert!(
        !after.disowned.is_empty() || !after.unreadable.is_empty(),
        "it came back from the folder without a word about why not"
    );
}

/// What a machine published is its own word; what the person answered for is this machine's.
/// Once confirmed, a history has to answer to that key and no other.
#[test]
fn a_history_signed_with_another_key_than_the_one_confirmed_does_not_come_home() {
    let one = machine("dev_a");
    let paths = tisty_core::Paths::new(one.data.clone(), one.data.join("config"));
    let who = DeviceId(one.device.clone());
    let key = tisty_core::signing::mine(&paths, &who).expect("a key");
    let mut held = Store::open(&one.store, who.clone())
        .unwrap()
        .signing_with(Some(key.clone()));
    held.append(Op::TaskAdd {
        id: Ulid::generate(),
        d: tisty_core::event::TaskAdd::new("chase the invoice", "a0"),
    })
    .unwrap();
    drop(held);

    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    let two = blank("dev_b");
    std::fs::create_dir_all(&two.data).unwrap();
    let elsewhere = tisty_core::Paths::new(two.data.clone(), two.data.join("config"));
    let other = tisty_core::signing::shown(
        &tisty_core::signing::mine(&elsewhere, &DeviceId("dev_c".into())).unwrap(),
    );
    assert!(
        tisty_core::vouched::confirm(&two.data, &who, &other),
        "the second machine could not answer for a key"
    );

    let after = carry(&two.data, &two.device, shared.path(), Way::Both, &[]).unwrap();

    assert_eq!(
        after.disowned,
        vec![one.device.clone()],
        "a history signed with a key nobody answered for was taken in"
    );
}

/// The cheapest hand there is: a history that answered for itself once, then arrives with every
/// signature removed. Reading that as a history from before signing hands the forger the folder.
#[test]
fn a_history_stripped_of_every_signature_does_not_come_home() {
    let one = machine("dev_a");
    let paths = tisty_core::Paths::new(one.data.clone(), one.data.join("config"));
    let who = DeviceId(one.device.clone());
    let key = tisty_core::signing::mine(&paths, &who).expect("a key");
    let mut held = Store::open(&one.store, who.clone())
        .unwrap()
        .signing_with(Some(key.clone()));
    held.append(Op::TaskAdd {
        id: Ulid::generate(),
        d: tisty_core::event::TaskAdd::new("chase the invoice", "a0"),
    })
    .unwrap();
    drop(held);

    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    let two = blank("dev_b");
    std::fs::create_dir_all(&two.data).unwrap();
    assert!(
        tisty_core::vouched::confirm(&two.data, &who, &tisty_core::signing::shown(&key)),
        "the second machine could not answer for the key"
    );
    let first = carry(&two.data, &two.device, shared.path(), Way::Both, &[]).unwrap();
    assert_eq!(
        first.disowned,
        Vec::<String>::new(),
        "a true history was refused"
    );
    assert!(
        home_of(&two, &one.device).contains("chase the invoice"),
        "the history did not come home at all"
    );

    let there = shared.path().join(STORE).join(&one.device);
    let whole = std::fs::read_to_string(there.join("active.tisty")).unwrap();
    std::fs::write(
        there.join("active.tisty"),
        whole.replace("chase the invoice", "chase the other one"),
    )
    .unwrap();
    for found in std::fs::read_dir(&there)
        .unwrap()
        .filter_map(|one| one.ok())
    {
        if found.path().extension().is_some_and(|one| one == "sig") {
            std::fs::remove_file(found.path()).unwrap();
        }
    }

    let after = carry(&two.data, &two.device, shared.path(), Way::Both, &[]).unwrap();

    assert!(
        !home_of(&two, &one.device).contains("chase the other one"),
        "a forged line came home once every signature beside it was removed"
    );
    assert!(
        !after.disowned.is_empty() || !after.unreadable.is_empty(),
        "the round said nothing at all about a history it could not answer for"
    );
}

/// Our own log not reading is no reason to stop checking everybody else: it is the one moment a
/// forger would most like us to take their word for it.
#[test]
fn a_log_of_our_own_that_will_not_read_does_not_open_the_folder() {
    let one = machine("dev_a");
    let paths = tisty_core::Paths::new(one.data.clone(), one.data.join("config"));
    let who = DeviceId(one.device.clone());
    let key = tisty_core::signing::mine(&paths, &who).expect("a key");
    let mut held = Store::open(&one.store, who.clone())
        .unwrap()
        .signing_with(Some(key));
    held.append(Op::TaskAdd {
        id: Ulid::generate(),
        d: tisty_core::event::TaskAdd::new("chase the invoice", "a0"),
    })
    .unwrap();
    drop(held);

    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    let two = blank("dev_b");
    std::fs::create_dir_all(&two.data).unwrap();
    carry(&two.data, &two.device, shared.path(), Way::Both, &[]).unwrap();

    let there = shared.path().join(STORE).join(&one.device);
    let whole = std::fs::read_to_string(there.join("active.tisty")).unwrap();
    std::fs::write(
        there.join("active.tisty"),
        whole.replace("chase the invoice", "chase the other one"),
    )
    .unwrap();

    let ours = two.store.join("dev_junk");
    std::fs::create_dir_all(&ours).unwrap();
    std::fs::write(
        ours.join("active.tisty"),
        b"{\"v\":99,\"what\":\"newer\"}
",
    )
    .unwrap();

    let _ = carry(&two.data, &two.device, shared.path(), Way::Both, &[]);

    assert!(
        !home_of(&two, &one.device).contains("chase the other one"),
        "a forged line came home while this machine's own log would not read"
    );
}

/// A folder stripped of every signature before anybody ever saw it signed leaves no memo to
/// remember by. What a person answered for is the demand itself, and it outlives any memo.
#[test]
fn a_key_somebody_answered_for_is_demanded_even_on_the_first_round() {
    let one = machine("dev_a");
    let paths = tisty_core::Paths::new(one.data.clone(), one.data.join("config"));
    let who = DeviceId(one.device.clone());
    let key = tisty_core::signing::mine(&paths, &who).expect("a key");
    let mut held = Store::open(&one.store, who.clone())
        .unwrap()
        .signing_with(Some(key.clone()));
    held.append(Op::TaskAdd {
        id: Ulid::generate(),
        d: tisty_core::event::TaskAdd::new("chase the invoice", "a0"),
    })
    .unwrap();
    drop(held);

    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    let there = shared.path().join(STORE).join(&one.device);
    for found in std::fs::read_dir(&there)
        .unwrap()
        .filter_map(|one| one.ok())
    {
        if found.path().extension().is_some_and(|one| one == "sig") {
            std::fs::remove_file(found.path()).unwrap();
        }
    }

    let two = blank("dev_b");
    std::fs::create_dir_all(&two.data).unwrap();
    assert!(
        tisty_core::vouched::confirm(&two.data, &who, &tisty_core::signing::shown(&key)),
        "the second machine could not answer for the key"
    );

    let after = carry(&two.data, &two.device, shared.path(), Way::Both, &[]).unwrap();

    assert_eq!(
        after.disowned,
        vec![one.device.clone()],
        "a history with no signature at all came in under a key somebody answered for"
    );
    assert!(
        !home_of(&two, &one.device).contains("chase the invoice"),
        "it was taken in anyway"
    );
}

/// The window cannot work this out from the two keys: the log keeps the first one a machine
/// published and never another, so they agree while the folder is being refused. The round leaves
/// its own verdict where the window can read it, and takes it back when the history comes home.
#[test]
fn what_a_round_turned_away_is_left_where_the_window_can_read_it() {
    let one = machine("dev_a");
    let paths = tisty_core::Paths::new(one.data.clone(), one.data.join("config"));
    let who = DeviceId(one.device.clone());
    let key = tisty_core::signing::mine(&paths, &who).expect("a key");
    let mut held = Store::open(&one.store, who.clone())
        .unwrap()
        .signing_with(Some(key.clone()));
    held.append(Op::TaskAdd {
        id: Ulid::generate(),
        d: tisty_core::event::TaskAdd::new("chase the invoice", "a0"),
    })
    .unwrap();
    drop(held);

    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    let two = blank("dev_b");
    std::fs::create_dir_all(&two.data).unwrap();
    let elsewhere = tisty_core::Paths::new(two.data.clone(), two.data.join("config"));
    let other = tisty_core::signing::shown(
        &tisty_core::signing::mine(&elsewhere, &DeviceId("dev_c".into())).unwrap(),
    );
    assert!(tisty_core::vouched::confirm(&two.data, &who, &other));

    carry(&two.data, &two.device, shared.path(), Way::Both, &[]).unwrap();

    assert_eq!(
        turned::of(&two.data).get(&one.device),
        Some(&turned::Away::Disowned),
        "the round turned a history away and left nothing the window could say so with"
    );

    std::fs::remove_file(two.data.join(".keys-confirmed")).unwrap();
    carry(&two.data, &two.device, shared.path(), Way::Both, &[]).unwrap();

    assert_eq!(
        turned::of(&two.data).get(&one.device),
        None,
        "the history came home and the window would still be warning about it"
    );
}

#[test]
fn a_machine_that_published_a_key_owes_a_signature_nobody_confirmed() {
    let one = machine("dev_a");
    let paths = tisty_core::Paths::new(one.data.clone(), one.data.join("config"));
    let who = DeviceId(one.device.clone());
    let key = tisty_core::signing::mine(&paths, &who).expect("a key");
    let mut held = Store::open(&one.store, who.clone())
        .unwrap()
        .signing_with(Some(key.clone()));
    held.append(Op::DeviceKey {
        d: who.clone(),
        p: tisty_core::signing::shown(&key),
    })
    .unwrap();
    held.append(Op::TaskAdd {
        id: Ulid::generate(),
        d: tisty_core::event::TaskAdd::new("chase the invoice", "a0"),
    })
    .unwrap();
    drop(held);

    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    let two = blank("dev_b");
    std::fs::create_dir_all(&two.data).unwrap();
    let first = carry(&two.data, &two.device, shared.path(), Way::Both, &[]).unwrap();
    assert!(
        home_of(&two, &one.device).contains("chase the invoice"),
        "a true signed history did not come home"
    );
    assert_eq!(first.disowned, Vec::<String>::new());

    let there = shared.path().join(STORE).join(&one.device);
    let whole = std::fs::read_to_string(there.join("active.tisty")).unwrap();
    std::fs::write(
        there.join("active.tisty"),
        whole.replace("chase the invoice", "chase the other one"),
    )
    .unwrap();
    for found in std::fs::read_dir(&there)
        .unwrap()
        .filter_map(|one| one.ok())
    {
        if found.path().extension().is_some_and(|one| one == "sig") {
            std::fs::remove_file(found.path()).unwrap();
        }
    }
    let _ = std::fs::remove_file(two.data.join(".verified-to"));
    assert!(
        !two.data.join(".verified-to").is_file(),
        "the latch would carry this round and not the key the machine published"
    );

    let after = carry(&two.data, &two.device, shared.path(), Way::Both, &[]).unwrap();

    assert_eq!(
        after.disowned,
        vec![one.device.clone()],
        "a machine that published a key was taken on trust once its signatures were removed"
    );
    assert!(
        !home_of(&two, &one.device).contains("chase the other one"),
        "the forged line came home"
    );
}

#[test]
fn a_history_from_before_the_fence_is_not_a_signature_taken_away() {
    let one = machine("dev_a");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    let before = tempfile::tempdir().unwrap();
    let there = before.path().join(STORE).join(&one.device);
    std::fs::create_dir_all(&there).unwrap();
    let whole = std::fs::read_to_string(
        shared
            .path()
            .join(STORE)
            .join(&one.device)
            .join("active.tisty"),
    )
    .unwrap();
    let older = whole.replace(
        &format!("\"v\":{}", tisty_core::event::SCHEMA_VERSION),
        &format!("\"v\":{}", tisty_core::event::SIGNED_FROM - 1),
    );
    assert_ne!(older, whole, "the history was not written at this schema");
    std::fs::write(there.join("active.tisty"), &older).unwrap();
    std::fs::copy(
        shared.path().join(STORE).join(tisty_core::store::MARKER),
        before.path().join(STORE).join(tisty_core::store::MARKER),
    )
    .unwrap();

    let two = blank("dev_b");
    std::fs::create_dir_all(&two.data).unwrap();
    let after = carry(&two.data, &two.device, before.path(), Way::Pull, &[]).unwrap();

    assert_eq!(
        after.disowned,
        Vec::<String>::new(),
        "a history from before the fence owes a signature it never could have carried"
    );
    assert!(
        home_of(&two, &one.device).contains("lo de dev_a"),
        "it did not come home"
    );
}

#[test]
fn a_segment_torn_to_hide_what_a_machine_signs_with_brings_nothing_home() {
    let one = machine("dev_a");
    let paths = tisty_core::Paths::new(one.data.clone(), one.data.join("config"));
    let who = DeviceId(one.device.clone());
    let key = tisty_core::signing::mine(&paths, &who).expect("a key");
    let mut held = Store::open(&one.store, who.clone())
        .unwrap()
        .signing_with(Some(key.clone()));
    held.append(Op::DeviceKey {
        d: who.clone(),
        p: tisty_core::signing::shown(&key),
    })
    .unwrap();
    held.append(Op::TaskAdd {
        id: Ulid::generate(),
        d: tisty_core::event::TaskAdd::new("chase the invoice", "a0"),
    })
    .unwrap();
    drop(held);

    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    let there = shared.path().join(STORE).join(&one.device);
    let whole = std::fs::read_to_string(there.join("active.tisty")).unwrap();
    assert!(whole.contains("device.key"));
    let rest = whole
        .lines()
        .filter(|one| !one.contains("device.key"))
        .collect::<Vec<&str>>()
        .join("\n");
    std::fs::write(
        there.join("active.tisty"),
        format!("{rest}\n").replace("chase the invoice", "chase the other one"),
    )
    .unwrap();
    for found in std::fs::read_dir(&there)
        .unwrap()
        .filter_map(|one| one.ok())
    {
        if found
            .path()
            .extension()
            .is_some_and(|one| one == "sig" || one == "count")
        {
            std::fs::remove_file(found.path()).unwrap();
        }
    }
    std::fs::write(there.join("000001.tisty"), b"not a line of anything\n").unwrap();

    let two = blank("dev_b");
    std::fs::create_dir_all(&two.data).unwrap();
    let after = carry(&two.data, &two.device, shared.path(), Way::Both, &[]);

    assert!(
        !home_of(&two, &one.device).contains("chase the other one"),
        "tearing the segment that says what the machine signs with let the rest come home"
    );

    let after = after.expect("the round broke instead of turning one history away");
    assert_eq!(
        after.disowned,
        vec![one.device.clone()],
        "the round read a history through to nothing and said nothing about it"
    );
    assert_eq!(after.brought, 0);
}

#[test]
fn a_machine_nobody_answered_for_waits_from_the_first_time_it_is_seen() {
    let one = machine("dev_a");
    let paths = tisty_core::Paths::new(one.data.clone(), one.data.join("config"));
    let who = DeviceId(one.device.clone());
    let key = tisty_core::signing::mine(&paths, &who).expect("a key");
    let mut held = Store::open(&one.store, who.clone())
        .unwrap()
        .signing_with(Some(key.clone()));
    held.append(Op::DeviceKey {
        d: who.clone(),
        p: tisty_core::signing::shown(&key),
    })
    .unwrap();
    held.append(Op::TaskAdd {
        id: Ulid::generate(),
        d: tisty_core::event::TaskAdd::new("chase the invoice", "a0"),
    })
    .unwrap();
    drop(held);

    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    let two = blank("dev_b");
    std::fs::create_dir_all(&two.data).unwrap();
    assert!(tisty_core::vouched::confirm(
        &two.data,
        &DeviceId("dev_c".into()),
        &tisty_core::signing::shown(
            &tisty_core::signing::mine(
                &tisty_core::Paths::new(two.data.clone(), two.data.join("config")),
                &DeviceId("dev_c".into())
            )
            .unwrap()
        )
    ));
    let waiting = carry(&two.data, &two.device, shared.path(), Way::Both, &[]).unwrap();

    assert_eq!(
        waiting.unconfirmed,
        vec![one.device.clone()],
        "a machine nobody answered for came in on sight"
    );
    assert!(
        !home_of(&two, &one.device).contains("chase the invoice"),
        "its history came home before anybody answered for its key"
    );
    assert_eq!(
        turned::of(&two.data).get(&one.device),
        Some(&turned::Away::Unconfirmed),
        "the window has nothing to say it is waiting with"
    );

    assert!(tisty_core::vouched::confirm(
        &two.data,
        &who,
        &tisty_core::signing::shown(&key)
    ));
    let after = carry(&two.data, &two.device, shared.path(), Way::Both, &[]).unwrap();

    assert!(after.unconfirmed.is_empty());
    assert!(
        home_of(&two, &one.device).contains("chase the invoice"),
        "answering for the key did not let the history in"
    );
    assert_eq!(turned::of(&two.data).get(&one.device), None);
}

#[test]
fn the_first_folder_a_machine_ever_reaches_is_taken_up_whole() {
    let one = machine("dev_a");
    let paths = tisty_core::Paths::new(one.data.clone(), one.data.join("config"));
    let who = DeviceId(one.device.clone());
    let key = tisty_core::signing::mine(&paths, &who).expect("a key");
    let mut held = Store::open(&one.store, who.clone())
        .unwrap()
        .signing_with(Some(key.clone()));
    held.append(Op::DeviceKey {
        d: who.clone(),
        p: tisty_core::signing::shown(&key),
    })
    .unwrap();
    held.append(Op::TaskAdd {
        id: Ulid::generate(),
        d: tisty_core::event::TaskAdd::new("chase the invoice", "a0"),
    })
    .unwrap();
    drop(held);
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    let two = blank("dev_b");
    std::fs::create_dir_all(&two.data).unwrap();
    let first = carry(&two.data, &two.device, shared.path(), Way::Both, &[]).unwrap();

    assert!(
        first.unconfirmed.is_empty(),
        "a machine with nothing confirmed yet cannot answer for anybody, so it would never get in"
    );
    assert!(home_of(&two, &one.device).contains("chase the invoice"));
    assert_eq!(
        tisty_core::vouched::confirmed(&two.data, &who).map(|one| one.key),
        Some(tisty_core::signing::shown(&key)),
        "taking the folder up did not answer for the key it was taken up with"
    );
}

#[test]
fn a_machine_that_shows_up_later_in_a_folder_we_already_use_waits_too() {
    let one = machine("dev_a");
    let kept = tempfile::tempdir().unwrap();
    let aside = Some(kept.path());
    let shared = tempfile::tempdir().unwrap();
    carry_leaning_on(&one.data, aside, &one.device, shared.path(), Way::Both, &[]).unwrap();

    let two = machine("dev_b");
    let theirs = tisty_core::Paths::new(two.data.clone(), two.data.join("config"));
    let who = DeviceId(two.device.clone());
    let key = tisty_core::signing::mine(&theirs, &who).expect("a key");
    says(
        &two,
        Op::DeviceKey {
            d: who.clone(),
            p: tisty_core::signing::shown(&key),
        },
    );
    let there = shared.path().join(STORE).join(&two.device);
    std::fs::create_dir_all(&there).unwrap();
    for found in std::fs::read_dir(two.store.join(&two.device)).unwrap() {
        let found = found.unwrap().path();
        std::fs::copy(&found, there.join(found.file_name().unwrap())).unwrap();
    }

    let after =
        carry_leaning_on(&one.data, aside, &one.device, shared.path(), Way::Both, &[]).unwrap();

    assert_eq!(
        after.unconfirmed,
        vec![two.device.clone()],
        "a directory that turned up in a folder this machine has been using was taken on sight"
    );
    assert!(
        !home_of(&one, &two.device).contains("lo de dev_b"),
        "its history came home before anybody answered for its key"
    );
}

#[test]
fn a_machine_from_before_signing_is_not_left_waiting_for_a_key_it_never_had() {
    let one = machine("dev_a");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();

    let two = blank("dev_b");
    std::fs::create_dir_all(&two.data).unwrap();
    let after = carry(&two.data, &two.device, shared.path(), Way::Both, &[]).unwrap();

    assert!(
        after.unconfirmed.is_empty(),
        "a machine that never said it signs was left waiting for a key nobody can read out"
    );
    assert!(home_of(&two, &one.device).contains("lo de dev_a"));
}

fn home_of(who: &Machine, whose: &str) -> String {
    std::fs::read_to_string(who.store.join(whose).join("active.tisty")).unwrap_or_default()
}

#[test]
fn a_history_from_the_fence_onward_owes_a_signature_even_saying_no_key() {
    let one = blank("dev_a");
    let whose = DeviceId(one.device.clone());
    let mut held = Store::open(&one.store, whose).unwrap();
    held.append(Op::TaskAdd {
        id: Ulid::generate(),
        d: tisty_core::event::TaskAdd::new("lo de dev_a", "a0"),
    })
    .unwrap();
    drop(held);

    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();
    let whole = std::fs::read_to_string(
        shared
            .path()
            .join(STORE)
            .join(&one.device)
            .join("active.tisty"),
    )
    .unwrap();
    assert!(
        whole.contains(&format!("\"v\":{}", tisty_core::event::SIGNED_FROM)),
        "the folder does not hold a history at the schema this is about"
    );
    assert!(!whole.contains("device.key"), "it says what it signs with");

    let two = blank("dev_b");
    std::fs::create_dir_all(&two.data).unwrap();
    let after = carry(&two.data, &two.device, shared.path(), Way::Both, &[]).unwrap();

    assert_eq!(
        after.disowned,
        vec![one.device.clone()],
        "a history written from the fence onward came in without a signature"
    );
    assert!(!home_of(&two, &one.device).contains("lo de dev_a"));
}

fn doc_named(who: &Machine, file: &str) -> Ulid {
    tisty_core::State::replay(&tisty_core::store::read_all(&who.store).unwrap())
        .docs
        .values()
        .find(|paper| paper.file == file)
        .map(|paper| paper.id)
        .expect("the document is in the log")
}

fn edited(who: &Machine, file: &str, body: &str) {
    wrote_body(&who.data.join(PAPERS), file, body);
    says(
        who,
        Op::DocSaid {
            id: doc_named(who, file),
            d: tisty_core::event::Said::of(body),
        },
    );
}

fn keyed(who: &Machine) -> Option<tisty_core::signing::SigningKey> {
    let paths = tisty_core::Paths::new(who.data.clone(), who.data.join("config"));
    let whose = DeviceId(who.device.clone());
    let key = tisty_core::signing::mine(&paths, &whose);
    says(
        who,
        Op::DeviceJoin {
            d: whose,
            k: Some(tisty_core::DeviceKind::Machine),
            p: key.as_ref().map(tisty_core::signing::shown),
        },
    );
    key
}

fn round(who: &Machine, shared: &Path, way: Way) -> Moved {
    let alive: Vec<String> =
        tisty_core::State::replay(&tisty_core::store::read_all(&who.store).unwrap())
            .docs
            .values()
            .map(|paper| paper.file.clone())
            .collect();
    carry_holding(
        &who.data,
        None,
        &who.device,
        shared,
        way,
        &alive,
        Holds::Everywhere,
    )
    .unwrap()
}

fn answered(who: &Machine, joined: &[String]) {
    for file in joined {
        let body =
            std::fs::read_to_string(who.data.join(PAPERS).join(format!("{file}.md"))).unwrap();
        says(
            who,
            Op::DocSaid {
                id: doc_named(who, file),
                d: tisty_core::event::Said::of(&body),
            },
        );
    }
}

fn joined_across(first: &str, joining: Way, last: Way) -> (Moved, String) {
    let one = machine("uno");
    keyed(&one);
    let shared = tempfile::tempdir().unwrap();
    let base = "# Kit\n\nla introduccion\n\nel cuerpo\n\nel cierre\n";
    filed(&one, "uno-0001", base);
    edited(&one, "uno-0001", base);
    round(&one, shared.path(), Way::Both);
    let two = blank("dos");
    round(&two, shared.path(), Way::Both);
    keyed(&two);
    round(&two, shared.path(), Way::Both);

    let mac = "# Kit\n\nla introduccion del mac\n\nel cuerpo\n\nel cierre\n";
    let windows = "# Kit\n\nla introduccion\n\nel cuerpo\n\nel cierre\n\nlo de windows\n";
    if first == "uno" {
        edited(&one, "uno-0001", mac);
        edited(&two, "uno-0001", windows);
    } else {
        edited(&two, "uno-0001", windows);
        edited(&one, "uno-0001", mac);
    }
    round(&two, shared.path(), Way::Both);
    let joining = round(&one, shared.path(), joining);
    assert_eq!(joining.joined, vec!["uno-0001".to_string()], "{joining:?}");
    answered(&one, &joining.to_answer());
    round(&one, shared.path(), Way::Push);

    let after = round(&two, shared.path(), last);
    let whole = std::fs::read_to_string(two.data.join(PAPERS).join("uno-0001.md")).unwrap();
    (after, whole)
}

#[test]
fn a_body_joined_on_one_machine_reaches_the_other_without_a_question() {
    for first in ["uno", "dos"] {
        for joining in [Way::Both, Way::Pull] {
            for last in [Way::Both, Way::Pull] {
                let (after, whole) = joined_across(first, joining, last);
                assert!(after.unreadable.is_empty(), "{after:?}");
                assert!(
                    after.undecided_ids().is_empty(),
                    "{first} first, joined on {joining:?}, read on {last:?}: {after:?}"
                );
                assert!(
                    whole.contains("del mac") && whole.contains("lo de windows"),
                    "{first} first, joined on {joining:?}, read on {last:?}: {whole}"
                );
            }
        }
    }
}

#[test]
fn a_round_that_only_pushes_never_asks_about_what_it_did_not_bring() {
    let (after, _) = joined_across("uno", Way::Both, Way::Push);
    assert!(after.undecided_ids().is_empty(), "{after:?}");
}

#[test]
fn a_quiet_round_asks_each_document_only_what_it_has_to() {
    let one = machine("uno");
    let shared = tempfile::tempdir().unwrap();
    let alive: Vec<String> = (1..=20)
        .map(|n| {
            let file = format!("uno-{n:04}");
            filed(&one, &file, &format!("# Doc {n}\n\ncuerpo {n}\n"));
            file
        })
        .collect();
    carry_papers(&one.data, shared.path(), &alive).unwrap();
    carry_papers(&one.data, shared.path(), &alive).unwrap();

    tisty_core::counting::from_now();
    tisty_core::counting::looks_from_now();
    let done = carry_papers(&one.data, shared.path(), &alive).unwrap();
    let opened = tisty_core::counting::from_now();
    let looked = tisty_core::counting::looks_from_now();

    assert_eq!((done.sent, done.brought), (0, 0));
    assert_eq!(opened, 0, "a quiet round read a body it already knew");
    assert!(
        looked <= 3 * alive.len() as u64,
        "{looked} questions about {} documents",
        alive.len()
    );
}

fn quiet_opens(retiring: bool) -> u64 {
    let one = machine("uno");
    for n in 1..=20 {
        filed(
            &one,
            &format!("uno-{n:04}"),
            &format!("# Doc {n}\n\ncuerpo {n}\n"),
        );
    }
    if retiring {
        let kept = planted(&one.data, "foto.png", b"una fotografia retirada");
        says(&one, Op::AttachRetire { d: kept });
    }
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();
    tisty_core::counting::from_now();
    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();
    tisty_core::counting::from_now()
}

#[test]
fn a_retired_attachment_costs_a_quiet_round_one_reading_of_the_documents() {
    let plain = quiet_opens(false);
    let retiring = quiet_opens(true);
    assert!(retiring <= plain + 20, "{retiring} against {plain}");
}

#[test]
fn a_body_edited_outside_the_window_travels_with_its_print_and_asks_nothing() {
    let one = machine("uno");
    keyed(&one);
    let shared = tempfile::tempdir().unwrap();
    let base = "# Acta\n\nlo de siempre\n";
    filed(&one, "uno-0001", base);
    edited(&one, "uno-0001", base);
    round(&one, shared.path(), Way::Both);
    let two = blank("dos");
    round(&two, shared.path(), Way::Both);
    keyed(&two);
    round(&two, shared.path(), Way::Both);

    let outside = "# Acta\n\nlo de siempre\n\nescrito con otro editor\n";
    wrote_body(&one.data.join(PAPERS), "uno-0001", outside);
    let held = round(&one, shared.path(), Way::Both);
    assert_eq!(held.unanswered, vec!["uno-0001".to_string()], "{held:?}");
    assert_eq!(
        std::fs::read_to_string(shared.path().join(PAPERS).join("uno-0001.md")).unwrap(),
        base,
        "a body went to the folder before the log that answers for it"
    );
    answered(&one, &held.to_answer());
    round(&one, shared.path(), Way::Push);

    let after = round(&two, shared.path(), Way::Pull);
    assert!(after.undecided_ids().is_empty(), "{after:?}");
    assert_eq!(
        std::fs::read_to_string(two.data.join(PAPERS).join("uno-0001.md")).unwrap(),
        outside
    );
}

fn sent_up_to_the_cloud(at: &Path) {
    let name = at.file_name().unwrap().to_str().unwrap();
    std::fs::rename(at, at.with_file_name(format!(".{name}.icloud"))).unwrap();
}

fn brought_down_from_the_cloud(at: &Path) {
    let name = at.file_name().unwrap().to_str().unwrap();
    std::fs::rename(at.with_file_name(format!(".{name}.icloud")), at).unwrap();
}

#[test]
fn a_history_still_in_the_cloud_is_on_its_way_and_comes_in_a_later_turn() {
    let theirs = machine("dev_a");
    let shared = tempfile::tempdir().unwrap();
    carry(&theirs.data, &theirs.device, shared.path(), Way::Push, &[]).unwrap();
    let up = shared.path().join(STORE).join("dev_a").join("active.tisty");
    sent_up_to_the_cloud(&up);

    let ours = blank("dev_b");
    let first = carry(&ours.data, &ours.device, shared.path(), Way::Pull, &[]).unwrap();

    assert_eq!(first.coming, vec!["dev_a".to_string()]);
    assert!(first.unreadable.is_empty(), "on its way is not unreadable");
    assert!(titles(&ours.store).is_empty());

    brought_down_from_the_cloud(&up);
    let next = carry(&ours.data, &ours.device, shared.path(), Way::Pull, &[]).unwrap();

    assert!(next.coming.is_empty());
    assert!(titles(&ours.store).contains(&"lo de dev_a".to_string()));
}

#[test]
fn a_document_still_in_the_cloud_is_never_read_and_comes_in_a_later_turn() {
    let theirs = machine("dev_a");
    let alive = ["nota-0001".to_string()];
    filed(
        &theirs,
        "nota-0001",
        "# Nota

lo que dice la nota
",
    );
    let shared = tempfile::tempdir().unwrap();
    carry(
        &theirs.data,
        &theirs.device,
        shared.path(),
        Way::Push,
        &alive,
    )
    .unwrap();
    let up = tisty_core::docs::resolve(&shared.path().join(PAPERS), "nota-0001").unwrap();
    sent_up_to_the_cloud(&up);

    let ours = blank("dev_b");
    let first = carry(&ours.data, &ours.device, shared.path(), Way::Pull, &alive).unwrap();

    assert!(!first.coming.is_empty(), "{first:?}");
    let mine = tisty_core::docs::resolve(&ours.data.join(PAPERS), "nota-0001").unwrap();
    assert!(!mine.exists());

    brought_down_from_the_cloud(&up);
    carry(&ours.data, &ours.device, shared.path(), Way::Pull, &alive).unwrap();

    assert_eq!(
        std::fs::read_to_string(&mine).unwrap(),
        "# Nota

lo que dice la nota
"
    );
}

#[test]
fn a_newer_machine_with_an_old_segment_still_in_the_cloud_still_stops_the_turn() {
    let one = machine("dev_a");
    let shared = tempfile::tempdir().unwrap();
    let theirs = shared.path().join("store/dev_b");
    std::fs::create_dir_all(&theirs).unwrap();
    std::fs::write(theirs.join(".000001.tisty.icloud"), b"stub").unwrap();
    std::fs::write(
        theirs.join("active.tisty"),
        b"{\"v\":99,\"ts\":\"2026-08-26T10:00:00Z\",\"by\":\"dev_b\",\"op\":\"task.add\",\"id\":\"01M0ZX62YMRXMABJ6Q4FEF69WT\",\"d\":{\"title\":\"from the future\",\"order\":\"V\"}}\n",
    )
    .unwrap();

    let stopped = carry(&one.data, &one.device, shared.path(), Way::Both, &[]);

    assert!(
        matches!(stopped, Err(Trouble::Newer(ref who)) if who == "dev_b"),
        "{stopped:?}"
    );
}

#[test]
fn what_arrives_from_the_cloud_stirs_the_folder() {
    let theirs = machine("dev_a");
    let shared = tempfile::tempdir().unwrap();
    carry(&theirs.data, &theirs.device, shared.path(), Way::Push, &[]).unwrap();
    let up = shared.path().join(STORE).join("dev_a").join("active.tisty");
    sent_up_to_the_cloud(&up);
    let before = stirring(shared.path());

    brought_down_from_the_cloud(&up);

    assert_ne!(before, stirring(shared.path()));
}

#[test]
fn the_name_in_the_folder_is_read_from_what_is_here_and_never_waits_on_the_cloud() {
    let signer = machine("uno");
    signs(&signer, "mario");
    let other = blank("dos");
    let shared = tempfile::tempdir().unwrap();
    carry(&signer.data, &signer.device, shared.path(), Way::Push, &[]).unwrap();
    carry(&other.data, &other.device, shared.path(), Way::Both, &[]).unwrap();
    wrote(&other, "lo de dos".into());
    carry(&other.data, &other.device, shared.path(), Way::Push, &[]).unwrap();
    sent_up_to_the_cloud(&shared.path().join(STORE).join("dos").join("active.tisty"));

    assert_eq!(signed_at(shared.path()).as_deref(), Some("mario"));
}

#[test]
fn a_name_still_in_the_cloud_is_said_to_be_coming_rather_than_missing() {
    let signer = machine("uno");
    signs(&signer, "mario");
    let shared = tempfile::tempdir().unwrap();
    carry(&signer.data, &signer.device, shared.path(), Way::Push, &[]).unwrap();
    let up = shared.path().join(STORE).join("uno").join("active.tisty");
    sent_up_to_the_cloud(&up);

    assert_eq!(
        signed_here(shared.path()),
        Signed {
            alias: None,
            coming: true
        }
    );

    brought_down_from_the_cloud(&up);

    assert_eq!(
        signed_here(shared.path()),
        Signed {
            alias: Some("mario".into()),
            coming: false
        }
    );
}

#[test]
fn a_round_tells_how_far_it_got_history_first_and_never_counting_back() {
    let one = machine("dev_a");
    filed(&one, "uno-0001", "# Kit\n\nuno\n");
    let (_src, file) = {
        let dir = tempfile::tempdir().unwrap();
        let at = dir.path().join("contrato.pdf");
        std::fs::write(&at, b"what the person really attached").unwrap();
        (dir, at)
    };
    let kept =
        tisty_core::attach::keep(&file, &one.data, tisty_core::attach::COPIED_UP_TO).unwrap();
    let shared = tempfile::tempdir().unwrap();
    carry(
        &one.data,
        &one.device,
        shared.path(),
        Way::Push,
        &["uno-0001".into()],
    )
    .unwrap();

    let other = blank("dev_b");
    std::fs::create_dir_all(&other.data).unwrap();
    let mut heard = Vec::new();
    carry_telling(
        &other.data,
        None,
        &other.device,
        shared.path(),
        Way::Both,
        &[],
        Holds::Everywhere,
        &mut |far| heard.push(far),
    )
    .unwrap();

    let first = |wanted: Stage| {
        heard
            .iter()
            .position(|far| matches!(far, Reached::Along { stage, .. } if *stage == wanted))
            .unwrap_or_else(|| panic!("{wanted:?} never said how far it got"))
    };
    let log_done = heard.iter().position(|far| *far == Reached::Log).unwrap();
    assert!(first(Stage::Log) < log_done);
    assert!(
        log_done < first(Stage::Papers) && first(Stage::Papers) < first(Stage::Attachments),
        "the history was not whole before bodies and attachments were opened: {heard:?}"
    );
    for wanted in [Stage::Log, Stage::Papers, Stage::Attachments] {
        let counts: Vec<(usize, usize)> = heard
            .iter()
            .filter_map(|far| match far {
                Reached::Along { stage, done, whole } if *stage == wanted => Some((*done, *whole)),
                _ => None,
            })
            .collect();
        assert!(
            counts.windows(2).all(|two| two[0].0 <= two[1].0),
            "{wanted:?} counted back: {counts:?}"
        );
        assert!(counts.iter().all(|(done, whole)| done <= whole));
        assert!(counts.last().is_some_and(|(done, whole)| done == whole));
    }
    assert_eq!(
        heard
            .iter()
            .filter_map(|far| match far {
                Reached::Kept { at, .. } => Some(at.clone()),
                _ => None,
            })
            .collect::<Vec<_>>(),
        vec![kept.at.clone()],
        "what landed was not told as it landed"
    );
}

fn given_a_key(one: &Machine) {
    let paths = tisty_core::Paths::new(one.data.clone(), one.data.join("config"));
    let who = DeviceId(one.device.clone());
    let key = tisty_core::signing::mine(&paths, &who).expect("a key");
    let mut held = Store::open(&one.store, who.clone())
        .unwrap()
        .signing_with(Some(key.clone()));
    held.append(Op::DeviceKey {
        d: who,
        p: tisty_core::signing::shown(&key),
    })
    .unwrap();
}

fn joined_with_a_key(named: &str, shared: &Path) -> Machine {
    let one = blank(named);
    std::fs::create_dir_all(&one.data).unwrap();
    carry(&one.data, &one.device, shared, Way::Both, &[]).unwrap();
    given_a_key(&one);
    wrote(&one, format!("lo de {named}"));
    carry(&one.data, &one.device, shared, Way::Push, &[]).unwrap();
    one
}

#[test]
fn a_machine_still_in_the_cloud_when_the_folder_was_taken_up_is_taken_up_when_it_comes_down() {
    let one = machine("dev_a");
    given_a_key(&one);
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Push, &[]).unwrap();
    let _three = joined_with_a_key("dev_c", shared.path());
    let up = shared.path().join(STORE).join("dev_c").join("active.tisty");
    sent_up_to_the_cloud(&up);

    let two = blank("dev_b");
    std::fs::create_dir_all(&two.data).unwrap();
    let kept = tempfile::tempdir().unwrap();
    let aside = Some(kept.path());
    let first =
        carry_leaning_on(&two.data, aside, &two.device, shared.path(), Way::Both, &[]).unwrap();
    assert_eq!(first.coming, vec!["dev_c".to_string()]);
    assert!(first.unconfirmed.is_empty());

    brought_down_from_the_cloud(&up);
    let after =
        carry_leaning_on(&two.data, aside, &two.device, shared.path(), Way::Both, &[]).unwrap();

    assert!(
        after.unconfirmed.is_empty(),
        "a machine that was in the folder when it was taken up waits as if it had shown up later"
    );
    assert!(home_of(&two, "dev_c").contains("lo de dev_c"));
    assert!(
        !kept.path().join("adopting").exists(),
        "taking up stayed open once everybody had come down"
    );

    let _four = joined_with_a_key("dev_d", shared.path());
    let later =
        carry_leaning_on(&two.data, aside, &two.device, shared.path(), Way::Both, &[]).unwrap();
    assert_eq!(
        later.unconfirmed,
        vec!["dev_d".to_string()],
        "a machine that showed up after taking up was taken on sight"
    );
}

fn upgraded_in_place(whose: &Machine, shared: &Path, older: &str, next: &str) -> String {
    let there = whose.store.join(&whose.device);
    std::fs::create_dir_all(&there).unwrap();
    std::fs::write(there.join("active.tisty"), older).unwrap();
    let who = DeviceId(whose.device.clone());
    let paths = tisty_core::Paths::new(whose.data.clone(), whose.data.join("config"));
    let key = tisty_core::signing::mine(&paths, &who).unwrap();
    let shown = tisty_core::signing::shown(&key);
    let mut held = Store::open(&whose.store, who.clone())
        .unwrap()
        .signing_with(Some(key));
    held.append(Op::DeviceKey {
        d: who,
        p: shown.clone(),
    })
    .unwrap();
    held.append(Op::TaskAdd {
        id: Ulid::generate(),
        d: TaskAdd::new(next, "a0"),
    })
    .unwrap();
    drop(held);
    let folder = shared.join(STORE).join(&whose.device);
    for found in std::fs::read_dir(&there).unwrap() {
        let found = found.unwrap().path();
        std::fs::copy(&found, folder.join(found.file_name().unwrap())).unwrap();
    }
    shown
}

fn before_the_fence(whose: &Machine, shared: &Path) -> String {
    let older = format!(
        "{{\"v\":{},\"ts\":\"2026-10-01T12:00:00Z\",\"by\":\"{}\",\"op\":\"task.add\",\"id\":\"{}\",\"d\":{{\"title\":\"lo viejo de {}\",\"order\":\"a0\"}}}}\n",
        tisty_core::event::SIGNED_FROM - 1,
        whose.device,
        Ulid::generate(),
        whose.device
    );
    let folder = shared.join(STORE).join(&whose.device);
    std::fs::create_dir_all(&folder).unwrap();
    std::fs::write(folder.join("active.tisty"), &older).unwrap();
    older
}

#[test]
fn a_machine_known_from_before_signing_is_taken_on_its_first_key_without_asking() {
    let one = machine("dev_a");
    let kept = tempfile::tempdir().unwrap();
    let aside = Some(kept.path());
    let shared = tempfile::tempdir().unwrap();
    carry_leaning_on(&one.data, aside, &one.device, shared.path(), Way::Both, &[]).unwrap();
    let two = blank("dev_b");
    let older = before_the_fence(&two, shared.path());
    carry_leaning_on(&one.data, aside, &one.device, shared.path(), Way::Both, &[]).unwrap();
    assert!(home_of(&one, &two.device).contains("lo viejo de dev_b"));

    let shown = upgraded_in_place(&two, shared.path(), &older, "lo nuevo de dev_b");
    let after =
        carry_leaning_on(&one.data, aside, &one.device, shared.path(), Way::Both, &[]).unwrap();

    assert!(after.unconfirmed.is_empty(), "{:?}", after.unconfirmed);
    assert!(home_of(&one, &two.device).contains("lo nuevo de dev_b"));
    let stood = tisty_core::vouched::confirmed(&one.data, &DeviceId(two.device.clone())).unwrap();
    assert_eq!(stood.key, shown);
    assert!(
        stood.carried,
        "nobody compared it, and the record has to say so"
    );
}

#[test]
fn a_known_machine_whose_old_history_was_rewritten_waits_for_a_person() {
    let one = machine("dev_a");
    let kept = tempfile::tempdir().unwrap();
    let aside = Some(kept.path());
    let shared = tempfile::tempdir().unwrap();
    carry_leaning_on(&one.data, aside, &one.device, shared.path(), Way::Both, &[]).unwrap();
    let two = blank("dev_b");
    let older = before_the_fence(&two, shared.path());
    carry_leaning_on(&one.data, aside, &one.device, shared.path(), Way::Both, &[]).unwrap();

    let rewritten = older.replace("lo viejo", "lo cambiado");
    upgraded_in_place(&two, shared.path(), &rewritten, "lo nuevo de dev_b");
    let after =
        carry_leaning_on(&one.data, aside, &one.device, shared.path(), Way::Both, &[]).unwrap();

    assert_eq!(after.unconfirmed, vec![two.device.clone()]);
    assert!(!home_of(&one, &two.device).contains("lo nuevo de dev_b"));
}

#[test]
fn a_signed_history_that_has_not_said_its_key_waits_instead_of_coming_in_unchecked() {
    let one = machine("dev_a");
    let kept = tempfile::tempdir().unwrap();
    let aside = Some(kept.path());
    let shared = tempfile::tempdir().unwrap();
    carry_leaning_on(&one.data, aside, &one.device, shared.path(), Way::Both, &[]).unwrap();
    let two = blank("dev_b");
    let older = before_the_fence(&two, shared.path());
    carry_leaning_on(&one.data, aside, &one.device, shared.path(), Way::Both, &[]).unwrap();

    let there = two.store.join(&two.device);
    std::fs::create_dir_all(&there).unwrap();
    std::fs::write(there.join("active.tisty"), &older).unwrap();
    let who = DeviceId(two.device.clone());
    let paths = tisty_core::Paths::new(two.data.clone(), two.data.join("config"));
    let mut held = Store::open(&two.store, who.clone())
        .unwrap()
        .signing_with(tisty_core::signing::mine(&paths, &who));
    held.append(Op::TaskAdd {
        id: Ulid::generate(),
        d: TaskAdd::new("sin decir su clave", "a0"),
    })
    .unwrap();
    drop(held);
    let folder = shared.path().join(STORE).join(&two.device);
    for found in std::fs::read_dir(&there).unwrap() {
        let found = found.unwrap().path();
        std::fs::copy(&found, folder.join(found.file_name().unwrap())).unwrap();
    }

    let after =
        carry_leaning_on(&one.data, aside, &one.device, shared.path(), Way::Both, &[]).unwrap();

    assert_eq!(after.unconfirmed, vec![two.device.clone()]);
    assert!(
        !home_of(&one, &two.device).contains("sin decir su clave"),
        "a signed history nobody can check came home on the folder's word"
    );
}

#[test]
fn a_reinstall_that_kept_the_cache_meets_the_folder_as_a_new_machine() {
    let kept = tempfile::tempdir().unwrap();
    let shared = tempfile::tempdir().unwrap();
    super::place::note_carried(Some(kept.path()), shared.path(), "dev_a");

    assert!(super::place::carried_here(
        Some(kept.path()),
        shared.path(),
        Some("dev_a")
    ));
    assert!(
        !super::place::carried_here(Some(kept.path()), shared.path(), Some("dev_z")),
        "another identity's memo stopped this one from taking the folder up"
    );
    assert!(super::place::carried_here(
        Some(kept.path()),
        shared.path(),
        None
    ));
}

#[test]
fn a_memo_from_before_it_named_its_machine_still_reads_as_having_been_here() {
    let kept = tempfile::tempdir().unwrap();
    let shared = tempfile::tempdir().unwrap();
    std::fs::write(
        kept.path().join(super::place::CARRIED_TO),
        tisty_core::paths::told_of(shared.path()),
    )
    .unwrap();

    assert!(
        super::place::carried_here(Some(kept.path()), shared.path(), Some("dev_a")),
        "every machine would take its folder up again, answering for every key in it"
    );
}

struct Hosted {
    host: Machine,
    agent: DeviceId,
    host_key: String,
}

fn hosted(shared: &Path, agent_speaks_for_itself: bool) -> Hosted {
    hosted_as(
        shared,
        agent_speaks_for_itself,
        tisty_core::event::DeviceKind::Agent,
    )
}

fn hosted_as(
    shared: &Path,
    agent_speaks_for_itself: bool,
    kind: tisty_core::event::DeviceKind,
) -> Hosted {
    let host = blank("dev_h");
    std::fs::create_dir_all(&host.data).unwrap();
    let paths = tisty_core::Paths::new(host.data.clone(), host.data.join("config"));
    let me = DeviceId(host.device.clone());
    let agent = DeviceId("dev_g".into());
    let host_key = tisty_core::signing::mine(&paths, &me).expect("a key");
    let agent_key = tisty_core::signing::mine(&paths, &agent).expect("a key");
    let agent_said = tisty_core::signing::shown(&agent_key);

    let mut mine = Store::open(&host.store, me.clone())
        .unwrap()
        .signing_with(Some(host_key.clone()));
    mine.append(Op::DeviceKey {
        d: me.clone(),
        p: tisty_core::signing::shown(&host_key),
    })
    .unwrap();
    if !agent_speaks_for_itself {
        mine.append(Op::DeviceHost {
            d: agent.clone(),
            of: me.clone(),
            p: Some(agent_said.clone()),
        })
        .unwrap();
    }
    drop(mine);

    let mut theirs = Store::open(&host.store, agent.clone())
        .unwrap()
        .signing_with(Some(agent_key));
    theirs
        .append_batch(vec![
            Op::DeviceJoin {
                d: agent.clone(),
                k: Some(kind),
                p: Some(agent_said.clone()),
            },
            Op::DeviceHost {
                d: agent.clone(),
                of: me.clone(),
                p: agent_speaks_for_itself.then(|| agent_said.clone()),
            },
        ])
        .unwrap();
    theirs
        .append(Op::TaskAdd {
            id: Ulid::generate(),
            d: TaskAdd::new("lo que escribió el agente", "a0"),
        })
        .unwrap();
    drop(theirs);

    std::fs::copy(
        shared.join(STORE).join(".store-id"),
        host.store.join(".store-id"),
    )
    .unwrap();
    carry(&host.data, &host.device, shared, Way::Push, &[]).unwrap();
    Hosted {
        host,
        agent,
        host_key: tisty_core::signing::shown(&host_key),
    }
}

/// Already settled in the folder before the host arrives, so nothing comes in on a first visit's terms.
fn settled_in(shared: &Path, aside: &Path) -> Machine {
    let other = machine("dev_b");
    carry_leaning_on(
        &other.data,
        Some(aside),
        &other.device,
        shared,
        Way::Both,
        &[],
    )
    .unwrap();
    other
}

#[test]
fn an_agent_comes_in_on_the_word_of_a_host_already_confirmed_here() {
    let shared = tempfile::tempdir().unwrap();
    let kept = tempfile::tempdir().unwrap();
    let other = settled_in(shared.path(), kept.path());
    let seen = hosted(shared.path(), false);
    assert!(tisty_core::vouched::confirm(
        &other.data,
        &DeviceId(seen.host.device.clone()),
        &seen.host_key
    ));

    let round = || {
        carry_leaning_on(
            &other.data,
            Some(kept.path()),
            &other.device,
            shared.path(),
            Way::Both,
            &[],
        )
        .unwrap()
    };
    round();
    let after = round();

    assert!(after.unconfirmed.is_empty(), "{:?}", after.unconfirmed);
    assert!(
        home_of(&other, &seen.agent.0).contains("lo que escribió el agente"),
        "an agent of a confirmed computer still waited to be confirmed on its own"
    );
    let stood = tisty_core::vouched::confirmed(&other.data, &seen.agent).unwrap();
    assert_eq!(stood.host, Some(DeviceId(seen.host.device.clone())));
    assert!(!stood.carried);
}

#[test]
fn an_agent_whose_host_nobody_here_confirmed_still_waits() {
    let shared = tempfile::tempdir().unwrap();
    let kept = tempfile::tempdir().unwrap();
    let other = settled_in(shared.path(), kept.path());
    let seen = hosted(shared.path(), false);

    let round = || {
        carry_leaning_on(
            &other.data,
            Some(kept.path()),
            &other.device,
            shared.path(),
            Way::Both,
            &[],
        )
        .unwrap()
    };
    round();
    let after = round();

    assert!(
        after.unconfirmed.contains(&seen.agent.0),
        "{:?}",
        after.unconfirmed
    );
    assert!(tisty_core::vouched::confirmed(&other.data, &seen.agent).is_none());
}

#[test]
fn an_agent_that_vouches_for_itself_is_not_taken_on_its_own_word() {
    let shared = tempfile::tempdir().unwrap();
    let kept = tempfile::tempdir().unwrap();
    let other = settled_in(shared.path(), kept.path());
    let seen = hosted(shared.path(), true);
    assert!(tisty_core::vouched::confirm(
        &other.data,
        &DeviceId(seen.host.device.clone()),
        &seen.host_key
    ));

    let round = || {
        carry_leaning_on(
            &other.data,
            Some(kept.path()),
            &other.device,
            shared.path(),
            Way::Both,
            &[],
        )
        .unwrap()
    };
    round();
    let after = round();

    assert!(
        after.unconfirmed.contains(&seen.agent.0),
        "an agent's word about its own key stood in for its host's"
    );
}

#[test]
fn a_whole_machine_is_never_taken_on_another_machines_word() {
    let shared = tempfile::tempdir().unwrap();
    let kept = tempfile::tempdir().unwrap();
    let other = settled_in(shared.path(), kept.path());
    let seen = hosted_as(shared.path(), false, tisty_core::event::DeviceKind::Machine);
    assert!(tisty_core::vouched::confirm(
        &other.data,
        &DeviceId(seen.host.device.clone()),
        &seen.host_key
    ));
    let round = || {
        carry_leaning_on(
            &other.data,
            Some(kept.path()),
            &other.device,
            shared.path(),
            Way::Both,
            &[],
        )
        .unwrap()
    };
    round();
    let after = round();

    assert!(
        after.unconfirmed.contains(&seen.agent.0),
        "a confirmed machine seated a whole other machine nobody compared"
    );
}

/// A machine nobody here confirmed, which changed `file` to `body` in the folder.
fn waiting_writer(shared: &Path, id: tisty_core::model::DocId, body: &str, signed: bool) {
    let w = blank("dev_w");
    std::fs::create_dir_all(&w.data).unwrap();
    let paths = tisty_core::Paths::new(w.data.clone(), w.data.join("config"));
    let who = DeviceId(w.device.clone());
    let key = tisty_core::signing::mine(&paths, &who).expect("a key");
    let mut held = Store::open(&w.store, who.clone())
        .unwrap()
        .signing_with(signed.then(|| key.clone()));
    held.append(Op::DeviceJoin {
        d: who.clone(),
        k: Some(tisty_core::DeviceKind::Machine),
        p: Some(tisty_core::signing::shown(&key)),
    })
    .unwrap();
    held.append(Op::DocSaid {
        id,
        d: tisty_core::event::Said::of(body),
    })
    .unwrap();
    drop(held);
    let there = shared.join(STORE).join(&w.device);
    std::fs::create_dir_all(&there).unwrap();
    for found in std::fs::read_dir(w.store.join(&w.device)).unwrap() {
        let found = found.unwrap().path();
        std::fs::copy(&found, there.join(found.file_name().unwrap())).unwrap();
    }
    wrote_body(&shared.join(PAPERS), "uno-0001", body);
}

fn a_document_settled_in(shared: &Path, aside: &Path) -> (Machine, tisty_core::model::DocId) {
    let one = machine("uno");
    keyed(&one);
    let body = "# Notas\n\nlo que escribi\n";
    filed(&one, "uno-0001", body);
    edited(&one, "uno-0001", body);
    let id = doc_named(&one, "uno-0001");
    carry_leaning_on(&one.data, Some(aside), &one.device, shared, Way::Both, &[]).unwrap();
    carry_leaning_on(&one.data, Some(aside), &one.device, shared, Way::Both, &[]).unwrap();
    (one, id)
}

fn turn(one: &Machine, aside: &Path, shared: &Path) -> Moved {
    let alive = vec!["uno-0001".to_string()];
    carry_leaning_on(
        &one.data,
        Some(aside),
        &one.device,
        shared,
        Way::Both,
        &alive,
    )
    .unwrap()
}

#[test]
fn a_body_a_waiting_machine_answers_for_waits_with_it_instead_of_asking() {
    let shared = tempfile::tempdir().unwrap();
    let kept = tempfile::tempdir().unwrap();
    let (one, id) = a_document_settled_in(shared.path(), kept.path());
    waiting_writer(
        shared.path(),
        id,
        "# Notas\n\nlo que cambio el otro\n",
        true,
    );

    let moved = turn(&one, kept.path(), shared.path());

    assert!(
        moved.unconfirmed.contains(&"dev_w".to_string()),
        "{moved:?}"
    );
    assert!(
        moved.undecided_ids().is_empty(),
        "the person was asked about a body whose answer was only waiting"
    );
    assert_eq!(moved.waiting, vec!["uno-0001".to_string()]);
    assert_eq!(
        body(&one.data, "uno-0001"),
        "# Notas\n\nlo que escribi\n",
        "a waiting body came in before the machine that wrote it was confirmed"
    );
}

#[test]
fn a_forged_waiting_history_never_silences_the_question() {
    let shared = tempfile::tempdir().unwrap();
    let kept = tempfile::tempdir().unwrap();
    let (one, id) = a_document_settled_in(shared.path(), kept.path());
    waiting_writer(shared.path(), id, "algo que nadie firmo\n", false);
    waited_long(&one.data, "uno-0001");

    let moved = turn(&one, kept.path(), shared.path());

    assert!(moved.waiting.is_empty(), "{moved:?}");
    assert_eq!(moved.undecided_ids(), vec!["uno-0001".to_string()]);
}

#[test]
fn removing_the_waiting_machine_puts_its_document_to_the_person() {
    let shared = tempfile::tempdir().unwrap();
    let kept = tempfile::tempdir().unwrap();
    let (one, id) = a_document_settled_in(shared.path(), kept.path());
    waiting_writer(
        shared.path(),
        id,
        "# Notas\n\nlo que cambio el otro\n",
        true,
    );
    assert_eq!(turn(&one, kept.path(), shared.path()).waiting.len(), 1);

    says(
        &one,
        Op::DeviceRemove {
            d: DeviceId("dev_w".into()),
        },
    );
    waited_long(&one.data, "uno-0001");
    let moved = turn(&one, kept.path(), shared.path());

    assert!(moved.waiting.is_empty(), "{moved:?}");
    assert!(
        !moved.unconfirmed.contains(&"dev_w".to_string()),
        "a removed machine was still said to be waiting to be confirmed"
    );
    assert_eq!(
        moved.undecided_ids(),
        vec!["uno-0001".to_string()],
        "a removed machine kept its document waiting with no way out"
    );
}

#[test]
fn a_locked_document_a_waiting_machine_answers_for_is_put_to_the_person() {
    let shared = tempfile::tempdir().unwrap();
    let kept = tempfile::tempdir().unwrap();
    let (one, id) = a_document_settled_in(shared.path(), kept.path());
    waiting_writer(
        shared.path(),
        id,
        "# Notas

lo que cambio el otro
",
        true,
    );
    says(&one, Op::DocLock { id });

    let moved = turn(&one, kept.path(), shared.path());

    assert!(
        moved.waiting.is_empty(),
        "a locked document was said to be waiting on a machine: {moved:?}"
    );
    assert_eq!(moved.undecided_ids(), vec!["uno-0001".to_string()]);
}

#[test]
fn a_document_edited_here_as_well_is_put_to_the_person_and_never_held() {
    let shared = tempfile::tempdir().unwrap();
    let kept = tempfile::tempdir().unwrap();
    let (one, id) = a_document_settled_in(shared.path(), kept.path());
    waiting_writer(
        shared.path(),
        id,
        "# Notas\n\nlo que cambio el otro\n",
        true,
    );
    edited(&one, "uno-0001", "# Notas\n\nlo que cambie yo\n");

    let moved = turn(&one, kept.path(), shared.path());

    assert!(
        moved.waiting.is_empty(),
        "the person's own edits were held back with someone else's"
    );
    assert_eq!(moved.undecided_ids(), vec!["uno-0001".to_string()]);
}

#[test]
fn a_round_leaves_nothing_copied_aside_behind() {
    let one = machine("dev_a");
    let other = blank("dev_b");
    let shared = tempfile::tempdir().unwrap();
    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();
    joined(&other, shared.path());
    wrote(&other, "lo de dev_b".into());
    carry(&other.data, &other.device, shared.path(), Way::Both, &[]).unwrap();
    let running = one.data.join(".bringing").join("1-another-round");
    std::fs::create_dir_all(&running).unwrap();

    carry(&one.data, &one.device, shared.path(), Way::Both, &[]).unwrap();

    assert!(
        running.is_dir(),
        "a round swept away the place another round was still using"
    );
    let left: Vec<_> = std::fs::read_dir(one.data.join(".bringing"))
        .unwrap()
        .filter_map(|one| one.ok())
        .map(|one| one.file_name())
        .collect();
    assert_eq!(
        left.len(),
        1,
        "what was copied aside stayed after the round: {left:?}"
    );
    let mine = titles(&one.store);
    assert!(mine.contains(&"lo de dev_b".to_string()), "{mine:?}");
}

#[test]
fn a_round_cut_short_still_leaves_nothing_copied_aside() {
    let room = tempfile::tempdir().unwrap();
    let aside = crate::bringing::Aside::taken(room.path());
    std::fs::create_dir_all(aside.at().join("dev_b")).unwrap();
    std::fs::write(aside.at().join("dev_b").join("active.tisty"), "x").unwrap();

    drop(aside);

    assert!(
        !room.path().join(".bringing").exists(),
        "a round that ended early left its copies behind"
    );
}
