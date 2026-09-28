use std::process::ExitCode;

use tisty_core::Paths;

pub(crate) fn turn(
    paths: &Paths,
    on: Option<bool>,
    lang: crate::i18n::Lang,
) -> anyhow::Result<ExitCode> {
    let config = tisty_core::Config::load_or_init(paths)?;
    let named = |who: &tisty_core::DeviceId| tisty_core::config::nicknamed(&who.0);

    match (on, config.agent_id.clone()) {
        (None, Some(who)) => {
            println!(
                "  {}",
                lang.fill("agent-already", &[("name", &named(&who))])
            );
            println!("  {}", crate::style::dim(lang.get("agent-how")));
        }
        (None, None) => println!("  {}", crate::style::dim(lang.get("agent-none"))),
        (Some(true), Some(who)) => {
            println!(
                "  {}",
                lang.fill("agent-already", &[("name", &named(&who))])
            );
            println!("  {}", crate::style::dim(lang.get("agent-how")));
        }
        (Some(true), None) => {
            match let_in(crate::typist::at_the_persons_store(paths), at_a_terminal()) {
                Door::Asks if !agreed(lang) => {
                    println!("  {}", crate::style::dim(lang.get("agent-none")));
                    return Ok(ExitCode::SUCCESS);
                }
                Door::NoTerminal => {
                    eprintln!(
                        "{}: {}",
                        crate::style::paint(crate::style::RED, lang.get("error")),
                        lang.get("agent-needs-terminal")
                    );
                    return Ok(ExitCode::from(crate::EXIT_ERROR));
                }
                Door::Asks | Door::Open => {}
            }
            let who = tisty_core::agent::register(paths)?;
            println!("  {}", lang.fill("agent-on", &[("name", &named(&who))]));
            println!("  {}", crate::style::dim(lang.get("agent-how")));
        }
        (Some(false), Some(_)) => {
            tisty_core::agent::retire(paths)?;
            println!("  {}", lang.get("agent-off"));
        }
        (Some(false), None) => println!("  {}", crate::style::dim(lang.get("agent-not-on"))),
    }
    Ok(ExitCode::SUCCESS)
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Door {
    Open,
    Asks,
    NoTerminal,
}

/// Letting an assistant in is the person's act, so the person is asked — on the terminal
/// itself, which a shell an assistant drives does not have and a piped answer never reaches.
pub(super) fn let_in(at_the_persons_store: bool, at_a_terminal: bool) -> Door {
    match (at_the_persons_store, at_a_terminal) {
        (false, _) => Door::Open,
        (true, true) => Door::Asks,
        (true, false) => Door::NoTerminal,
    }
}

fn at_a_terminal() -> bool {
    use std::io::IsTerminal;
    std::io::stdin().is_terminal() && std::io::stderr().is_terminal()
}

fn agreed(lang: crate::i18n::Lang) -> bool {
    dialoguer::Confirm::new()
        .with_prompt(lang.get("agent-sure"))
        .default(false)
        .interact()
        .unwrap_or(false)
}
