use jiff::{Zoned, civil::Date};
use tisty_core::{DateSpec, List, Priority, State, Status, Task};

use crate::i18n::Lang;
use crate::style::{self, BLUE, GREEN, RED, YELLOW};

pub fn list(tasks: &[&Task], state: &State, heading: &str, today: Date, lang: Lang) -> String {
    if tasks.is_empty() {
        return format!("\n  {}\n\n", style::dim(lang.get("nothing-here")));
    }

    let mut out = format!(
        "\n  {}{}\n\n",
        style::bold(heading),
        style::dim(&format!(
            "{:>width$}",
            lang.plural("tasks", tasks.len()),
            width = 62usize.saturating_sub(heading.len())
        ))
    );

    for (i, task) in tasks.iter().enumerate() {
        out.push_str(&row(i + 1, task, state, today, lang));
    }
    out.push('\n');
    out
}

fn row(number: usize, task: &Task, state: &State, today: Date, lang: Lang) -> String {
    let mut out = format!(
        "  {:>3}  {} {}",
        style::dim(&number.to_string()),
        marker(task),
        task.title
    );

    let meta = meta(task, state, today, lang);
    if !meta.is_empty() {
        out.push_str(&format!("\n       {meta}"));
    }
    out.push('\n');
    out
}

pub fn line(task: &Task, state: &State, today: Date, lang: Lang) -> String {
    let mut out = format!("  {} {}\n", marker(task), task.title);

    let meta = meta(task, state, today, lang);
    if !meta.is_empty() {
        out.push_str(&format!("    {meta}\n"));
    }
    out
}

fn meta(task: &Task, state: &State, today: Date, lang: Lang) -> String {
    let mut meta = Vec::new();
    if let Some(p) = priority(task.priority) {
        meta.push(p);
    }
    if let Some(d) = &task.date {
        meta.push(when(d, today, lang));
    }
    if let Some(d) = &task.deadline {
        meta.push(style::paint(
            RED,
            &format!("{} {}", lang.get("deadline"), short(d.date(), today, lang)),
        ));
    }
    if let Some(list) = task.list.and_then(|id| state.lists.get(&id)) {
        meta.push(style::dim(&format!("#{}", slug(&list.name))));
    }
    for tag in &task.tags {
        meta.push(style::dim(&format!("@{tag}")));
    }
    let (done, total) = task.steps_done();
    if total > 0 {
        meta.push(style::dim(&format!("{done}/{total}")));
    }
    let entries = task.journal().count();
    if entries > 0 {
        meta.push(style::dim(&format!("✎{entries}")));
    }
    meta.join(" · ")
}

pub fn detail(task: &Task, state: &State, today: Date, lang: Lang) -> String {
    let mut out = format!(
        "\n  {} {}{}\n",
        marker(task),
        style::bold(&task.title),
        style::dim(&format!("  {}", short_id(task)))
    );

    let mut meta = Vec::new();
    if let Some(d) = &task.date {
        meta.push(when(d, today, lang));
    }
    if let Some(d) = &task.deadline {
        meta.push(format!(
            "{} {}",
            lang.get("deadline"),
            short(d.date(), today, lang)
        ));
    }
    if let Some(p) = priority(task.priority) {
        meta.push(p);
    }
    if let Some(list) = task.list.and_then(|id| state.lists.get(&id)) {
        meta.push(format!("#{}", slug(&list.name)));
    }
    meta.extend(task.tags.iter().map(|t| format!("@{t}")));
    if !task.reminders.is_empty() {
        meta.push(format!("⏰ {}", task.reminders.len()));
    }
    if !meta.is_empty() {
        out.push_str(&format!("    {}\n", meta.join(" · ")));
    }

    if let Some(description) = &task.description {
        out.push_str(&section(lang.get("description"), None));
        for line in description.lines() {
            out.push_str(&format!("    {line}\n"));
        }
    }

    if !task.steps.is_empty() {
        let (done, total) = task.steps_done();
        out.push_str(&section(
            lang.get("steps"),
            Some(&format!("{done}/{total}")),
        ));
        for (i, step) in task.steps.iter().enumerate() {
            let mark = if step.done {
                style::paint(GREEN, "✓")
            } else {
                "○".into()
            };
            let text = if step.done {
                style::dim(&step.text)
            } else {
                step.text.clone()
            };
            out.push_str(&format!(
                "  {:>3} {mark} {text}\n",
                style::dim(&(i + 1).to_string())
            ));
        }
    }

    let journal: Vec<_> = task.journal().collect();
    if !journal.is_empty() {
        out.push_str(&section(
            lang.get("journal"),
            Some(&journal.len().to_string()),
        ));
        for entry in journal {
            let z = entry.zoned();
            let stamp = format!(
                "{} {} {} · {}",
                lang.weekday(weekday_index(z.date())),
                z.day(),
                lang.month(z.month() as u8),
                z.strftime("%H:%M")
            );
            out.push_str(&format!("    {}\n", style::dim(&stamp)));
            for line in entry.body.lines() {
                out.push_str(&format!("      {line}\n"));
            }
        }
    }

    out.push('\n');
    out
}

pub fn captured(task: &Task, state: &State, today: Date, lang: Lang) -> String {
    let mut out = format!(
        "\n  {} {}\n",
        style::paint(GREEN, "✓"),
        style::bold(&task.title)
    );

    let mut meta = Vec::new();
    if let Some(d) = &task.date {
        meta.push(when(d, today, lang));
    }
    if let Some(d) = &task.deadline {
        meta.push(format!(
            "{} {}",
            lang.get("deadline"),
            short(d.date(), today, lang)
        ));
    }
    if let Some(p) = priority(task.priority) {
        meta.push(p);
    }
    if let Some(list) = task.list.and_then(|id| state.lists.get(&id)) {
        meta.push(format!("#{}", slug(&list.name)));
    }
    meta.extend(task.tags.iter().map(|t| format!("@{t}")));

    if !meta.is_empty() {
        out.push_str(&format!("    {}\n", meta.join(" · ")));
    }
    out.push_str(&format!("    {}\n\n", style::dim(&short_id(task))));
    out
}

pub fn lists(state: &State, lang: Lang) -> String {
    let mut active: Vec<&List> = state.active_lists().collect();
    if active.is_empty() {
        return format!("\n  {}\n\n", style::dim(lang.get("no-lists-yet")));
    }
    active.sort_by(|a, b| a.order.cmp(&b.order));

    let mut out = format!("\n  {}\n\n", style::bold(lang.get("lists")));
    for list in active {
        let open = state.tasks_in(list.id).count();
        let settled = if open == 0 {
            style::dim("  ✓")
        } else {
            String::new()
        };
        out.push_str(&format!(
            "    {:<32}{}{}\n",
            list.name,
            style::dim(&open.to_string()),
            settled
        ));
    }
    out.push('\n');
    out
}

/// Absolute dates, never «tomorrow»: an export is read months later.
pub fn markdown(tasks: &[&Task], state: &State, heading: &str, lang: Lang) -> String {
    let mut out = format!("# {heading}\n\n");
    if tasks.is_empty() {
        out.push_str(&format!("{}\n", lang.get("nothing-here")));
        return out;
    }

    for task in tasks {
        let mark = match task.status {
            Status::Open => " ",
            Status::Done => "x",
            Status::Dropped => "-",
        };
        out.push_str(&format!("## [{mark}] {}\n\n", task.title));

        let mut meta = Vec::new();
        if let Some(d) = &task.date {
            meta.push(stamp(d));
        }
        if let Some(d) = &task.deadline {
            meta.push(format!("{} {}", lang.get("deadline"), stamp(d)));
        }
        if let Some(at) = task.completed_at {
            meta.push(format!(
                "{} {}",
                lang.get("completed"),
                at.to_zoned(jiff::tz::TimeZone::system())
                    .strftime("%Y-%m-%d")
            ));
        }
        if task.priority != Priority::P4 {
            meta.push(format!("!{}", u8::from(task.priority)));
        }
        if let Some(list) = task.list.and_then(|id| state.lists.get(&id)) {
            meta.push(format!("#{}", slug(&list.name)));
        }
        meta.extend(task.tags.iter().map(|t| format!("@{t}")));
        if !meta.is_empty() {
            out.push_str(&format!("{}\n\n", meta.join(" · ")));
        }

        if let Some(description) = &task.description {
            out.push_str(&format!("{description}\n\n"));
        }

        if !task.steps.is_empty() {
            out.push_str(&format!("### {}\n\n", lang.get("steps")));
            for step in &task.steps {
                let tick = if step.done { "x" } else { " " };
                out.push_str(&format!("- [{tick}] {}\n", step.text));
            }
            out.push('\n');
        }

        let journal: Vec<_> = task.journal().collect();
        if !journal.is_empty() {
            out.push_str(&format!("### {}\n\n", lang.get("journal")));
            for entry in journal {
                // The offset places the entry on a timeline without a lookup.
                let at = entry.zoned();
                out.push_str(&format!("**{}**\n\n", at.strftime("%Y-%m-%d %H:%M %:z")));
                out.push_str(&format!("{}\n\n", entry.body));
            }
        }
    }
    out
}

fn stamp(spec: &DateSpec) -> String {
    if spec.has_time {
        spec.at.strftime("%Y-%m-%d %H:%M").to_string()
    } else {
        spec.at.strftime("%Y-%m-%d").to_string()
    }
}

fn section(name: &str, count: Option<&str>) -> String {
    match count {
        Some(c) => format!("\n  {}  {}\n", style::dim(name), style::dim(c)),
        None => format!("\n  {}\n", style::dim(name)),
    }
}

fn marker(task: &Task) -> String {
    match task.status {
        Status::Open => "○".into(),
        Status::Done => style::paint(GREEN, "✓"),
        Status::Dropped => style::dim("✕"),
    }
}

fn priority(p: Priority) -> Option<String> {
    match p {
        Priority::P1 => Some(style::paint(RED, "!1")),
        Priority::P2 => Some(style::paint(YELLOW, "!2")),
        Priority::P3 => Some(style::paint(BLUE, "!3")),
        Priority::P4 => None,
    }
}

fn when(spec: &DateSpec, today: Date, lang: Lang) -> String {
    let label = short(spec.date(), today, lang);
    let text = if spec.has_time {
        format!("{label} {}", spec.at.strftime("%H:%M"))
    } else {
        label
    };
    if spec.date() < today {
        style::paint(RED, &text)
    } else {
        text
    }
}

fn short(date: Date, today: Date, lang: Lang) -> String {
    match (date - today).get_days() {
        0 => lang.get("today").into(),
        1 => lang.get("tomorrow").into(),
        -1 => lang.get("yesterday").into(),
        d if (0..7).contains(&d) => lang.weekday(weekday_index(date)).to_string(),
        _ => format!("{} {}", date.day(), lang.month(date.month() as u8)),
    }
}

fn weekday_index(date: Date) -> u8 {
    date.weekday().to_monday_one_offset() as u8
}

fn short_id(task: &Task) -> String {
    let id = task.id.to_string();
    id[id.len() - 6..].to_lowercase()
}

fn slug(name: &str) -> String {
    name.to_lowercase().replace(' ', "-")
}

pub fn today() -> Date {
    Zoned::now().date()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tisty_core::Tag;
    use ulid::Ulid;

    fn task(title: &str) -> Task {
        Task::new(Ulid::generate(), title, "a0")
    }

    fn day(s: &str) -> Date {
        s.parse().unwrap()
    }

    #[test]
    fn relative_days_read_naturally() {
        let today = day("2026-08-05");
        assert_eq!(
            short(day("2026-08-05"), today, Lang::from_code("en")),
            "today"
        );
        assert_eq!(
            short(day("2026-08-06"), today, Lang::from_code("es")),
            "mañana"
        );
        assert_eq!(
            short(day("2026-08-04"), today, Lang::from_code("en")),
            "yesterday"
        );
    }

    #[test]
    fn a_far_off_date_shows_the_day_and_month() {
        let out = short(day("2026-12-24"), day("2026-08-05"), Lang::from_code("en"));
        assert!(out.contains("24"), "{out}");
    }

    #[test]
    fn an_empty_list_says_so_instead_of_printing_nothing() {
        let out = list(
            &[],
            &State::default(),
            "today",
            day("2026-08-05"),
            Lang::from_code("en"),
        );
        assert!(out.contains(Lang::from_code("en").get("nothing-here")));
    }

    #[test]
    fn a_bare_task_renders_as_a_single_line() {
        let t = task("book a haircut");
        let out = list(
            &[&t],
            &State::default(),
            "today",
            day("2026-08-05"),
            Lang::from_code("en"),
        );
        let body: Vec<_> = out.lines().filter(|l| l.contains("haircut")).collect();

        assert_eq!(body.len(), 1);
        assert_eq!(out.lines().filter(|l| l.trim().starts_with('·')).count(), 0);
    }

    #[test]
    fn a_documented_task_shows_its_sections() {
        let mut t = task("fix the failing checkout");
        t.description = Some("reproduces only with an empty cart".into());
        t.tags = vec![Tag::new("work").unwrap()];

        let out = detail(
            &t,
            &State::default(),
            day("2026-08-05"),
            Lang::from_code("en"),
        );

        assert!(out.contains("description"));
        assert!(out.contains("reproduces only with an empty cart"));
        assert!(out.contains("@work"));
        assert!(!out.contains("steps"), "no steps means no section");
        assert!(!out.contains("journal"));
    }

    #[test]
    fn tasks_created_together_still_get_distinct_short_ids() {
        let mut a = task("first");
        let mut b = task("second");
        a.id = "01J8F2K3XQ0000000000000ABC".parse().unwrap();
        b.id = "01J8F2K3XQ0000000000000XYZ".parse().unwrap();
        assert_ne!(short_id(&a), short_id(&b));
    }

    #[test]
    fn priority_four_is_not_noise() {
        assert_eq!(priority(Priority::P4), None);
        assert!(priority(Priority::P1).is_some());
    }
}
