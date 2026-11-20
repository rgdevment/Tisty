use tisty_core::State;
use tisty_core::model::{Task, TaskId};
use ulid::Ulid;

use super::{OUT_OF_SIGHT, masked, unmasked};

struct Shelf {
    state: State,
    hidden: TaskId,
    shown: TaskId,
    gone: TaskId,
}

fn shelf() -> Shelf {
    let mut state = State::default();
    let (hidden, shown, gone) = (Ulid::generate(), Ulid::generate(), Ulid::generate());
    let mut secret = Task::new(hidden, "Regalo sorpresa", "a0");
    secret.hidden = true;
    state.tasks.insert(hidden, secret);
    state
        .tasks
        .insert(shown, Task::new(shown, "Pagar la luz", "a1"));
    Shelf {
        state,
        hidden,
        shown,
        gone,
    }
}

#[test]
fn a_hidden_task_and_one_never_there_read_alike_and_a_visible_one_as_written() {
    let shelf = shelf();
    let with = |id: TaskId| {
        format!(
            "# Plan\n\n![Regalo sorpresa \\[Ana\\]](tisty:task/{id}) y [Pagar la luz](tisty:task/{}).\n",
            shelf.shown
        )
    };

    let hidden = masked(&shelf.state, &with(shelf.hidden)).unwrap();
    let gone = masked(&shelf.state, &with(shelf.gone)).unwrap();

    assert!(!hidden.contains("Regalo"), "{hidden}");
    assert!(hidden.contains(&format!("![{OUT_OF_SIGHT}](tisty:task/{})", shelf.hidden)));
    assert!(hidden.contains("[Pagar la luz]"), "{hidden}");
    assert_eq!(
        hidden.replace(&shelf.hidden.to_string(), "X"),
        gone.replace(&shelf.gone.to_string(), "X")
    );
}

#[test]
fn what_was_masked_comes_back_byte_for_byte_in_order() {
    let shelf = shelf();
    let was = format!(
        "![Regalo sorpresa](tisty:task/{h})\n\n<li><a href=\"tisty:task/{h}\">Regalo para Ana</a></li>\n\n[otra vez](tisty:task/{h}) y [Pagar](tisty:task/{s})\n",
        h = shelf.hidden,
        s = shelf.shown
    );

    let shown = masked(&shelf.state, &was).unwrap();
    assert!(
        !shown.contains("Regalo") && !shown.contains("otra vez"),
        "{shown}"
    );

    assert_eq!(
        unmasked(&shelf.state, &shown, &was).as_deref(),
        Some(was.as_str())
    );
    let edited = shown.replace("[Pagar]", "[Pagar hoy]");
    let back = unmasked(&shelf.state, &edited, &was).unwrap();
    assert_eq!(back, was.replace("[Pagar]", "[Pagar hoy]"));
}

#[test]
fn a_mask_with_nothing_to_stand_for_stays_as_sent() {
    let shelf = shelf();
    let was = "# Plan\n\nNada aún.\n";
    let sent = format!("[{OUT_OF_SIGHT}](tisty:task/{})", shelf.hidden);

    assert_eq!(unmasked(&shelf.state, &sent, was), None);
    assert_eq!(masked(&shelf.state, was), None);
    assert_eq!(
        masked(&shelf.state, &sent),
        None,
        "a mask is never masked again"
    );
}

#[test]
fn a_label_over_two_lines_or_with_brackets_inside_is_masked_whole() {
    let shelf = shelf();
    for label in ["Regalo\nsorpresa", "Regalo [para Ana] sorpresa"] {
        let text = format!("- [ ] Ver [{label}](tisty:task/{}).\n", shelf.hidden);
        let shown = masked(&shelf.state, &text).unwrap();
        assert_eq!(
            shown,
            format!("- [ ] Ver [{OUT_OF_SIGHT}](tisty:task/{}).\n", shelf.hidden)
        );
        assert_eq!(
            unmasked(&shelf.state, &shown, &text).as_deref(),
            Some(text.as_str())
        );
    }
}

#[test]
fn only_a_whole_task_link_inside_one_paragraph_is_read_as_one() {
    let shelf = shelf();
    for text in [
        format!("](tisty:task/{})", shelf.hidden),
        format!("[a\n\n](tisty:task/{})", shelf.hidden),
        "[x](tisty:task/)".to_string(),
        format!("[x](tisty:task/{} )", shelf.hidden),
    ] {
        assert_eq!(masked(&shelf.state, &text), None, "{text:?}");
    }
}
