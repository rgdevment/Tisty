use super::*;
use std::sync::Arc;

#[derive(Default)]
struct Fake {
    blind: bool,
    unread: bool,
    refused: Vec<String>,
    pending: Mutex<Vec<String>>,
    asked: Mutex<usize>,
}

impl Fake {
    fn holding(keys: &[String]) -> Arc<Self> {
        Arc::new(Self {
            pending: Mutex::new(keys.to_vec()),
            ..Self::default()
        })
    }

    fn held(&self) -> Vec<String> {
        self.pending.lock().unwrap().clone()
    }

    fn asked(&self) -> usize {
        *self.asked.lock().unwrap()
    }
}

impl Book for Arc<Fake> {
    fn shows(&self) -> bool {
        *self.asked.lock().unwrap() += 1;
        !self.blind
    }

    fn pending(&self) -> Option<Vec<String>> {
        (!self.unread).then(|| self.held())
    }

    fn cancel(&self, keys: &[String]) {
        self.pending
            .lock()
            .unwrap()
            .retain(|one| !keys.contains(one));
    }

    fn add(&self, ring: &Ring) -> bool {
        if self.refused.contains(&ring.title) {
            return false;
        }
        self.pending.lock().unwrap().push(ring.key());
        true
    }
}

const TASK: &str = "01KA7B3VJ9RZ0Q8M5N2X4Y6W1T";

fn moment(when: &str) -> jiff::Timestamp {
    when.parse().unwrap()
}

fn ring(title: &str, when: &str) -> Ring {
    Ring {
        task: TASK.to_string(),
        at: moment(when),
        heading: "Recordatorio".to_string(),
        title: title.to_string(),
        heard: true,
    }
}

fn due(of: &Ring) -> Due {
    Due {
        at: of.at,
        what: Happening::Due {
            title: of.title.clone(),
            task: of.task.clone(),
        },
    }
}

fn booking(book: &Arc<Fake>) -> Booking {
    Booking::new(Some(Box::new(Arc::clone(book))), 64)
}

#[test]
fn what_is_missing_is_booked_and_what_is_there_is_kept() {
    let kept = ring("comprar pan", "2026-08-12T13:00:00Z");
    let fresh = ring("regar las plantas", "2026-08-12T18:00:00Z");
    let book = Fake::holding(&[kept.key()]);

    let done = reconciled(&book, &[kept.clone(), fresh.clone()]);

    assert!(done.whole);
    assert_eq!(done.held, vec![kept.clone(), fresh.clone()]);
    assert_eq!(book.held(), vec![kept.key(), fresh.key()]);
}

#[test]
fn what_a_closed_tisty_left_behind_and_no_longer_applies_is_taken_back() {
    let gone = ring("pagar la luz", "2026-08-12T09:00:00Z");
    let moved = ring("pagar la luz", "2026-08-12T11:00:00Z");
    let book = Fake::holding(&[gone.key()]);

    let done = reconciled(&book, std::slice::from_ref(&moved));

    assert!(done.whole);
    assert_eq!(book.held(), vec![moved.key()]);
}

#[test]
fn what_someone_else_booked_is_left_alone() {
    let book = Fake::holding(&["another-app-1".to_string()]);

    reconciled(&book, &[]);

    assert_eq!(book.held(), vec!["another-app-1".to_string()]);
}

#[test]
fn a_system_that_cannot_show_holds_nothing_and_is_not_touched() {
    let left = ring("comprar pan", "2026-08-12T13:00:00Z");
    let book = Arc::new(Fake {
        blind: true,
        pending: Mutex::new(vec![left.key()]),
        ..Fake::default()
    });

    let done = reconciled(&book, &[ring("regar las plantas", "2026-08-12T18:00:00Z")]);

    assert_eq!(done, Reconciled::default());
    assert_eq!(book.held(), vec![left.key()]);
}

#[test]
fn a_system_that_cannot_say_what_it_holds_holds_nothing() {
    let book = Arc::new(Fake {
        unread: true,
        ..Fake::default()
    });

    let done = reconciled(&book, &[ring("comprar pan", "2026-08-12T13:00:00Z")]);

    assert_eq!(done, Reconciled::default());
}

#[test]
fn one_the_system_refuses_is_left_to_the_window_and_the_rest_still_go() {
    let refused = ring("comprar pan", "2026-08-12T13:00:00Z");
    let taken = ring("regar las plantas", "2026-08-12T18:00:00Z");
    let book = Arc::new(Fake {
        refused: vec!["comprar pan".to_string()],
        ..Fake::default()
    });

    let done = reconciled(&book, &[refused, taken.clone()]);

    assert!(!done.whole);
    assert_eq!(done.refused, 1);
    assert_eq!(done.held, vec![taken]);
}

#[test]
fn with_nothing_to_book_the_system_is_not_asked_to_show_but_still_tidied() {
    let gone = ring("pagar la luz", "2026-08-12T09:00:00Z");
    let book = Fake::holding(&[gone.key()]);

    let done = reconciled(&book, &[]);

    assert_eq!(book.asked(), 0);
    assert!(done.whole);
    assert!(book.held().is_empty());
}

#[test]
fn a_renamed_task_a_moved_hour_or_a_muted_sound_is_another_booking() {
    let first = ring("comprar pan", "2026-08-12T13:00:00Z");
    let mut quiet = first.clone();
    quiet.heard = false;

    assert_ne!(
        first.key(),
        ring("comprar pan integral", "2026-08-12T13:00:00Z").key()
    );
    assert_ne!(
        first.key(),
        ring("comprar pan", "2026-08-12T13:30:00Z").key()
    );
    assert_ne!(first.key(), quiet.key());
    assert_eq!(first.key(), first.clone().key());
}

#[test]
fn the_key_carries_its_task_for_the_click_and_fits_a_windows_tag() {
    let key = ring("comprar pan", "2026-08-12T13:00:00Z").key();

    assert!(ours(&key));
    assert_eq!(task_of(&key), Some(TASK));
    assert!(key.len() <= 64, "{key}");
    assert_eq!(task_of("another-app-1"), None);
    assert_eq!(task_of("tisty-0a1b"), None);
}

#[test]
fn only_reminders_become_rings_with_the_words_they_show() {
    let coming = vec![
        due(&ring("comprar pan", "2026-08-12T13:00:00Z")),
        Due {
            at: moment("2026-08-12T14:00:00Z"),
            what: Happening::Missed { count: 4 },
        },
    ];

    let rings = rings(coming, "Recordatorio", false);

    assert_eq!(rings.len(), 1);
    assert_eq!(rings[0].heading, "Recordatorio");
    assert!(!rings[0].heard);
}

#[test]
fn the_same_rings_again_need_no_word_with_the_system() {
    let book = Arc::new(Fake::default());
    let booking = booking(&book);
    let wanted = [ring("comprar pan", "2026-08-12T13:00:00Z")];
    let now = moment("2026-08-12T10:00:00Z");

    booking.keep(&wanted, now);
    booking.keep(&wanted, now);

    assert_eq!(book.asked(), 1);
}

#[test]
fn a_settled_plan_is_checked_with_the_system_again_after_a_while() {
    let book = Arc::new(Fake::default());
    let booking = booking(&book);
    let wanted = [ring("comprar pan", "2026-08-12T13:00:00Z")];

    booking.keep(&wanted, moment("2026-08-12T10:00:00Z"));
    booking.keep(&wanted, moment("2026-08-12T10:05:00Z"));
    booking.keep(&wanted, moment("2026-08-12T10:11:00Z"));

    assert_eq!(book.asked(), 2);
}

#[test]
fn a_booking_the_system_dropped_is_booked_again_at_the_next_check() {
    let book = Arc::new(Fake::default());
    let booking = booking(&book);
    let pan = ring("comprar pan", "2026-08-12T13:00:00Z");
    booking.keep(std::slice::from_ref(&pan), moment("2026-08-12T10:00:00Z"));
    book.pending.lock().unwrap().clear();

    booking.keep(std::slice::from_ref(&pan), moment("2026-08-12T10:11:00Z"));

    assert_eq!(book.held(), vec![pan.key()]);
}

#[test]
fn once_the_system_was_emptied_nothing_is_held_and_a_tisty_still_open_books_again() {
    let book = Arc::new(Fake::default());
    let booking = booking(&book);
    let pan = ring("comprar pan", "2026-08-12T13:00:00Z");
    booking.keep(std::slice::from_ref(&pan), moment("2026-08-12T10:00:00Z"));
    book.pending.lock().unwrap().clear();

    booking.forgotten();
    assert!(!booking.holds(&due(&pan), moment("2026-08-12T13:00:20Z")));

    booking.keep(std::slice::from_ref(&pan), moment("2026-08-12T10:01:00Z"));
    assert_eq!(book.held(), vec![pan.key()]);
}

#[test]
fn two_reminders_of_one_task_at_the_same_second_are_one_ring() {
    let pan = ring("comprar pan", "2026-08-12T13:00:00Z");

    let rings = rings(vec![due(&pan), due(&pan)], "Recordatorio", true);

    assert_eq!(rings, vec![pan]);
}

#[test]
fn a_booking_left_half_done_is_tried_again() {
    let book = Arc::new(Fake {
        refused: vec!["comprar pan".to_string()],
        ..Fake::default()
    });
    let booking = booking(&book);
    let wanted = [ring("comprar pan", "2026-08-12T13:00:00Z")];
    let now = moment("2026-08-12T10:00:00Z");

    booking.keep(&wanted, now);
    booking.keep(&wanted, now);

    assert_eq!(book.asked(), 2);
}

#[test]
fn the_first_look_settles_with_the_system_even_with_nothing_to_book() {
    let gone = ring("pagar la luz", "2026-08-12T09:00:00Z");
    let book = Fake::holding(&[gone.key()]);

    booking(&book).keep(&[], moment("2026-08-12T08:00:00Z"));

    assert!(book.held().is_empty());
}

#[test]
fn a_ring_the_system_holds_stays_off_the_window_while_it_is_fresh() {
    let book = Arc::new(Fake::default());
    let booking = booking(&book);
    let pan = ring("comprar pan", "2026-08-12T13:00:00Z");
    booking.keep(std::slice::from_ref(&pan), moment("2026-08-12T10:00:00Z"));

    assert!(booking.holds(&due(&pan), moment("2026-08-12T13:00:20Z")));
    assert!(!booking.holds(&due(&pan), moment("2026-08-12T13:06:00Z")));
}

#[test]
fn a_ring_the_system_never_took_is_the_window_s_to_ring() {
    let book = Arc::new(Fake {
        refused: vec!["comprar pan".to_string()],
        ..Fake::default()
    });
    let booking = booking(&book);
    let pan = ring("comprar pan", "2026-08-12T13:00:00Z");
    booking.keep(std::slice::from_ref(&pan), moment("2026-08-12T10:00:00Z"));

    assert!(!booking.holds(&due(&pan), moment("2026-08-12T13:00:20Z")));
}

#[test]
fn a_ring_just_delivered_is_still_held_after_the_next_booking() {
    let book = Arc::new(Fake::default());
    let booking = booking(&book);
    let pan = ring("comprar pan", "2026-08-12T13:00:00Z");
    let plants = ring("regar las plantas", "2026-08-12T18:00:00Z");
    booking.keep(
        &[pan.clone(), plants.clone()],
        moment("2026-08-12T10:00:00Z"),
    );

    booking.keep(&[plants], moment("2026-08-12T13:00:05Z"));

    assert!(booking.holds(&due(&pan), moment("2026-08-12T13:00:20Z")));
}

#[test]
fn another_task_at_the_same_hour_is_not_held() {
    let book = Arc::new(Fake::default());
    let booking = booking(&book);
    let pan = ring("comprar pan", "2026-08-12T13:00:00Z");
    booking.keep(std::slice::from_ref(&pan), moment("2026-08-12T10:00:00Z"));
    let mut other = due(&pan);
    other.what = Happening::Due {
        title: "comprar pan".to_string(),
        task: "01KA7B3VJ9RZ0Q8M5N2X4Y6W1V".to_string(),
    };

    assert!(!booking.holds(&other, moment("2026-08-12T13:00:20Z")));
}

#[test]
fn without_a_system_to_book_with_the_window_rings_everything() {
    let booking = Booking::new(None, 64);
    let pan = ring("comprar pan", "2026-08-12T13:00:00Z");
    booking.keep(std::slice::from_ref(&pan), moment("2026-08-12T10:00:00Z"));

    assert!(!booking.books());
    assert!(!booking.holds(&due(&pan), moment("2026-08-12T13:00:20Z")));
}

#[cfg(windows)]
mod toasts {
    use super::ring;
    use crate::booking::there::{NAMED, joined, page, ticks, toast};

    #[test]
    fn a_title_with_markup_stays_text_in_the_toast() {
        let page = toast(&ring("pan & <queso> \"rico\"", "2026-08-12T13:00:00Z"));

        assert!(
            page.contains("pan &amp; &lt;queso&gt; &quot;rico&quot;"),
            "{page}"
        );
        assert!(!page.contains("<queso>"));
    }

    #[test]
    fn the_toast_opens_its_task_through_the_tisty_address() {
        let page = toast(&ring("comprar pan", "2026-08-12T13:00:00Z"));

        assert!(page.contains(r#"launch="tisty://task?id=01KA7B3VJ9RZ0Q8M5N2X4Y6W1T""#));
        assert!(page.contains(r#"activationType="protocol""#));
    }

    #[test]
    fn a_muted_chime_books_a_silent_toast() {
        let mut quiet = ring("comprar pan", "2026-08-12T13:00:00Z");
        quiet.heard = false;

        assert!(toast(&quiet).contains(r#"<audio silent="true"/>"#));
        assert!(!toast(&ring("comprar pan", "2026-08-12T13:00:00Z")).contains("<audio"));
    }

    #[test]
    fn windows_reads_the_toast_whatever_the_title_holds() {
        joined();
        for title in [
            "comprar pan",
            "pan & <queso> \"rico\" 'ya'",
            "comprar\u{1}pan",
            "café ☕ 🍞",
        ] {
            let mut quiet = ring(title, "2026-08-12T13:00:00Z");
            quiet.heard = false;
            assert!(
                page(&ring(title, "2026-08-12T13:00:00Z")).is_ok(),
                "{title}"
            );
            assert!(page(&quiet).is_ok(), "{title}");
        }
    }

    #[test]
    fn a_title_with_control_characters_still_makes_a_toast() {
        let page = toast(&ring("comprar\u{1}pan", "2026-08-12T13:00:00Z"));

        assert!(page.contains("comprarpan"));
    }

    #[test]
    fn windows_counts_its_time_from_1601_in_tenths_of_a_microsecond() {
        assert_eq!(
            ticks(jiff::Timestamp::UNIX_EPOCH).UniversalTime,
            116_444_736_000_000_000
        );
    }

    #[test]
    fn the_name_toasts_go_out_under_is_the_app_s_own() {
        let config = include_str!("../tauri.conf.json");

        assert!(config.contains(&format!("\"identifier\": \"{NAMED}\"")));
    }
}
