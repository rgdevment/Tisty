use unicase::UniCase;

// The tokenizer stops expanding references past the length of its text, and never before this.
const BUDGET: usize = 100_000;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Defined {
    pub(crate) links: Vec<[String; 2]>,
    pub(crate) notes: Vec<String>,
    widest: usize,
    pub(crate) closers: usize,
}

impl Defined {
    pub(crate) fn of(mut links: Vec<[String; 2]>, widest: usize, text: &str) -> Self {
        links.sort_unstable();
        let closers = if links.is_empty() { 0 } else { closers(text) };
        Self {
            links,
            notes: Vec::new(),
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
        self.links
            .iter()
            .find(|[one, _]| UniCase::new(one.as_str()) == label)
            .map(|[_, to]| to.as_str())
    }

    pub(crate) fn after(&self, gone: &str, put: &str) -> usize {
        if self.links.is_empty() {
            0
        } else {
            self.closers + closers(put) - closers(gone)
        }
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
