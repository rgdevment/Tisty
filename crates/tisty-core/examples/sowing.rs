use tisty_core::event::{DeviceId, TaskAdd};
use tisty_core::{Op, Store};
use ulid::Ulid;

const MANY: usize = 100_000;
const AT_A_TIME: usize = 5_000;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let paths = tisty_core::paths::Paths::resolve()?;
    let store = paths.store();
    println!("sowing into {}", store.display());

    let mut held = Store::open(&store, DeviceId("sown".into()))?;
    let mut alive: Vec<Ulid> = Vec::with_capacity(MANY);
    let started = std::time::Instant::now();

    for turn in 0..(MANY / AT_A_TIME) {
        let mut ops = Vec::with_capacity(AT_A_TIME);
        for n in 0..AT_A_TIME {
            let id = Ulid::generate();
            alive.push(id);
            ops.push(Op::TaskAdd {
                id,
                d: TaskAdd::new(
                    format!("una de las muchas que hubo, la {} de todas", turn * AT_A_TIME + n),
                    "a0",
                ),
            });
        }
        held.append_batch(ops)?;
        println!("  puestas {}", (turn + 1) * AT_A_TIME);
    }

    let leave = 5_000;
    for (turn, gone) in alive[leave..].chunks(AT_A_TIME).enumerate() {
        let ops: Vec<Op> = gone.iter().map(|id| Op::TaskDelete { id: *id }).collect();
        held.append_batch(ops)?;
        println!("  quitadas {}", (turn + 1) * AT_A_TIME);
    }

    let told = tisty_core::store::read_all(&store)?;
    let state = tisty_core::State::replay(&told);
    println!(
        "{} eventos en el registro, {} tareas vivas, en {:?}",
        told.len(),
        state.tasks.len(),
        started.elapsed()
    );
    Ok(())
}
