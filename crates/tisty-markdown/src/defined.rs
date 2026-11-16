use unicase::UniCase;

// The tokenizer stops expanding references past the length of its text, and never before this.
const BUDGET: usize = 100_000;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Defined {
    links: Vec<[String; 2]>,
    notes: Vec<String>,
    widest: usize,
    closers: usize,
}

impl Defined {
    pub(crate) fn of(
        mut links: Vec<[String; 2]>,
        widest: usize,
        notes: Vec<String>,
        text: &str,
    ) -> Self {
        links.sort_unstable_by(|[one, _], [other, _]| UniCase::new(one).cmp(&UniCase::new(other)));
        let closers = if links.is_empty() { 0 } else { closers(text) };
        Self {
            links,
            notes,
            widest,
            closers,
        }
    }

    pub(crate) fn labels(&self, far: &[String]) -> [u32; 2] {
        let links = self.links.iter().flatten();
        crate::names_sum(links.chain(&self.notes).chain(far).map(String::as_str))
    }

    pub(crate) fn to(&self, label: &str) -> Option<&str> {
        let label = UniCase::new(label);
        let at = self
            .links
            .binary_search_by(|[one, _]| UniCase::new(one.as_str()).cmp(&label))
            .ok()?;
        Some(self.links[at][1].as_str())
    }

    pub(crate) fn after(&self, gone: &str, put: &str) -> usize {
        if self.links.is_empty() {
            0
        } else {
            self.closers + closers(put) - closers(gone)
        }
    }

    pub(crate) fn recount(&mut self, closers: usize) {
        self.closers = closers;
    }

    // The blocks kept come from the old text, so it must have been read within the budget too.
    pub(crate) fn affords(&self, old: &str, text: &str, closers: usize, stretch: &str) -> bool {
        let fits = |closers: usize, read: &str| {
            self.widest.saturating_mul(closers) < read.len().max(BUDGET)
        };
        self.links.is_empty()
            || (fits(self.closers, old)
                && fits(closers, text)
                && fits(self::closers(stretch), stretch))
    }
}

fn closers(text: &str) -> usize {
    text.matches(']').count()
}
