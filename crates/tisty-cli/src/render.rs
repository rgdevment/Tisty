use jiff::{Zoned, civil::Date};
use tisty_core::{DateSpec, List, Priority, State, Status, Task};

use crate::style::{self, BLUE, GREEN, RED, YELLOW};

pub fn list(tasks: &[&Task], state: &State, heading: &str, today: Date) -> String {
    if tasks.is_empty() {
        return format!("\n  {}\n\n", style::dim("nada por aquí"));
    }

    let mut out = format!(
        "\n  {}{}\n\n",
        style::bold(heading),
        style::dim(&format!(
            "{:>width$}",
            plural(tasks.len(), "tarea", "tareas"),
            width = 62usize.saturating_sub(heading.len())
        ))
    );

    for (i, task) in tasks.iter().enumerate() {
        out.push_str(&row(i + 1, task, state, today));
    }
    out.push('\n');
    out
}

fn row(number: usize, task: &Task, state: &State, today: Date) -> String {
    let mut line = format!(
        "  {:>3}  {} {}",
        style::dim(&number.to_string()),
        marker(task),
        task.title
    );

    let mut meta = Vec::new();
    if let Some(p) = priority(task.priority) {
        meta.push(p);
    }
    if let Some(d) = &task.date {
        meta.push(when(d, today));
    }
    if let Some(d) = &task.deadline {
        meta.push(style::paint(
            RED,
            &format!("límite {}", short(d.date(), today)),
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
    if !task.log.is_empty() {
        meta.push(style::dim(&format!("✎{}", task.log.len())));
    }

    if !meta.is_empty() {
        line.push_str(&format!("\n       {}", meta.join(" · ")));
    }
    line.push('\n');
    line
}

pub fn detail(task: &Task, state: &State, today: Date) -> String {
    let mut out = format!(
        "\n  {} {}{}\n",
        marker(task),
        style::bold(&task.title),
        style::dim(&format!("  {}", short_id(task)))
    );

    let mut meta = Vec::new();
    if let Some(d) = &task.date {
        meta.push(when(d, today));
    }
    if let Some(d) = &task.deadline {
        meta.push(format!("límite {}", short(d.date(), today)));
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
        out.push_str(&section("descripción", None));
        for line in description.lines() {
            out.push_str(&format!("    {line}\n"));
        }
    }

    if !task.steps.is_empty() {
        let (done, total) = task.steps_done();
        out.push_str(&section("pasos", Some(&format!("{done}/{total}"))));
        for step in &task.steps {
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
            out.push_str(&format!("    {mark} {text}\n"));
        }
    }

    if !task.log.is_empty() {
        out.push_str(&section("bitácora", Some(&task.log.len().to_string())));
        for entry in &task.log {
            let stamp = entry
                .at
                .to_zoned(jiff::tz::TimeZone::system())
                .strftime("%a %-d %b · %H:%M")
                .to_string();
            out.push_str(&format!("    {}\n", style::dim(&stamp)));
            for line in entry.body.lines() {
                out.push_str(&format!("      {line}\n"));
            }
        }
    }

    out.push('\n');
    out
}

pub fn captured(task: &Task, state: &State, today: Date) -> String {
    let mut out = format!(
        "\n  {} {}\n",
        style::paint(GREEN, "✓"),
        style::bold(&task.title)
    );

    let mut meta = Vec::new();
    if let Some(d) = &task.date {
        meta.push(when(d, today));
    }
    if let Some(d) = &task.deadline {
        meta.push(format!("límite {}", short(d.date(), today)));
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

pub fn lists(state: &State) -> String {
    let mut active: Vec<&List> = state.active_lists().collect();
    if active.is_empty() {
        return format!("\n  {}\n\n", style::dim("ninguna lista todavía"));
    }
    active.sort_by(|a, b| a.order.cmp(&b.order));

    let mut out = format!("\n  {}\n\n", style::bold("listas"));
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

fn when(spec: &DateSpec, today: Date) -> String {
    let label = short(spec.date(), today);
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

fn short(date: Date, today: Date) -> String {
    match (date - today).get_days() {
        0 => "hoy".into(),
        1 => "mañana".into(),
        -1 => "ayer".into(),
        d if (0..7).contains(&d) => date.strftime("%a").to_string(),
        _ => date.strftime("%-d %b").to_string(),
    }
}

/// The tail, not the head: a ULID starts with its timestamp, so anything
/// created in the same millisecond shares a prefix.
fn short_id(task: &Task) -> String {
    let id = task.id.to_string();
    id[id.len() - 6..].to_lowercase()
}

fn slug(name: &str) -> String {
    name.to_lowercase().replace(' ', "-")
}

fn plural(n: usize, one: &str, many: &str) -> String {
    if n == 1 {
        format!("{n} {one}")
    } else {
        format!("{n} {many}")
    }
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
        assert_eq!(short(day("2026-08-05"), today), "hoy");
        assert_eq!(short(day("2026-08-06"), today), "mañana");
        assert_eq!(short(day("2026-08-04"), today), "ayer");
    }

    #[test]
    fn a_far_off_date_shows_the_day_and_month() {
        let out = short(day("2026-12-24"), day("2026-08-05"));
        assert!(out.contains("24"), "{out}");
    }

    #[test]
    fn an_empty_list_says_so_instead_of_printing_nothing() {
        let out = list(&[], &State::default(), "hoy", day("2026-08-05"));
        assert!(out.contains("nada por aquí"));
    }

    /// A trivial task must not drag a line of empty metadata behind it.
    #[test]
    fn a_bare_task_renders_as_a_single_line() {
        let t = task("agendar reunión con Pepe");
        let out = list(&[&t], &State::default(), "hoy", day("2026-08-05"));
        let body: Vec<_> = out.lines().filter(|l| l.contains("agendar")).collect();

        assert_eq!(body.len(), 1);
        assert_eq!(out.lines().filter(|l| l.trim().starts_with('·')).count(), 0);
    }

    #[test]
    fn a_documented_task_shows_its_sections() {
        let mut t = task("issue en redirecciones istio");
        t.description = Some("el query string se pierde".into());
        t.tags = vec![Tag::new("istio").unwrap()];

        let out = detail(&t, &State::default(), day("2026-08-05"));

        assert!(out.contains("descripción"));
        assert!(out.contains("el query string se pierde"));
        assert!(out.contains("@istio"));
        assert!(!out.contains("pasos"), "no steps means no section");
        assert!(!out.contains("bitácora"));
    }

    #[test]
    fn tasks_created_together_still_get_distinct_short_ids() {
        let a = task("first");
        let b = task("second");
        assert_ne!(short_id(&a), short_id(&b));
    }

    #[test]
    fn priority_four_is_not_noise() {
        assert_eq!(priority(Priority::P4), None);
        assert!(priority(Priority::P1).is_some());
    }
}
