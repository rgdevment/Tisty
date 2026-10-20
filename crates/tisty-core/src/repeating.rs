use crate::event::Op;
use crate::model::{Priority, Task, TaskId};
use crate::{State, order};

impl State {
    /// A whole's parts still open are let go in the same transaction, so one undo takes it back.
    pub fn completing(&self, id: TaskId, now: jiff::Zoned) -> Vec<Op> {
        let mut ops = self.open_parts_dropped(id);
        ops.extend(self.said_ticked(id));
        ops.extend(self.finishing(id, now));
        ops
    }

    fn finishing(&self, id: TaskId, now: jiff::Zoned) -> Vec<Op> {
        let done = vec![Op::TaskDone { id, filled: false }];
        let Some(task) = self.tasks.get(&id) else {
            return done;
        };
        if task.status != crate::model::Status::Open {
            return done;
        }
        let Some(repeat) = task.repeat else {
            return done;
        };
        let Some(next) = repeat.next(
            task.date.as_ref(),
            now.datetime(),
            now.datetime()
                .date()
                .to_datetime(jiff::civil::Time::midnight()),
            now.time_zone().iana_name().unwrap_or("UTC"),
        ) else {
            return done;
        };
        if repeat.ended(next.at.date()) {
            return done;
        }

        let mut ops = done;
        let order = self.order_last_in(task.list);
        ops.extend(self.turn_after(task, id, next, repeat, false, &order));
        ops
    }

    /// The turn that follows one being closed: same shape, dates carried along by how far it moved.
    /// A bare one is a record of a day that passed, so it carries no work to do and nothing to ring.
    fn turn_after(
        &self,
        task: &Task,
        after: TaskId,
        next: crate::model::DateSpec,
        repeat: crate::model::Repeat,
        bare: bool,
        order: &str,
    ) -> Vec<Op> {
        let along = task
            .date
            .as_ref()
            .map(|was| next.at.date().since(was.at.date()));

        let mut fresh = crate::event::TaskAdd::new(task.title.clone(), order.to_string());
        if !bare {
            fresh.deadline = shifted(task.deadline.as_ref(), along.as_ref());
            fresh.reminders = task
                .reminders
                .iter()
                .filter_map(|one| match (&task.date, repeat.cadence().after(one.at)) {
                    (None, Some(at)) => Some(one.moved(at)),
                    _ => shifted(Some(one), along.as_ref()),
                })
                .collect();
        }
        fresh.date = Some(next);
        fresh.priority = Some(match task.priority {
            Priority::Minor => Priority::Unset,
            kept => kept,
        });
        fresh.list = task.list;
        fresh.tags = task.tags.clone();
        fresh.repeat = Some(repeat);
        fresh.after = Some(after);

        let born = ulid::Ulid::generate();
        let mut ops = vec![Op::TaskAdd { id: born, d: fresh }];
        if bare {
            return ops;
        }

        if let Some(body) = &task.description {
            ops.push(Op::TaskDescribe {
                id: born,
                d: crate::event::Body {
                    body: Some(body.clone()),
                },
            });
        }
        for step in &task.steps {
            ops.push(Op::StepAdd {
                id: born,
                d: crate::event::StepAdd {
                    step: ulid::Ulid::generate(),
                    text: step.text.clone(),
                    order: step.order.clone(),
                },
            });
        }
        ops
    }

    /// The dates the cadence would skip between this turn and today, capped at what a person recalls.
    pub fn owed_since(&self, id: TaskId, today: jiff::civil::Date) -> Vec<jiff::civil::Date> {
        const RECALLS: usize = 5;
        /// Five turns of a weekly cadence reach back five weeks, which is no longer memory.
        const REACHES: i32 = 30;
        let Some(task) = self.tasks.get(&id) else {
            return Vec::new();
        };
        let (Some(due), Some(repeat)) = (task.date.as_ref(), task.repeat) else {
            return Vec::new();
        };
        if repeat.cadence().every == 0 {
            return Vec::new();
        }

        let mut at = due.at;
        let mut all = Vec::new();
        while all.len() <= RECALLS {
            let Some(next) = repeat.cadence().after(at) else {
                break;
            };
            if next.date() > today || repeat.ended(next.date()) {
                break;
            }
            all.push(next.date());
            at = next;
        }
        let dark = today
            .since(due.at.date())
            .is_ok_and(|gone| gone.get_days() > REACHES);
        if all.len() > RECALLS || dark {
            Vec::new()
        } else {
            all
        }
    }

    /// Fills the dates the person says they did, as turns already closed, so no gap is invented.
    pub fn covering(&self, id: TaskId, now: jiff::Zoned, also: &[jiff::civil::Date]) -> Vec<Op> {
        let Some(task) = self.tasks.get(&id) else {
            return vec![Op::TaskDone { id, filled: false }];
        };
        let (Some(due), Some(repeat)) = (task.date.clone(), task.repeat) else {
            return self.completing(id, now);
        };

        // A claimed date is only honoured if it was offered: anything else would write a turn the
        // cadence never had, and a mistyped year would drag the whole series into next January.
        let offered = self.owed_since(id, now.date());
        let mut wanted: Vec<jiff::civil::Date> = also
            .iter()
            .filter(|day| offered.contains(day))
            .copied()
            .collect();
        wanted.sort_unstable();
        wanted.dedup();

        let mut ops = self.said_ticked(id);
        ops.push(Op::TaskDone { id, filled: false });
        let mut before = id;
        let mut last = due.clone();
        let mut order = self.order_last_in(task.list);
        for day in wanted {
            if day <= last.date() {
                continue;
            }
            let filled = due.moved(day.to_datetime(due.at.time()));
            let mut born = self.turn_after(task, before, filled.clone(), repeat, true, &order);
            order = order::after(&order);
            let Some(Op::TaskAdd { id: fresh, .. }) = born.first() else {
                break;
            };
            let fresh = *fresh;
            ops.append(&mut born);
            ops.push(Op::TaskDone {
                id: fresh,
                filled: true,
            });
            last = filled;
            before = fresh;
        }

        if before == id {
            return self.completing(id, now);
        }

        let Some(next) = repeat.next(
            Some(&last),
            now.datetime(),
            now.datetime()
                .date()
                .to_datetime(jiff::civil::Time::midnight()),
            now.time_zone().iana_name().unwrap_or("UTC"),
        ) else {
            return ops;
        };
        if repeat.ended(next.at.date()) {
            return ops;
        }

        ops.extend(self.turn_after(task, before, next, repeat, false, &order));
        ops
    }
}

fn shifted(
    spec: Option<&crate::DateSpec>,
    along: Option<&Result<jiff::Span, jiff::Error>>,
) -> Option<crate::DateSpec> {
    let spec = spec?;
    let Some(Ok(span)) = along else {
        return Some(spec.clone());
    };
    match spec.at.checked_add(*span) {
        Ok(at) => Some(spec.moved(at)),
        Err(_) => Some(spec.clone()),
    }
}
