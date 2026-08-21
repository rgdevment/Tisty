pub const ICONS: &[(&str, &str)] = &[
    ("home", "🏠"),
    ("garden", "🪴"),
    ("bed", "🛏"),
    ("kitchen", "🍳"),
    ("cleaning", "🧹"),
    ("laundry", "🧺"),
    ("build", "🔧"),
    ("place", "📍"),
    ("city", "🏙"),
    ("beach", "🏖"),
    ("camping", "⛺"),
    ("mountain", "⛰"),
    ("work", "💼"),
    ("meeting", "👥"),
    ("deal", "🤝"),
    ("support", "🛟"),
    ("chart", "📈"),
    ("target", "🎯"),
    ("mail", "✉️"),
    ("phone", "📞"),
    ("print", "🖨"),
    ("clip", "📎"),
    ("code", "💻"),
    ("bug", "🐞"),
    ("test", "🧪"),
    ("review", "👀"),
    ("rocket", "🚀"),
    ("server", "🗄"),
    ("database", "🧮"),
    ("design", "🎨"),
    ("health", "🩺"),
    ("hospital", "🏥"),
    ("pill", "💊"),
    ("sport", "🏃"),
    ("gym", "🏋"),
    ("yoga", "🧘"),
    ("sleep", "😴"),
    ("mind", "🧠"),
    ("tooth", "🦷"),
    ("money", "💳"),
    ("bank", "🏦"),
    ("receipt", "🧾"),
    ("savings", "🐷"),
    ("coin", "🪙"),
    ("shopping", "🛒"),
    ("gift", "🎁"),
    ("package", "📦"),
    ("travel", "✈️"),
    ("hotel", "🏨"),
    ("map", "🗺"),
    ("ticket", "🎫"),
    ("luggage", "🧳"),
    ("car", "🚗"),
    ("bike", "🚲"),
    ("train", "🚆"),
    ("study", "📚"),
    ("note", "📝"),
    ("science", "🔬"),
    ("language", "🗣"),
    ("music", "🎧"),
    ("film", "🎬"),
    ("game", "🎮"),
    ("camera", "📷"),
    ("art", "🖌"),
    ("food", "🍽"),
    ("coffee", "☕"),
    ("cake", "🎂"),
    ("drink", "🍷"),
    ("family", "👪"),
    ("baby", "🍼"),
    ("pet", "🐾"),
    ("plant", "🌱"),
    ("sun", "☀️"),
    ("water", "💧"),
    ("folder", "🗂"),
    ("archive", "🗃"),
    ("inbox", "📥"),
    ("calendar", "📅"),
    ("clock", "⏰"),
    ("star", "⭐"),
    ("flag", "🚩"),
    ("done", "✅"),
    ("todo", "☑️"),
    ("repeat", "🔁"),
    ("timer", "⏱"),
    ("waiting", "⏳"),
    ("bell", "🔔"),
    ("pause", "⏸"),
    ("urgent", "❗"),
    ("question", "❓"),
    ("info", "ℹ️"),
    ("blocked", "⛔"),
    ("idea", "💡"),
    ("alert", "⚠️"),
    ("lock", "🔒"),
    ("key", "🔑"),
    ("book", "📖"),
    ("law", "⚖️"),
    ("vote", "🗳"),
    ("contract", "📜"),
    ("write", "✍️"),
    ("page", "📄"),
    ("pages", "📑"),
    ("draft", "🗒"),
    ("clipboard", "📋"),
    ("bookmark", "🔖"),
    ("label", "🏷"),
    ("link", "🔗"),
    ("pin", "📌"),
    ("cut", "✂️"),
    ("ruler", "📐"),
    ("search", "🔍"),
    ("folderdoc", "📂"),
    ("chat", "💬"),
    ("call", "📱"),
    ("video", "🎥"),
    ("mic", "🎙"),
    ("tv", "📺"),
    ("photo", "🖼"),
    ("cloud", "☁️"),
    ("wifi", "📶"),
    ("battery", "🔋"),
    ("plug", "🔌"),
    ("tools", "🛠"),
    ("paint", "🎭"),
    ("trash", "🗑"),
    ("shield", "🛡"),
    ("fire", "🔥"),
    ("moon", "🌙"),
    ("leaf", "🍃"),
    ("tree", "🌳"),
    ("harvest", "🌾"),
    ("bread", "🍞"),
    ("fruit", "🍎"),
    ("veg", "🥕"),
    ("recycle", "♻️"),
    ("snow", "❄️"),
    ("fish", "🐟"),
    ("chartdown", "📉"),
    ("badge", "🎖"),
    ("trophy", "🏆"),
    ("puzzle", "🧩"),
    ("guitar", "🎸"),
    ("dance", "💃"),
    ("swim", "🏊"),
    ("climb", "🧗"),
    ("ball", "⚽"),
    ("boat", "⛵"),
    ("bus", "🚌"),
    ("school", "🏫"),
    ("factory", "🏭"),
    ("store", "🏪"),
    ("office", "🏢"),
];

pub fn drawn(key: &str) -> Option<&'static str> {
    ICONS
        .iter()
        .find(|(named, _)| *named == key)
        .map(|(_, glyph)| *glyph)
}

pub fn known(key: &str) -> bool {
    drawn(key).is_some()
}

pub fn kept(key: &str) -> Option<&'static str> {
    ICONS
        .iter()
        .find(|(named, _)| *named == key)
        .map(|(named, _)| *named)
}

#[cfg(test)]
mod tests {
    #[test]
    fn no_two_icons_answer_to_the_same_name() {
        let mut seen = std::collections::BTreeSet::new();
        for (key, _) in ICONS {
            assert!(seen.insert(*key), "two icons called {key}");
        }
    }

    #[test]
    fn no_two_names_draw_the_same_thing() {
        let mut seen = std::collections::BTreeMap::new();
        for (key, glyph) in ICONS {
            if let Some(was) = seen.insert(*glyph, *key) {
                panic!("{was} and {key} both draw {glyph}");
            }
        }
    }

    #[test]
    fn every_name_is_a_plain_lowercase_word() {
        for (key, _) in ICONS {
            assert!(
                !key.is_empty() && key.chars().all(|c| c.is_ascii_lowercase()),
                "awkward name: {key}"
            );
        }
    }

    #[test]
    fn what_a_document_needs_is_there() {
        for wanted in ["write", "page", "clipboard", "bookmark", "link", "search"] {
            assert!(known(wanted), "missing: {wanted}");
        }
    }

    #[test]
    fn what_a_list_needs_is_there() {
        for wanted in ["done", "todo", "repeat", "waiting", "urgent", "blocked"] {
            assert!(known(wanted), "missing: {wanted}");
        }
    }

    use super::*;
    use std::collections::HashSet;

    #[test]
    fn every_key_is_plain_ascii_so_a_file_name_never_carries_a_drawing() {
        for (key, _) in ICONS {
            assert!(
                key.chars().all(|c| c.is_ascii_lowercase()),
                "{key} is not plain"
            );
        }
    }

    #[test]
    fn no_key_and_no_glyph_is_used_twice() {
        let keys: HashSet<&str> = ICONS.iter().map(|(key, _)| *key).collect();
        let glyphs: HashSet<&str> = ICONS.iter().map(|(_, glyph)| *glyph).collect();

        assert_eq!(keys.len(), ICONS.len(), "a key is repeated");
        assert_eq!(glyphs.len(), ICONS.len(), "a drawing is repeated");
    }

    #[test]
    fn a_key_nobody_ships_is_refused_rather_than_drawn() {
        assert!(!known("dragon"));
        assert!(!known(""));
        assert!(!known("HOME"));
        assert_eq!(drawn("nope"), None);
    }

    #[test]
    fn a_key_that_ships_answers_with_its_drawing() {
        assert_eq!(drawn("home"), Some("🏠"));
        assert_eq!(kept("home"), Some("home"));
    }

    #[test]
    fn there_are_enough_to_choose_from_without_scrolling_for_ever() {
        assert!(ICONS.len() >= 60, "only {}", ICONS.len());
        assert!(ICONS.len() <= 400, "more than anyone would sift through");
    }
}
