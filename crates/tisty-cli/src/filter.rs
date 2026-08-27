use jiff::ToSpan;
use jiff::civil::Date;
use tisty_core::{
    Reading, Tag,
    view::{self, Window},
};

use crate::app::App;
use crate::i18n::{self, Lang};

#[derive(Debug, Default)]
pub struct Filter {
    pub inner: view::Filter,
    heading: String,
}

impl Filter {
    pub fn parse(tokens: &[String], app: &App, today: Date, lang: Lang) -> anyhow::Result<Self> {
        let mut f = Self {
            heading: heading(tokens, lang),
            ..Default::default()
        };
        if tokens.is_empty() {
            f.inner.window = Some(Window::Today);
            return Ok(f);
        }

        for token in tokens {
            f.take(token, app, today, lang)?;
        }
        Ok(f)
    }

    fn take(&mut self, token: &str, app: &App, today: Date, lang: Lang) -> anyhow::Result<()> {
        if let Some(name) = token.strip_prefix('@') {
            return self.take_list(name, app, lang);
        }
        if let Some(name) = token.strip_prefix('#') {
            self.inner.tags.push(Tag::new(name)?);
            return Ok(());
        }
        if let Some(raw) = token.strip_prefix('!') {
            self.inner.priority = Some(crate::cmd::task::named_priority(raw, lang)?);
            return Ok(());
        }

        match i18n::canonical_filter(token) {
            Some("all") => Ok(()),
            Some("inbox") => {
                self.inner.inbox = true;
                Ok(())
            }
            Some("story") => self.take_layer(Reading::Story, lang),
            Some("routine") => self.take_layer(Reading::Routine, lang),
            Some("trace") => self.take_layer(Reading::Trace, lang),
            Some("archive") => {
                self.inner.scope = view::Scope::Archived;
                Ok(())
            }
            Some("folded") => {
                self.inner.scope = view::Scope::Either;
                self.inner.hidden = true;
                Ok(())
            }
            Some("today") => self.take_window(Window::Today, lang),
            Some("tomorrow") => self.take_window(Window::On(today.tomorrow()?), lang),
            Some("week") => self.take_window(Window::Until(today.checked_add(7.days())?), lang),
            Some("overdue") => self.take_window(Window::Overdue, lang),
            _ => self.take_date(token, lang),
        }
    }

    fn take_layer(&mut self, how: Reading, lang: Lang) -> anyhow::Result<()> {
        if self.inner.reading.is_some_and(|held| held != how) {
            anyhow::bail!("{}", lang.get("one-layer-only"));
        }
        self.inner.scope = view::Scope::Archived;
        self.inner.reading = Some(how);
        Ok(())
    }

    fn take_list(&mut self, name: &str, app: &App, lang: Lang) -> anyhow::Result<()> {
        match app.state.find_list(name).as_slice() {
            [one] => {
                self.inner.lists.push(one.id);
                Ok(())
            }
            [] => anyhow::bail!("{}", lang.fill("no-such-list", &[("selector", name)])),
            _ => anyhow::bail!("{}", lang.fill("ambiguous-list", &[("selector", name)])),
        }
    }

    fn take_date(&mut self, token: &str, lang: Lang) -> anyhow::Result<()> {
        let now = jiff::Zoned::now();
        match tisty_nl::parse_date(token, &now, lang.code()) {
            Some(spec) => self.take_window(Window::On(spec.date()), lang),
            None => anyhow::bail!(
                "{}",
                lang.fill(
                    "unknown-filter",
                    &[("filter", token), ("known", i18n::FILTERS)]
                )
            ),
        }
    }

    fn take_window(&mut self, window: Window, lang: Lang) -> anyhow::Result<()> {
        if self.inner.window.is_some() {
            anyhow::bail!("{}", lang.get("one-window-only"));
        }
        self.inner.window = Some(window);
        Ok(())
    }

    pub fn heading(&self) -> &str {
        &self.heading
    }
}

fn heading(tokens: &[String], lang: Lang) -> String {
    match tokens {
        [] => lang.get("today").to_string(),
        [one] => match i18n::canonical_filter(one) {
            Some(name) => lang.get(name).to_string(),
            None => one.clone(),
        },
        many => many.join(" "),
    }
}
