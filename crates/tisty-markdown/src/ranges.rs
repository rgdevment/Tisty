use std::ops::Range;

/// Ranges sorted once, so each piece of a long paragraph asks only those around it instead of all of them.
pub(crate) struct Nearby {
    ranges: Vec<Range<usize>>,
    reach: Vec<usize>,
}

impl Nearby {
    pub(crate) fn of(mut ranges: Vec<Range<usize>>) -> Self {
        ranges.sort_by_key(|one| one.start);
        let reach = ranges
            .iter()
            .scan(0, |far, one| {
                *far = (*far).max(one.end);
                Some(*far)
            })
            .collect();
        Self { ranges, reach }
    }

    pub(crate) fn ranges(&self) -> &[Range<usize>] {
        &self.ranges
    }

    // Sorted by start and cut by how far each run reaches, so a long range earlier on is never missed.
    pub(crate) fn near(&self, within: &Range<usize>) -> &[Range<usize>] {
        let from = self.reach.partition_point(|far| *far <= within.start);
        let to = self.ranges.partition_point(|one| one.start < within.end);
        &self.ranges[from..to.max(from)]
    }

    pub(crate) fn touching(&self, within: &Range<usize>) -> impl Iterator<Item = &Range<usize>> {
        self.near(within)
            .iter()
            .filter(move |one| one.start < within.end && within.start < one.end)
    }

    pub(crate) fn holding(&self, within: &Range<usize>) -> bool {
        self.touching(within)
            .any(|one| one.start <= within.start && within.end <= one.end)
    }

    pub(crate) fn holds(&self, at: usize) -> bool {
        self.holding(&(at..at + 1))
    }
}

// One sweep over `silent`, sorted by start: splitting the pieces again for every range made a long text quadratic.
pub(crate) fn outside(within: Range<usize>, silent: &[Range<usize>]) -> Vec<Range<usize>> {
    let mut pieces = Vec::new();
    let mut from = within.start;
    for one in silent {
        if one.start >= within.end {
            break;
        }
        if one.start > from {
            pieces.push(from..one.start);
        }
        from = from.max(one.end);
    }
    if from < within.end {
        pieces.push(from..within.end);
    }
    pieces
}

#[cfg(test)]
#[path = "ranges_test.rs"]
mod tests;
