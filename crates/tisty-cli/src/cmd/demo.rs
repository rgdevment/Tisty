use std::process::ExitCode;

use jiff::civil::Date;
use tisty_core::event::{Body, ListAdd, LogAdd, Op, StepAdd, TaskAdd};
use tisty_core::model::{Cadence, DateSpec, Repeat, Tag, Unit};
use tisty_core::{ListId, Priority, TaskId};

use crate::app::App;
use crate::i18n::Lang;

pub fn demo(app: &mut App, force: bool, lang: Lang) -> anyhow::Result<ExitCode> {
    let Some(under) = tisty_core::paths::profile() else {
        anyhow::bail!("{}", lang.get("demo-needs-sandbox"));
    };
    if !app.state.tasks.is_empty() && !force {
        anyhow::bail!(
            "{}",
            lang.fill(
                "demo-not-empty",
                &[("n", &app.state.tasks.len().to_string())]
            )
        );
    }

    let today = jiff::Zoned::now().date();
    let mut ops = Vec::new();
    let lists = shelves(&mut ops);
    tasks(&mut ops, today, &lists);
    let planted = ops.len();
    app.commit_all(ops)?;

    println!(
        "  {}",
        lang.fill(
            "demo-planted",
            &[("n", &planted.to_string()), ("name", &under)]
        )
    );
    Ok(ExitCode::SUCCESS)
}

fn shelves(ops: &mut Vec<Op>) -> Vec<ListId> {
    ["Casa", "Trabajo", "Salud", "Finanzas"]
        .into_iter()
        .enumerate()
        .map(|(n, name)| {
            let id = ulid::Ulid::generate();
            ops.push(Op::ListAdd {
                id,
                d: ListAdd {
                    name: name.to_string(),
                    color: None,
                    order: format!("a{n}"),
                },
            });
            id
        })
        .collect()
}

struct Seed {
    title: &'static str,
    away: Option<i8>,
    at: Option<(i8, i8)>,
    priority: Priority,
    tags: &'static [&'static str],
    list: Option<usize>,
    every: Option<(u16, Unit)>,
    deadline: Option<i8>,
}

impl Seed {
    const fn new(title: &'static str) -> Self {
        Self {
            title,
            away: None,
            at: None,
            priority: Priority::Unset,
            tags: &[],
            list: None,
            every: None,
            deadline: None,
        }
    }
}

const ZONE: &str = "UTC";

fn tasks(ops: &mut Vec<Op>, today: Date, lists: &[ListId]) {
    let mut order = 0;
    let mut next = || {
        order += 1;
        format!("a{order:03}")
    };

    for (n, seed) in bed().into_iter().enumerate() {
        let id = ulid::Ulid::generate();
        let mut add = TaskAdd::new(seed.title, next());
        add.priority = Some(seed.priority);
        add.tags = seed.tags.iter().filter_map(|t| Tag::new(t).ok()).collect();
        add.list = seed.list.and_then(|at| lists.get(at).copied());
        add.date = seed.away.map(|away| when(today, away, seed.at));
        add.deadline = seed
            .deadline
            .map(|away| DateSpec::all_day(shifted(today, away), ZONE));
        add.repeat = seed
            .every
            .map(|(every, unit)| Repeat::due(Cadence { every, unit }));
        ops.push(Op::TaskAdd { id, d: add });

        if n == 0 {
            fleshed(ops, id);
        }
    }
}

fn when(today: Date, away: i8, at: Option<(i8, i8)>) -> DateSpec {
    let day = shifted(today, away);
    match at {
        Some((hour, minute)) => DateSpec::floating(day.at(hour, minute, 0, 0), ZONE),
        None => DateSpec::all_day(day, ZONE),
    }
}

fn shifted(today: Date, away: i8) -> Date {
    today
        .checked_add(jiff::Span::new().days(i64::from(away)))
        .unwrap_or(today)
}

fn fleshed(ops: &mut Vec<Op>, id: TaskId) {
    ops.push(Op::TaskDescribe {
        id,
        d: Body {
            body: Some(
                "Presupuesto aceptado. Falta confirmar el montacargas con 48 h.".to_string(),
            ),
        },
    });
    for (n, text) in ["pedir cajas", "avisar al portero", "contratar el furgón"]
        .into_iter()
        .enumerate()
    {
        let step = ulid::Ulid::generate();
        ops.push(Op::StepAdd {
            id,
            d: StepAdd {
                step,
                text: text.to_string(),
                order: format!("a{n}"),
            },
        });
        if n == 0 {
            ops.push(Op::StepDone {
                id,
                d: tisty_core::event::StepRef { step },
            });
        }
    }
    for note in [
        "Mudanzas Ríos: 640 € con embalaje.",
        "El montacargas se reserva con 48 h de antelación.",
    ] {
        ops.push(Op::TaskLog {
            id,
            d: LogAdd::new(ulid::Ulid::generate(), note).in_zone(Some(ZONE.to_string())),
        });
    }
}

fn bed() -> Vec<Seed> {
    vec![
        Seed {
            away: Some(17),
            priority: Priority::Do,
            tags: &["casa", "familia"],
            list: Some(0),
            ..Seed::new("preparar la mudanza")
        },
        Seed {
            away: Some(-10),
            priority: Priority::Do,
            tags: &["finanzas"],
            list: Some(3),
            ..Seed::new("pagar la luz")
        },
        Seed {
            away: Some(-7),
            tags: &["libros"],
            ..Seed::new("devolver el libro a la biblioteca")
        },
        Seed {
            away: Some(-3),
            priority: Priority::Decide,
            tags: &["coche"],
            ..Seed::new("llamar al seguro del coche")
        },
        Seed {
            away: Some(-1),
            priority: Priority::Do,
            tags: &["salud"],
            list: Some(2),
            ..Seed::new("recoger la receta")
        },
        Seed {
            away: Some(0),
            tags: &["compras"],
            ..Seed::new("comprar pan")
        },
        Seed {
            away: Some(0),
            at: Some((15, 0)),
            priority: Priority::Delegate,
            tags: &["trabajo"],
            list: Some(1),
            ..Seed::new("reunión de equipo")
        },
        Seed {
            away: Some(0),
            at: Some((21, 0)),
            tags: &["casa"],
            every: Some((1, Unit::Day)),
            ..Seed::new("sacar la basura")
        },
        Seed {
            away: Some(1),
            at: Some((11, 0)),
            tags: &["compras"],
            ..Seed::new("recoger el paquete")
        },
        Seed {
            away: Some(1),
            priority: Priority::Delegate,
            tags: &["trabajo"],
            list: Some(1),
            deadline: Some(4),
            ..Seed::new("preparar la presentación")
        },
        Seed {
            away: Some(3),
            at: Some((10, 0)),
            priority: Priority::Decide,
            tags: &["finanzas"],
            list: Some(3),
            ..Seed::new("cita con el gestor")
        },
        Seed {
            away: Some(4),
            tags: &["familia"],
            every: Some((1, Unit::Year)),
            ..Seed::new("cumpleaños de Lucía")
        },
        Seed {
            away: Some(7),
            tags: &["casa", "jardin"],
            every: Some((1, Unit::Week)),
            ..Seed::new("regar las plantas")
        },
        Seed {
            away: Some(9),
            priority: Priority::Do,
            tags: &["trabajo"],
            list: Some(1),
            deadline: Some(9),
            ..Seed::new("entregar el informe trimestral")
        },
        Seed {
            away: Some(14),
            at: Some((7, 40)),
            priority: Priority::Do,
            tags: &["viaje"],
            ..Seed::new("vuelo a Madrid")
        },
        Seed {
            away: Some(20),
            tags: &["finanzas"],
            every: Some((1, Unit::Month)),
            ..Seed::new("pagar el alquiler")
        },
        Seed {
            tags: &["cocina"],
            ..Seed::new("aprender a hacer pan de masa madre")
        },
        Seed {
            tags: &["libros", "estudio"],
            ..Seed::new("leer el libro de arquitectura de software")
        },
        Seed {
            tags: &["casa"],
            list: Some(0),
            ..Seed::new("montar la estantería del pasillo")
        },
        Seed {
            priority: Priority::Wont,
            tags: &["regalos"],
            ..Seed::new("elegir el regalo de aniversario")
        },
        Seed {
            tags: &["deporte", "viaje"],
            ..Seed::new("planificar la ruta de senderismo")
        },
        Seed {
            tags: &["musica"],
            ..Seed::new("buscar un profesor de guitarra")
        },
    ]
}
