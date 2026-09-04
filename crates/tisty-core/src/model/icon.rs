/// Only the names, so a store written by another machine cannot smuggle a drawing in.
#[rustfmt::skip]
pub const ICONS: &[&str] = &[
    "home", "flat", "room", "bed", "sofa", "lamp", "kitchen", "cleaning", "laundry", "iron", "bin",
    "recycle", "build", "tools", "drill", "paint", "plug", "power", "light", "heating", "water",
    "tap", "key", "lock", "bell", "mail", "parcel", "post", "garden", "plant", "flower", "tree",
    "pet", "dog", "cat", "bird", "baby", "family", "friends", "neighbour", "list", "curtain",
    "window", "garage", "balcony", "vacuum", "towel", "mirror", "alarm", "candle", "blanket", "toy",
    "chore", "rubbish", "delivery", "visitor", "siren", "cctv", "boiler", "fan", "aircon", "stairs",
    "hammer-home", "ladder", "shelf", "plant-water", "compost-home", "smoke", "armchair",
    "rocking-chair", "lamp-desk", "lamp-ceiling", "lamp-floor", "lamp-wall", "shelving", "blender",
    "toilet", "towel-rack", "mop", "broom", "soap", "bubbles", "extinguisher", "smoke-alarm",
    "house-plug", "house-wifi", "house-add", "house-heart", "dam", "pole", "solar-panel",
    "ev-charge", "car-battery", "heater", "concierge", "anvil", "axe", "pickaxe", "knife",
    "toolbox", "tool-case", "construction", "traffic-cone", "paint-bucket", "spray",
    "cook", "dishes", "coffee", "tea", "cake", "bread", "food", "fruit", "veg", "meat", "fish",
    "egg", "milk", "wine", "beer", "shopping", "basket", "fridge", "microwave", "cutlery", "pan",
    "recipe", "salad", "soup", "pizza", "sandwich", "icecream", "candy", "popcorn", "glass",
    "kettle", "burger", "ham", "banana", "cherry", "broccoli", "donut", "dessert", "cookie",
    "lollipop", "popsicle", "candy-cane", "drumstick", "shrimp", "fried-egg", "bean", "barrel",
    "bottle", "serve", "gelato", "clover", "milk-off", "beer-off", "fish-off", "beef-off",
    "egg-off", "nut-off", "bean-off", "candy-off", "wine-off", "cannabis", "smoking",
    "work", "office", "desk", "meeting", "deal", "sign", "contract", "invoice", "quote-work",
    "report", "chart", "chartdown", "pie", "target", "goal", "flag", "milestone", "plan",
    "deadline", "sprint", "launch", "review", "approve", "blocked", "risk", "design", "draw",
    "layout", "badge", "trophy", "team", "hire", "support", "ticket-work", "stock", "ship",
    "factory", "store", "till", "interview", "onboard", "payroll", "expense", "client", "pitch",
    "brand", "survey", "legal", "policy", "audit", "swimlane", "standup", "retro", "okr",
    "headcount", "vendor", "contract-work", "bars", "bars-up", "bars-down", "columns-down",
    "columns-stack", "trend-both", "percent-circle", "proportions", "ratio", "grid-table",
    "table-props", "merge-cells", "split-cells",
    "bug", "fix", "code", "branch", "commit", "merge", "test", "build-ci", "deploy", "server",
    "database", "cloud", "api", "terminal", "bugfix", "kanban", "backlog", "estimate", "release",
    "version", "rollback", "log", "oncall", "incident", "spec", "queue", "pullrequest", "backup",
    "fork", "graph", "conflict", "pr-draft", "pr-closed", "pr-new", "compare", "repo", "repo-code",
    "code-file", "code-diff", "code-search", "snippet", "console", "braces", "brackets", "parens",
    "regex", "binary", "hash", "variable", "function", "component", "module", "unpack", "registry",
    "container", "chip", "cpu", "gpu", "memory", "circuit", "pipeline", "automation", "debug",
    "crash", "server-off", "server-add", "server-config", "db-add", "db-check", "db-backup",
    "db-search", "db-drop", "db-fast", "cloud-sync", "cloud-backup", "cloud-off", "cloud-alert",
    "cloud-check", "cloud-config", "cloud-down", "firewall", "wall", "vpn", "dns", "offline",
    "uptime", "latency", "metrics", "tracing", "topology", "gantt", "logs", "inspect", "lint",
    "refactor", "replace", "loop", "loop-back", "shuffle", "issue", "roadmap", "timeline", "diff",
    "scan-code", "scan-text", "scan-search",
    "mobile", "laptop", "print", "scan", "monitor", "network", "router", "drive", "battery", "wifi",
    "signal", "ethernet", "usb", "usb-c", "hdmi", "midi", "bluetooth", "nfc", "antenna",
    "satellite", "dish", "radar", "feed", "webhook-off", "unplug", "socket", "surge", "keyboard",
    "keyboard-off", "mouse", "touchpad", "monitor-check", "monitor-off", "monitor-cloud",
    "monitor-config", "screenshare", "devices", "tablet", "tower-pc", "projector", "webcam",
    "power-off", "power-circle", "battery-low", "battery-warning", "signal-zero", "signal-high",
    "signal-low", "wifi-off", "wifi-sync", "wifi-config",
    "secure", "shield", "lock-thing", "key-thing", "eye-thing", "hide", "qr", "barcode", "id",
    "verified", "secret", "public", "profile", "avatar", "member", "contact", "contact-card",
    "lanyard", "user-add", "user-remove", "user-block", "user-edit", "user-key", "user-lock",
    "user-role", "user-admin", "user-star", "guest", "login", "logout", "session", "token",
    "api-key", "oauth", "encrypt", "decrypt", "permission", "shield-off", "shield-ban",
    "shield-config", "shield-ask", "signature", "fingerprint", "face-scan", "eye-scan", "unlocked",
    "unlock", "door-locked", "key-book", "vote", "verified-badge", "badge-alert", "badge-info",
    "badge-help", "badge-add", "badge-remove", "badge-void", "privacy", "mask", "skull", "bomb",
    "radiation", "biohazard", "shredder", "inspection",
    "call", "video", "mic", "mail-work", "chat", "thread", "send", "email", "chat-bubble",
    "chat-text", "chat-more", "chat-check", "chat-off", "chat-reply", "chat-code", "chat-lock",
    "chat-quote", "chat-warning", "chat-add", "chat-share", "chat-heart", "mail-check", "mail-add",
    "mail-search", "mail-clock", "mail-warning", "mail-void", "mails", "reply", "reply-all",
    "forward", "call-in", "call-out", "call-missed", "call-off", "call-forward", "voicemail",
    "megaphone-off", "speech", "headset",
    "wiki", "slide", "page", "pages", "draft", "folder", "folderdoc", "archive", "inbox",
    "file-add", "file-remove", "file-void", "file-check", "file-down", "file-up", "file-in",
    "file-out", "file-key", "file-lock", "file-scan", "file-search", "file-terminal",
    "file-symlink", "file-stack", "file-clock", "file-user", "file-image", "file-music",
    "file-video", "file-audio", "file-archive", "file-type", "file-badge", "file-chart", "file-ask",
    "file-alert", "folder-add", "folder-remove", "folder-void", "folder-check", "folder-clock",
    "folder-key", "folder-lock", "folder-sync", "folder-tree", "folder-root", "folder-archive",
    "folder-search", "folder-board", "folder-heart", "folder-mark", "folder-in", "folder-out",
    "folder-many", "folder-closed", "save", "save-all", "save-check", "notepad", "sheet", "album",
    "archive-restore", "archive-void",
    "health", "doctor", "hospital", "pill", "syringe", "bandage", "tooth", "eye-care", "ear",
    "heart", "brain", "bone", "lungs", "scale-body", "sleep", "rest", "gym", "run", "walk",
    "bike-sport", "swim", "climb", "yoga", "ball", "football", "tennis", "stretch", "water-drink",
    "vitamin", "mood", "calm", "therapy", "appointment", "prescription", "results", "blood",
    "allergy", "fever", "chill", "physio", "meditate", "breathe", "steps", "weigh", "drink-water",
    "diet", "supplement", "checkup", "glasses", "hearing", "row", "track-fit", "accessible",
    "dentist", "optician", "pharmacy", "emergency", "insurance-health", "medical-bag", "tablets",
    "test-tube", "test-tubes", "beaker", "bone-fracture", "biceps", "scan-heart", "heart-add",
    "heart-off", "heart-crack", "non-binary", "venus", "mars", "transgender", "mixed", "mood-happy",
    "mood-flat", "mood-sad", "angry", "annoyed", "thumbs-up", "thumbs-down",
    "money", "cash", "coin", "bank", "savings", "invest", "loss", "bill", "tax", "wallet", "card",
    "transfer", "salary", "budget", "rent", "mortgage", "insurance", "subscription", "gift",
    "donate", "price", "discount", "crypto", "safe", "chart-money", "invoice-money", "payment",
    "refund", "statement", "account", "atm", "loan", "interest", "fund", "pension", "bill-water",
    "bill-power", "bill-net", "bill-phone", "council", "fine", "deposit", "exchange", "candlestick",
    "combined", "wage", "bonus", "audit-money", "cashflow", "forecast", "dollar", "euro", "pound",
    "yen", "rupee", "ruble", "franc", "lira", "riyal", "peso", "cent", "money-circle",
    "money-check", "money-void", "wallet-flat", "receipt-cent", "receipt-pound", "receipt-yen",
    "offer", "ticket-offer", "ticket-check", "ticket-add", "ticket-void", "meter", "bag",
    "paper-bag", "swatch", "tally",
    "travel", "flight", "landing", "hotel", "camping", "caravan", "beach", "mountain", "map",
    "place", "route", "compass", "luggage", "backpack", "passport", "ticket", "car", "taxi", "bus",
    "train", "tram", "metro", "bike", "scooter", "boat", "ferry", "fuel", "parking", "city",
    "village", "bridge", "sun-away", "snow", "rain", "photo", "souvenir", "booking", "checkin",
    "visa", "customs", "delay", "rental", "charge", "motorway", "trail", "summit", "lake", "island",
    "dive", "ski", "museum", "gallery", "theatre", "concert", "market", "restaurant", "bar", "park",
    "zoo", "guide", "currency", "sim", "telescope", "tour", "cruise", "hostel", "airbnb",
    "roadtrip", "hike", "ambulance", "helicopter", "tractor", "forklift", "truck-electric", "van",
    "motorbike", "cable-car", "bus-front", "tunnel", "roller-coaster", "ferris-wheel", "castle",
    "mosque", "stage", "podium", "lectern", "signpost", "baggage", "tickets", "tickets-plane",
    "map-pinned", "map-add", "pin-house", "pin-check", "pin-off", "locate", "navigation",
    "compass-draft", "binoculars", "parking-off", "ship-wheel", "trainers", "balloon",
    "study", "school", "class", "book", "read", "library", "note", "write", "journal", "idea",
    "question", "answer", "science", "maths", "language", "translate", "history", "law", "game",
    "puzzle", "chess", "cards", "art", "photo-art", "craft-hobby", "sew", "build-model",
    "garden-hobby", "collect", "news", "course", "homework", "exam", "degree", "research", "cite",
    "flashcard", "revise", "timetable", "tutor", "chemistry", "physics", "biology", "geography",
    "astronomy", "code-learn", "draw-learn", "feather", "speak", "tower", "dice", "blocks",
    "shapes", "spool", "bake", "brew", "theatre-mind", "seminar", "mentor", "quiz", "portfolio",
    "certificate", "chess-king", "chess-queen", "chess-rook", "chess-bishop", "chess-knight",
    "chess-pawn", "dice-one", "dice-two", "dice-three", "dice-four", "dice-five", "dice-six",
    "club", "origami", "sticker", "ribbon", "medal", "bow", "toy-brick", "paintbrush", "pencil",
    "pen-off", "wand", "book-heart", "book-audio", "book-check", "book-image", "book-text",
    "book-done", "bookmark-check", "bookmark-add", "shelves", "letter", "type", "spell",
    "text-quote", "text-search", "sword", "swords",
    "music", "play-music", "guitar", "piano", "sing", "film", "tv", "podcast", "listen", "watch",
    "stream", "disc", "disc-album", "cassette", "turntable", "boombox", "speaker", "drum",
    "metronome", "audio-wave", "audio-lines", "volume", "volume-off", "mic-off", "video-off",
    "videotape", "film-strip", "captions", "subtitles", "airplay", "remote", "tv-play", "carousel",
    "thumbnails", "image-add", "image-off", "image-play", "images", "wallpaper", "aperture", "lens",
    "focus", "flashlight", "spotlight", "drone", "joystick", "gamepad", "dpad", "vr", "wristwatch",
    "sun", "moon", "star", "cloud-sky", "storm", "wind", "leaf", "forest", "mountain-wild", "sea",
    "river", "fire", "earth", "harvest", "seed", "bee", "butterfly", "fish-wild", "bird-wild",
    "season", "compost", "solar", "dawn", "dusk", "night", "fog", "hail", "drizzle", "rainbow",
    "volcano", "field", "orchard", "grape", "vegan", "nest", "shell", "orbit", "rabbit", "snail",
    "turtle", "squirrel", "rat", "cow", "wheat-off", "frost", "bloom", "root", "horizon", "cliff",
    "cloudy", "haze", "tornado", "sun-dim", "sun-moon", "sun-snow", "cloud-sun", "cloud-moon",
    "cloud-snow", "cloud-wind", "waves-up", "pine", "oak", "shrub", "rose", "worm", "panda", "land",
    "stone", "eclipse", "globe-earth", "pool", "fishing", "hook", "kayak", "tent-tree", "road",
    "aries", "taurus", "gemini", "cancer", "leo", "virgo", "libra", "scorpio", "sagittarius",
    "capricorn", "aquarius", "pisces", "ophiuchus",
    "clip", "link", "bookmark", "pin", "label", "search", "filter", "sort", "calendar", "clock",
    "timer", "waiting", "repeat", "pause", "done", "todo", "urgent", "alert", "info", "share",
    "copy", "cut", "settings", "tune", "box", "layers", "puzzle-thing", "anchor", "magnet",
    "umbrella", "crown-thing", "gem", "rocket-thing", "bolt", "sticky", "stack", "upload",
    "download", "export", "import", "template", "warning", "help", "robot", "ghost", "heart-thing",
    "bot-off", "star-half", "star-add", "star-off", "star-check", "circle-star", "square-star",
    "time-back", "clock-check", "clock-add", "clock-fading", "alarm-off", "alarm-add", "timer-off",
    "timer-reset", "calendar-add", "calendar-void", "calendar-sync", "calendar-search",
    "calendar-config", "calendar-fold", "calendars", "bell-ring", "bell-off", "bell-add",
    "bell-dot", "bell-electric", "sticky-add", "sticky-check", "sticky-many", "list-check",
    "list-add", "list-clock", "list-music", "list-video", "list-filter", "list-restart", "list-up",
    "list-down", "tags", "tag-add", "link-off", "link-two", "hand", "hand-heart",
    "ruler", "grid-thing", "split", "undo", "redo", "refresh", "sync", "gridview", "zoom",
    "fullscreen", "close", "more", "menu", "drag", "rotate", "flip", "crop", "colour", "brush",
    "eraser", "pen", "stamp", "scaling", "split-square", "toggle", "slider", "switch",
    "radio-button", "checkbox", "input", "filter-thing", "group-thing", "ungroup", "align", "space",
    "frame", "sum", "pi", "infinity", "asterisk", "ampersand", "section", "copyright", "copyleft",
    "commons", "vector", "spline", "cube", "cone", "cylinder", "pyramid", "torus", "pentagon",
    "octagon", "diamond", "ellipse", "squircle", "rectangle", "point", "crosshair", "blend",
    "contrast", "pipette", "lasso", "square", "hexagon", "triangle", "triangle-dashed",
    "circle-dashed", "squircle-dashed", "rectangle-tall", "astroid", "dot", "slash", "crossed-out",
    "circle-off", "square-off", "circle-add", "circle-remove", "circle-void", "square-void",
    "octagon-void", "diamond-add", "diamond-remove", "unite", "subtract", "intersect", "exclude",
    "combine", "box-select", "scan-box", "grid-four", "grid-check", "grid-six", "up", "down",
    "left", "right", "swap", "step-back", "step-on", "return", "send-right", "pointer-arrow",
    "check-mark", "check-line", "delete-back", "flag-left", "flag-right", "flag-off", "badge-plain",
    "circle-ellipsis", "ellipsis-tall", "circle-pile", "shield-half", "shield-ellipsis",
    "search-alert", "search-void", "filter-void", "funnel-add", "star-remove", "star-void",
    "heart-void", "unpin", "equal", "not-equal", "approx", "divide", "radical", "omega", "phi",
    "tangent", "radius", "diameter", "angle", "weight", "weight-tilde", "scale-space",
    "rotate-space", "ruler-line", "dimensions", "bold", "italic", "underline", "strike",
    "superscript", "subscript", "pilcrow", "baseline", "ligature", "case-upper", "case-lower",
    "case-sensitive", "whole-word", "text-wrap", "text-cursor", "remove-format", "heading",
    "letter-size", "type-letter", "tally-one", "tally-two", "tally-three", "tally-four",
    "decimals-left", "file-digit", "play", "play-off", "pause-circle", "stop-circle", "skip-back",
    "skip-on", "rewind", "fast-forward", "repeat-one", "repeat-off", "loader", "spinner", "expand",
    "shrink", "minimise", "screen-full", "zoom-out", "picture-in-picture", "pointer", "click",
    "grab", "grip", "grip-flat", "fist", "metal", "lasso-pick", "brain-circuit", "squiggle",
    "line-style", "line-dotted", "mirror-round", "mirror-flat", "view", "share-plain",
    "merge-arrows", "slice", "cog", "settings-two", "command", "option", "power-plain",
    "bolt-plain", "currency-sign", "contents", "summary", "layout-plain", "list-layout", "form",
    "field-input", "separator", "stretch-wide", "square-stack", "layers-add", "layers-remove",
    "copy-check", "copy-add", "clipboard", "clipboard-copy", "clipboard-paste", "clipboard-clock",
];

/// How many of ICONS each family takes, in order: the window rules the picker off these cuts.
pub const FAMILIES: &[(&str, usize)] = &[
    ("home", 105),
    ("table", 62),
    ("work", 70),
    ("dev", 106),
    ("gear", 51),
    ("account", 65),
    ("talk", 40),
    ("file", 63),
    ("body", 81),
    ("money", 78),
    ("away", 103),
    ("study", 101),
    ("media", 50),
    ("wild", 89),
    ("thing", 91),
    ("symbol", 226),
];

const MARK_AT_MOST: usize = 8;

pub fn known(key: &str) -> bool {
    kept(key).is_some()
}

pub fn kept(key: &str) -> Option<String> {
    if let Some(named) = ICONS.iter().copied().find(|named| *named == key) {
        return Some(named.to_string());
    }
    a_mark(key).then(|| key.to_string())
}

pub fn a_mark(key: &str) -> bool {
    let many = key.chars().count();
    if many == 0 || many > MARK_AT_MOST {
        return false;
    }
    if key.chars().any(|one| one as u32 == 0x20E3) {
        return keyed(key);
    }
    key.chars().next().is_some_and(drawn) && key.chars().all(|one| drawn(one) || joins(one))
}

fn keyed(key: &str) -> bool {
    let capped = |one: char| one.is_ascii_digit() || one == '#' || one == '*';
    key.chars().next().is_some_and(capped) && key.chars().all(|one| capped(one) || joins(one))
}

fn drawn(one: char) -> bool {
    matches!(one as u32,
        0x00A9 | 0x00AE | 0x203C | 0x2049 | 0x2122 | 0x2139
        | 0x2194..=0x21AA | 0x231A..=0x231B | 0x2328
        | 0x23CF..=0x23FA | 0x24C2 | 0x25AA..=0x25FE
        | 0x2600..=0x27BF | 0x2934..=0x2935 | 0x2B00..=0x2BFF
        | 0x3030 | 0x303D | 0x3297 | 0x3299
        | 0x1F000..=0x1FAFF)
}

fn joins(one: char) -> bool {
    matches!(one as u32, 0xFE0F | 0x200D | 0x20E3 | 0x1F3FB..=0x1F3FF | 0xE0020..=0xE007F)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_name_outside_the_catalogue_is_refused() {
        assert!(kept("unicorn").is_none());
        assert_eq!(kept("home").as_deref(), Some("home"));
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

    #[test]
    fn the_families_cover_the_catalogue_exactly_once() {
        let counted: usize = FAMILIES.iter().map(|(_, many)| many).sum();
        assert_eq!(counted, ICONS.len());
    }

    #[test]
    fn no_family_is_empty_and_none_is_named_twice() {
        let mut seen: Vec<&str> = FAMILIES.iter().map(|(name, _)| *name).collect();
        let many = seen.len();
        seen.sort_unstable();
        seen.dedup();
        assert_eq!(seen.len(), many);
        assert!(FAMILIES.iter().all(|(_, held)| *held > 0));
    }
}
