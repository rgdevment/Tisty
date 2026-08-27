/// Only the names, so a store written by another machine cannot smuggle a drawing in.
#[rustfmt::skip]
pub const ICONS: &[&str] = &[
    "home", "flat", "room", "bed", "sofa", "lamp", "kitchen", "cook", "cleaning", "laundry",
    "iron", "dishes", "bin", "recycle", "build", "tools", "drill", "paint", "plug", "power",
    "light", "heating", "water", "tap", "key", "lock", "bell", "mail", "parcel", "post",
    "garden", "plant", "flower", "tree", "pet", "dog", "cat", "bird", "baby", "family",
    "friends", "neighbour", "coffee", "tea", "cake", "bread", "food", "fruit", "veg", "meat",
    "fish", "egg", "milk", "wine", "beer", "shopping", "basket", "list", "fridge", "microwave",
    "curtain", "window", "garage", "balcony", "vacuum", "towel", "mirror", "alarm", "candle",
    "blanket", "toy", "chore", "rubbish", "delivery", "visitor", "siren", "cctv", "boiler",
    "fan", "aircon", "cutlery", "pan", "recipe", "salad", "soup", "pizza", "sandwich",
    "icecream", "candy", "popcorn", "glass", "kettle", "stairs", "hammer-home", "ladder",
    "shelf", "plant-water", "compost-home", "smoke",
    "work", "office", "desk", "meeting", "call", "video", "mic", "mail-work", "chat", "thread",
    "send", "deal", "sign", "contract", "invoice", "quote-work", "report", "chart", "chartdown",
    "pie", "target", "goal", "flag", "milestone", "plan", "deadline", "sprint", "launch",
    "review", "approve", "blocked", "risk", "bug", "fix", "code", "branch", "commit", "merge",
    "test", "build-ci", "deploy", "server", "database", "cloud", "api", "terminal", "bugfix",
    "design", "draw", "layout", "mobile", "laptop", "print", "scan", "badge", "trophy", "team",
    "hire", "support", "ticket-work", "stock", "ship", "factory", "store", "till", "email",
    "kanban", "backlog", "estimate", "release", "version", "rollback", "monitor", "log",
    "oncall", "incident", "spec", "wiki", "slide", "interview", "onboard", "payroll", "expense",
    "client", "pitch", "brand", "survey", "legal", "policy", "audit", "secure", "network",
    "router", "queue", "pullrequest", "drive", "backup", "swimlane", "standup", "retro", "okr",
    "headcount", "vendor", "contract-work",
    "health", "doctor", "hospital", "pill", "syringe", "bandage", "tooth", "eye-care", "ear",
    "heart", "brain", "bone", "lungs", "scale-body", "sleep", "rest", "gym", "run", "walk",
    "bike-sport", "swim", "climb", "yoga", "ball", "football", "tennis", "stretch",
    "water-drink", "vitamin", "mood", "calm", "therapy", "appointment", "prescription",
    "results", "blood", "allergy", "fever", "chill", "physio", "meditate", "breathe", "steps",
    "weigh", "drink-water", "diet", "supplement", "checkup", "glasses", "hearing", "row",
    "track-fit", "accessible", "dentist", "optician", "pharmacy", "emergency",
    "insurance-health",
    "money", "cash", "coin", "bank", "savings", "invest", "loss", "bill", "tax", "wallet",
    "card", "transfer", "salary", "budget", "rent", "mortgage", "insurance", "subscription",
    "gift", "donate", "price", "discount", "crypto", "safe", "chart-money", "invoice-money",
    "payment", "refund", "statement", "account", "atm", "loan", "interest", "fund", "pension",
    "bill-water", "bill-power", "bill-net", "bill-phone", "council", "fine", "deposit",
    "exchange", "candlestick", "combined", "wage", "bonus", "audit-money", "cashflow",
    "forecast",
    "travel", "flight", "landing", "hotel", "camping", "caravan", "beach", "mountain", "map",
    "place", "route", "compass", "luggage", "backpack", "passport", "ticket", "car", "taxi",
    "bus", "train", "tram", "metro", "bike", "scooter", "boat", "ferry", "fuel", "parking",
    "city", "village", "bridge", "sun-away", "snow", "rain", "photo", "souvenir", "booking",
    "checkin", "visa", "customs", "delay", "rental", "charge", "motorway", "trail", "summit",
    "lake", "island", "dive", "ski", "museum", "gallery", "theatre", "concert", "market",
    "restaurant", "bar", "park", "zoo", "guide", "currency", "sim", "telescope", "tour",
    "cruise", "hostel", "airbnb", "roadtrip", "hike",
    "study", "school", "class", "book", "read", "library", "note", "write", "journal", "idea",
    "question", "answer", "science", "maths", "language", "translate", "history", "law",
    "music", "play-music", "guitar", "piano", "sing", "film", "tv", "game", "puzzle", "chess",
    "cards", "art", "photo-art", "craft-hobby", "sew", "build-model", "garden-hobby", "collect",
    "podcast", "news", "course", "homework", "exam", "degree", "research", "cite", "flashcard",
    "revise", "timetable", "tutor", "chemistry", "physics", "biology", "geography", "astronomy",
    "code-learn", "draw-learn", "feather", "speak", "listen", "watch", "stream", "tower",
    "dice", "blocks", "shapes", "spool", "bake", "brew", "theatre-mind", "seminar", "mentor",
    "quiz", "portfolio", "certificate",
    "sun", "moon", "star", "cloud-sky", "storm", "wind", "leaf", "forest", "mountain-wild",
    "sea", "river", "fire", "earth", "harvest", "seed", "bee", "butterfly", "fish-wild",
    "bird-wild", "season", "compost", "solar", "dawn", "dusk", "night", "fog", "hail",
    "drizzle", "rainbow", "volcano", "field", "orchard", "grape", "vegan", "nest", "shell",
    "orbit", "rabbit", "snail", "turtle", "squirrel", "rat", "cow", "wheat-off", "frost",
    "bloom", "root", "horizon", "cliff",
    "page", "pages", "draft", "folder", "folderdoc", "archive", "inbox", "clip", "link",
    "bookmark", "pin", "label", "search", "filter", "sort", "calendar", "clock", "timer",
    "waiting", "repeat", "pause", "done", "todo", "urgent", "alert", "info", "shield",
    "lock-thing", "key-thing", "eye-thing", "hide", "share", "copy", "cut", "ruler", "battery",
    "wifi", "signal", "settings", "tune", "grid-thing", "box", "layers", "puzzle-thing",
    "anchor", "magnet", "umbrella", "crown-thing", "gem", "rocket-thing", "bolt", "sticky",
    "stack", "split", "undo", "redo", "refresh", "sync", "upload", "download", "export",
    "import", "template", "gridview", "zoom", "fullscreen", "close", "more", "menu", "drag",
    "rotate", "flip", "crop", "colour", "brush", "eraser", "pen", "stamp", "qr", "barcode",
    "id", "verified", "warning", "help", "secret", "public", "robot", "ghost", "heart-thing",
    "bot-off", "scaling", "split-square", "toggle", "slider", "switch", "radio-button",
    "checkbox", "input", "filter-thing", "group-thing", "ungroup", "align", "space", "frame",
];

pub fn known(key: &str) -> bool {
    kept(key).is_some()
}

pub fn kept(key: &str) -> Option<&'static str> {
    ICONS.iter().copied().find(|named| *named == key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_name_outside_the_catalogue_is_refused() {
        assert!(kept("unicorn").is_none());
        assert_eq!(kept("home"), Some("home"));
    }

    #[test]
    fn every_key_is_lowercase_ascii_so_it_travels_between_machines() {
        assert!(
            ICONS
                .iter()
                .all(|key| key.chars().all(|c| c.is_ascii_lowercase() || c == '-'))
        );
    }

    #[test]
    fn no_key_is_written_twice() {
        let mut seen: Vec<&str> = ICONS.to_vec();
        seen.sort_unstable();
        let many = seen.len();
        seen.dedup();
        assert_eq!(seen.len(), many);
    }
}
