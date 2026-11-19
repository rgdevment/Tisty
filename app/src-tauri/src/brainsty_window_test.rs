use super::{
    Kind, LABEL, Neuron, Place, Screen, Seen, address, after, allowed, keep_place, kept_place,
    opens, read_place,
};

const SCREEN: Screen = Screen {
    x: 0.0,
    y: 0.0,
    width: 1920.0,
    height: 1040.0,
};

const SPOT: Place = Place {
    x: 200.0,
    y: 120.0,
    width: 1280.0,
    height: 820.0,
    maximized: false,
};

fn seen(place: Place, minimized: bool, maximized: bool) -> Seen {
    Seen {
        place,
        minimized,
        maximized,
    }
}

#[test]
fn the_network_window_asks_only_for_what_its_page_uses() {
    for used in [
        "brain_network",
        "brainsty_groups",
        "brainsty_open",
        "doc_glance",
        "doc_drawn",
        "cards_drawn",
        "served",
        "draws_heic",
        "note_break",
        "note_trouble",
    ] {
        assert!(allowed(LABEL, used), "{used}");
    }
    for refused in [
        "snapshot",
        "delete_task",
        "doc_save",
        "brainsty_show",
        "task_of",
    ] {
        assert!(!allowed(LABEL, refused), "{refused}");
    }
}

#[test]
fn every_other_window_keeps_all_its_commands() {
    for label in ["main", "quick", "view-3", "brainsty-2"] {
        assert!(allowed(label, "delete_task"), "{label}");
    }
}

#[test]
fn a_spot_on_a_screen_comes_back() {
    assert_eq!(kept_place(Some(SPOT), &[SCREEN]), Some(SPOT));
}

#[test]
fn a_spot_on_a_screen_since_unplugged_is_let_go() {
    let away = Place { x: 2400.0, ..SPOT };
    assert_eq!(kept_place(Some(away), &[SCREEN]), None);
    let second = Screen {
        x: 1920.0,
        ..SCREEN
    };
    assert_eq!(kept_place(Some(away), &[SCREEN, second]), Some(away));
}

#[test]
fn a_spot_whose_title_bar_cannot_be_reached_is_let_go() {
    let above = Place { y: -60.0, ..SPOT };
    let below = Place { y: 1020.0, ..SPOT };
    let sliver = Place { x: 1850.0, ..SPOT };
    for gone in [above, below, sliver] {
        assert_eq!(kept_place(Some(gone), &[SCREEN]), None, "{gone:?}");
    }
}

#[test]
fn a_broken_or_missing_spot_opens_the_window_centred() {
    let flat = Place { width: 0.0, ..SPOT };
    let wild = Place {
        x: f64::NAN,
        ..SPOT
    };
    assert_eq!(kept_place(Some(flat), &[SCREEN]), None);
    assert_eq!(kept_place(Some(wild), &[SCREEN]), None);
    assert_eq!(kept_place(None, &[SCREEN]), None);
    assert_eq!(kept_place(Some(SPOT), &[]), None);
}

#[test]
fn a_move_or_a_new_size_is_what_the_window_keeps() {
    let moved = Place {
        x: 40.0,
        width: 900.0,
        ..SPOT
    };
    assert_eq!(after(Some(SPOT), seen(moved, false, false)), Some(moved));
    assert_eq!(after(None, seen(moved, false, false)), Some(moved));
}

#[test]
fn minimized_it_keeps_where_it_was() {
    let far = Place {
        x: -32000.0,
        y: -32000.0,
        ..SPOT
    };
    assert_eq!(after(Some(SPOT), seen(far, true, false)), Some(SPOT));
    assert_eq!(after(None, seen(far, true, false)), None);
}

#[test]
fn maximized_it_keeps_the_size_it_comes_back_to() {
    let whole = Place {
        x: 0.0,
        y: 0.0,
        width: 1920.0,
        height: 1040.0,
        maximized: false,
    };
    let kept = after(Some(SPOT), seen(whole, false, true));
    assert_eq!(
        kept,
        Some(Place {
            maximized: true,
            ..SPOT
        })
    );
    assert_eq!(after(kept, seen(SPOT, false, false)), Some(SPOT));
    assert_eq!(
        after(None, seen(whole, false, true)),
        Some(Place {
            maximized: true,
            ..whole
        })
    );
}

#[test]
fn the_place_survives_the_window_closing() {
    let room = tempfile::tempdir().unwrap();
    let at = room.path().join("brainsty-window.json");
    let maximized = Place {
        maximized: true,
        ..SPOT
    };

    assert_eq!(read_place(&at), None);
    keep_place(&at, maximized);
    assert_eq!(read_place(&at), Some(maximized));
    std::fs::write(&at, "{ not json").unwrap();
    assert_eq!(read_place(&at), None);
    keep_place(&room.path().join("gone").join("place.json"), SPOT);
}

#[test]
fn an_older_place_without_maximized_reads_as_not_maximized() {
    let room = tempfile::tempdir().unwrap();
    let at = room.path().join("brainsty-window.json");
    std::fs::write(&at, r#"{"x":1,"y":2,"width":800,"height":600}"#).unwrap();
    assert_eq!(
        read_place(&at),
        Some(Place {
            x: 1.0,
            y: 2.0,
            width: 800.0,
            height: 600.0,
            maximized: false,
        })
    );
}

#[test]
fn the_page_opens_in_the_app_s_language_on_what_was_last_touched() {
    assert_eq!(address(true, None), "index.html?locale=es");
    assert_eq!(address(false, Some("")), "index.html?locale=en");
    assert_eq!(address(false, Some("01A")), "index.html?locale=en&near=01A");
    assert_eq!(
        address(true, Some("notes & más.md")),
        "index.html?locale=es&near=notes%20%26%20m%C3%A1s%2Emd"
    );
}

#[test]
fn a_file_has_nowhere_to_open_and_everything_else_opens_in_the_main_window() {
    let one = |kind| Neuron {
        kind,
        id: "x".into(),
        title: "x".into(),
    };
    assert!(!opens(&one(Kind::File)));
    for kind in [Kind::Doc, Kind::Page, Kind::Task, Kind::Tag] {
        assert!(opens(&one(kind)), "{kind:?}");
    }
    let said: Neuron =
        serde_json::from_str(r##"{"kind":"tag","id":"#home","title":"home"}"##).unwrap();
    assert_eq!(said.kind, Kind::Tag);
    assert_eq!(
        serde_json::to_value(&said).unwrap(),
        serde_json::json!({ "kind": "tag", "id": "#home", "title": "home" })
    );
}
