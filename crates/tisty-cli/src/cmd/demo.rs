use std::process::ExitCode;

use jiff::civil::Date;
use tisty_core::event::{Body, DocAdd, ListAdd, LogAdd, Op, StepAdd, TaskAdd};
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
    let lists = shelves(&mut ops, lang);
    tasks(&mut ops, today, &lists, lang);
    ops.extend(papers(app, lang)?);
    let planted = ops.len();
    let written: Vec<String> = ops
        .iter()
        .filter_map(|op| match op {
            Op::DocAdd { d, .. } => Some(d.file.clone()),
            _ => None,
        })
        .collect();

    if let Err(gone) = app.commit_all(ops) {
        let root = app.paths.docs();
        for file in &written {
            let _ = tisty_core::docs::remove(&root, file);
        }
        return Err(gone.into());
    }

    println!(
        "  {}",
        lang.fill(
            "demo-planted",
            &[("n", &planted.to_string()), ("name", &under)]
        )
    );
    Ok(ExitCode::SUCCESS)
}

const ES: &[&str] = &[
    r#"# Minuta del lunes

Estuvimos Ana, Bruno y yo. Media hora, sin pantalla compartida.

## Lo que quedó decidido

- El cambio de tarifas entra el **1 de octubre**, no antes.
- Bruno prepara el correo a los clientes con dos semanas de aviso.
- Ana revisa los contratos que vencen en septiembre.

## Lo que quedó en el aire

| Tema | Quién | Cuándo |
| --- | --- | --- |
| Migrar el histórico | Sin dueño | Octubre |
| Sustituir la impresora | Ana | Cuando llegue el presupuesto |

> No se toca el sistema viejo hasta que la migración esté probada.

La fecha que no se mueve es ==el 1 de octubre==, y la que sigue en el aire es
<mark data-pen="blue">la impresora</mark>. Las tarifas se publican en
[la página de precios](https://example.org/precios) esa misma mañana.
"#,
    r#"# Pan de masa madre

La masa lleva viva desde marzo. Refrescarla la noche antes.

## Ingredientes

- 500 g de harina de fuerza
- 350 g de agua templada
- 100 g de masa madre activa
- 10 g de sal

## Cómo va

1. Mezclar harina y agua, y dejarlo reposar 40 minutos.
2. Añadir la masa madre y la sal.
3. Tres pliegues, uno cada media hora.
4. Levar en frío toda la noche.
5. Horno a 250 °C con vapor los primeros 20 minutos.

La última vez salió apretada por meterla al horno demasiado pronto.

---

🍞 **Hornear a las ocho, comer a las nueve**

Lo único que merece quedar escrito: <mark data-pen="green">fermentar en
frío</mark>, nunca sobre la encimera.
"#,
    r#"# Viaje a Lisboa

Del 14 al 18 de octubre. Vuelo por la mañana, vuelta el sábado tarde.

## Antes de salir

- [x] Reservar el hotel en Alfama
- [x] Avisar en el trabajo
- [ ] Renovar el pasaporte
- [ ] Cambiar dinero

## Gastos previstos

| Concepto | Estimado |
| --- | --- |
| Vuelos | 180 € |
| Hotel, cuatro noches | 320 € |
| Comidas | 200 € |

Miradouro da Senhora do Monte al atardecer, y la librería de la Rua Garrett.

<mark data-pen="pink">El pasaporte es lo que no puede esperar</mark>: seis
semanas si va por la vía lenta.
"#,
    r#"# El servidor de casa

Lo que hay que recordar cuando algo deja de responder.

## Qué corre ahí

- `nas` — copias de seguridad, arranca solo
- `media` — biblioteca, depende del NAS
- `dns` — bloqueo de anuncios

## Cuando el disco se llena

```
docker system prune -a
journalctl --vacuum-time=7d
```

Si el NAS no monta, revisar primero el cable: ya ha fallado dos veces.
"#,
];

const EN: &[&str] = &[
    r#"# Monday minutes

Ana, Bruno and me. Half an hour, no screen sharing.

## Settled

- The new rates start on **October 1st**, not before.
- Bruno writes to the customers with two weeks' notice.
- Ana goes through the contracts ending in September.

## Left open

| Matter | Who | When |
| --- | --- | --- |
| Move the old records | Nobody yet | October |
| Replace the printer | Ana | Once the quote arrives |

> The old system stays untouched until the move has been tested.

The date nobody may move is ==October 1st==, and the one still in the air is
<mark data-pen="blue">the printer</mark>. Rates are published on
[the pricing page](https://example.org/pricing) the same morning.
"#,
    r#"# Sourdough bread

The starter has been alive since March. Feed it the night before.

## What goes in

- 500 g strong flour
- 350 g warm water
- 100 g active starter
- 10 g salt

## How it goes

1. Mix flour and water, rest for 40 minutes.
2. Add the starter and the salt.
3. Three folds, one every half hour.
4. Prove cold overnight.
5. Oven at 250 °C, steam for the first 20 minutes.

Last time the crumb came out tight: it went in too early.

---

🍞 **Bake at eight, eat at nine**

The one thing worth writing down: <mark data-pen="green">prove it cold</mark>,
never on the counter.
"#,
    r#"# Lisbon trip

October 14th to 18th. Morning flight out, Saturday evening back.

## Before leaving

- [x] Book the hotel in Alfama
- [x] Tell work
- [ ] Renew the passport
- [ ] Get cash

## What it should cost

| Item | Estimate |
| --- | --- |
| Flights | 180 € |
| Hotel, four nights | 320 € |
| Food | 200 € |

Miradouro da Senhora do Monte at sunset, and the bookshop on Rua Garrett.

<mark data-pen="pink">The passport is the one that cannot wait</mark> — six
weeks if it goes the slow way.
"#,
    r#"# The home server

What to remember when something stops answering.

## What runs there

- `nas` — backups, starts on its own
- `media` — library, needs the NAS
- `dns` — ad blocking

## When the disk fills up

```
docker system prune -a
journalctl --vacuum-time=7d
```

If the NAS will not mount, check the cable first: it has failed twice already.
"#,
];

fn papers(app: &App, lang: Lang) -> anyhow::Result<Vec<Op>> {
    let root = app.paths.docs();
    let device = app.device().clone();
    let sheets = if lang.code().starts_with("es") {
        ES
    } else {
        EN
    };

    let made = sheets
        .iter()
        .map(|body| tisty_core::docs::create(&root, &device, body))
        .collect::<Result<Vec<_>, _>>()?;

    if let (Some(first), Some(last)) = (made.first(), made.last()) {
        let card = Said::of(
            "\n\nLo que hay que mirar cuando algo deja de responder:\n\n",
            "\n\nWhat to look at when something stops answering:\n\n",
        );
        let body = format!(
            "{}{}![{}](tisty:doc/{})\n",
            sheets[0],
            card.pick(lang),
            last.title,
            last.id
        );
        tisty_core::docs::write(&root, &first.id, &body)?;
    }

    let shelf = ulid::Ulid::generate();
    let called = Said::of("Referencia", "Reference");
    let mut ops = vec![Op::FolderAdd {
        id: shelf,
        d: tisty_core::event::FolderAdd {
            name: called.pick(lang).to_string(),
            order: "a0".into(),
            parent: None,
            icon: None,
            color: None,
        },
    }];

    ops.extend(made.into_iter().enumerate().map(|(n, one)| Op::DocAdd {
        id: ulid::Ulid::generate(),
        d: DocAdd {
            page_of: None,
            file: one.id,
            order: format!("a{n}"),
            folder: (n >= 2).then_some(shelf),
        },
    }));

    Ok(ops)
}

fn shelves(ops: &mut Vec<Op>, lang: Lang) -> Vec<ListId> {
    [
        Said::of("Casa", "Home"),
        Said::of("Trabajo", "Work"),
        Said::of("Salud", "Health"),
        Said::of("Finanzas", "Money"),
    ]
    .into_iter()
    .enumerate()
    .map(|(n, name)| {
        let id = ulid::Ulid::generate();
        ops.push(Op::ListAdd {
            id,
            d: ListAdd {
                name: name.pick(lang).to_string(),
                color: None,
                order: format!("a{n}"),
            },
        });
        id
    })
    .collect()
}

#[derive(Clone, Copy)]
struct Said {
    es: &'static str,
    en: &'static str,
}

impl Said {
    const fn of(es: &'static str, en: &'static str) -> Self {
        Self { es, en }
    }

    fn pick(self, lang: Lang) -> &'static str {
        if lang.code().starts_with("es") {
            self.es
        } else {
            self.en
        }
    }
}

struct Seed {
    title: Said,
    away: Option<i8>,
    at: Option<(i8, i8)>,
    priority: Priority,
    tags: &'static [Said],
    list: Option<usize>,
    every: Option<(u16, Unit)>,
    deadline: Option<i8>,
}

impl Seed {
    const fn new(title: Said) -> Self {
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

fn tasks(ops: &mut Vec<Op>, today: Date, lists: &[ListId], lang: Lang) {
    let mut order = 0;
    let mut next = || {
        order += 1;
        format!("a{order:03}")
    };

    for (n, seed) in bed().into_iter().enumerate() {
        let id = ulid::Ulid::generate();
        let mut add = TaskAdd::new(seed.title.pick(lang), next());
        add.priority = Some(seed.priority);
        add.tags = seed
            .tags
            .iter()
            .filter_map(|t| Tag::new(t.pick(lang)).ok())
            .collect();
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
    const HOME: Said = Said::of("casa", "home");
    const FAMILY: Said = Said::of("familia", "family");
    const MONEY: Said = Said::of("finanzas", "money");
    const BOOKS: Said = Said::of("libros", "books");
    const CAR: Said = Said::of("coche", "car");
    const HEALTH: Said = Said::of("salud", "health");
    const SHOPPING: Said = Said::of("compras", "shopping");
    const WORK: Said = Said::of("trabajo", "work");
    const TRAVEL: Said = Said::of("viaje", "travel");
    const GARDEN: Said = Said::of("jardin", "garden");
    const COOKING: Said = Said::of("cocina", "cooking");
    const STUDY: Said = Said::of("estudio", "study");
    const GIFTS: Said = Said::of("regalos", "gifts");
    const SPORT: Said = Said::of("deporte", "sport");
    const MUSIC: Said = Said::of("musica", "music");

    vec![
        Seed {
            away: Some(17),
            priority: Priority::Do,
            tags: &[HOME, FAMILY],
            list: Some(0),
            ..Seed::new(Said::of("preparar la mudanza", "get ready for the move"))
        },
        Seed {
            away: Some(-10),
            priority: Priority::Do,
            tags: &[MONEY],
            list: Some(3),
            ..Seed::new(Said::of("pagar la luz", "pay the electricity bill"))
        },
        Seed {
            away: Some(-7),
            priority: Priority::Delegate,
            tags: &[BOOKS],
            ..Seed::new(Said::of(
                "devolver el libro a la biblioteca",
                "take the book back to the library",
            ))
        },
        Seed {
            away: Some(-3),
            priority: Priority::Decide,
            tags: &[CAR],
            ..Seed::new(Said::of(
                "llamar al seguro del coche",
                "call the car insurance",
            ))
        },
        Seed {
            away: Some(-1),
            priority: Priority::Do,
            tags: &[HEALTH],
            list: Some(2),
            ..Seed::new(Said::of("recoger la receta", "pick up the prescription"))
        },
        Seed {
            away: Some(0),
            tags: &[SHOPPING],
            ..Seed::new(Said::of("comprar pan", "buy bread"))
        },
        Seed {
            away: Some(0),
            at: Some((15, 0)),
            priority: Priority::Delegate,
            tags: &[WORK],
            list: Some(1),
            ..Seed::new(Said::of("reunión de equipo", "team meeting"))
        },
        Seed {
            away: Some(0),
            at: Some((21, 0)),
            tags: &[HOME],
            every: Some((1, Unit::Day)),
            ..Seed::new(Said::of("sacar la basura", "take the bins out"))
        },
        Seed {
            away: Some(1),
            at: Some((11, 0)),
            priority: Priority::Delegate,
            tags: &[SHOPPING],
            ..Seed::new(Said::of("recoger el paquete", "pick up the parcel"))
        },
        Seed {
            away: Some(1),
            priority: Priority::Delegate,
            tags: &[WORK],
            list: Some(1),
            deadline: Some(4),
            ..Seed::new(Said::of(
                "preparar la presentación",
                "put the talk together",
            ))
        },
        Seed {
            away: Some(3),
            at: Some((10, 0)),
            priority: Priority::Decide,
            tags: &[MONEY],
            list: Some(3),
            ..Seed::new(Said::of(
                "cita con el gestor",
                "meeting with the accountant",
            ))
        },
        Seed {
            away: Some(4),
            tags: &[FAMILY],
            every: Some((1, Unit::Year)),
            ..Seed::new(Said::of("cumpleaños de Lucía", "Lucia's birthday"))
        },
        Seed {
            away: Some(7),
            priority: Priority::Minor,
            tags: &[HOME, GARDEN],
            every: Some((1, Unit::Week)),
            ..Seed::new(Said::of("regar las plantas", "water the plants"))
        },
        Seed {
            away: Some(9),
            priority: Priority::Do,
            tags: &[WORK],
            list: Some(1),
            deadline: Some(9),
            ..Seed::new(Said::of(
                "entregar el informe trimestral",
                "hand in the quarterly report",
            ))
        },
        Seed {
            away: Some(14),
            at: Some((7, 40)),
            priority: Priority::Do,
            tags: &[TRAVEL],
            ..Seed::new(Said::of("vuelo a Madrid", "flight to Madrid"))
        },
        Seed {
            away: Some(20),
            tags: &[MONEY],
            every: Some((1, Unit::Month)),
            ..Seed::new(Said::of("pagar el alquiler", "pay the rent"))
        },
        Seed {
            priority: Priority::Minor,
            tags: &[COOKING],
            ..Seed::new(Said::of(
                "aprender a hacer pan de masa madre",
                "learn to bake sourdough",
            ))
        },
        Seed {
            priority: Priority::Decide,
            tags: &[BOOKS, STUDY],
            ..Seed::new(Said::of(
                "leer el libro de arquitectura de software",
                "read the software architecture book",
            ))
        },
        Seed {
            priority: Priority::Decide,
            tags: &[HOME],
            list: Some(0),
            ..Seed::new(Said::of(
                "montar la estantería del pasillo",
                "put up the hallway shelf",
            ))
        },
        Seed {
            priority: Priority::Minor,
            tags: &[GIFTS],
            ..Seed::new(Said::of(
                "elegir el regalo de aniversario",
                "choose the anniversary present",
            ))
        },
        Seed {
            priority: Priority::Decide,
            tags: &[SPORT, TRAVEL],
            ..Seed::new(Said::of(
                "planificar la ruta de senderismo",
                "plan the hiking route",
            ))
        },
        Seed {
            tags: &[MUSIC],
            ..Seed::new(Said::of(
                "buscar un profesor de guitarra",
                "find a guitar teacher",
            ))
        },
    ]
}
