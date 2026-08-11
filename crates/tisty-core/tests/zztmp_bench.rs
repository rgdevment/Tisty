use std::time::Instant;
use tisty_core::{DeviceId, Event, Op, State, event::TaskAdd};
use ulid::Ulid;

fn at(ms: i64) -> jiff::Timestamp {
    jiff::Timestamp::from_millisecond(ms).unwrap()
}

#[test]
fn zztmp_where_the_time_goes() {
    let n: i64 = 20_000;
    let mut state = State::default();
    let dev = DeviceId("a".into());
    for i in 0..n {
        let id = Ulid::generate();
        state.apply(&Event::new(
            dev.clone(),
            at(i),
            Op::TaskAdd {
                id,
                d: TaskAdd::new(
                    format!("Revisar el informe trimestral de la sucursal {i} antes del cierre"),
                    format!("a{i}"),
                ),
            },
        ));
        state.apply(&Event::new(
            dev.clone(),
            at(i),
            Op::TaskDescribe {
                id,
                d: tisty_core::event::Body {
                    body: Some(
                        "Un cuerpo razonablemente largo, del tamano que la gente escribe cuando \
                         de verdad usa la aplicacion todos los dias durante anos y anos. \
                         Repetido para llegar a unos cuantos cientos de bytes por tarea. "
                            .repeat(3),
                    ),
                },
            },
        ));
    }

    for (label, query) in [("one letter 'a'", "a"), ("two letters 'in'", "in")] {
        for most in [usize::MAX, 200] {
            let t = Instant::now();
            let (hits, total) = state.searching(query, tisty_core::view::Scope::Either, most);
            let scan = t.elapsed();
            let t2 = Instant::now();
            let cloned: Vec<_> = hits.into_iter().cloned().collect();
            let clone = t2.elapsed();
            let t3 = Instant::now();
            let json = serde_json::to_string(&cloned).unwrap();
            let ser = t3.elapsed();
            println!(
                "{label:18} most={:>10} total={total:>6} kept={:>6} scan+sort={scan:>12?} clone={clone:>12?} serialize={ser:>12?} json={} MiB",
                if most == usize::MAX { "ALL".into() } else { most.to_string() },
                cloned.len(),
                json.len() / 1024 / 1024,
            );
        }
    }
}
