use tisty_core::agent::{a_key_itself, secret_in};

fn told(text: &str) -> Option<String> {
    secret_in(text.as_bytes()).map(|one| one.said())
}

#[test]
fn documenting_environment_variables_is_not_holding_a_secret() {
    for plain in [
        r#"FOO_URL="https://ejemplo.cl/x"
FOO_VALOR="«REDACTADO»""#,
        r#"FOO_CLIENT_KEY="REDACTADO"
FOO_TOKEN="REDACTADO"
FOO_SECRET="REDACTADO""#,
        r#"export FOO_CLIENT_KEY="$FOO_CLIENT_KEY"
export FOO_TOKEN="${FOO_TOKEN}""#,
        r#"npm install --global corepack
docker compose up -d"#,
        r#"# FOO_KEY: pendiente
# FOO_SECRET: pendiente"#,
        r#"| Variable | Valor |
| FOO_KEY | pendiente |
| FOO_SECRET | pendiente |"#,
        r#"El CLIENT_KEY se pide en el portal y el CLIENT_SECRET llega por correo."#,
        r#"MAX_TOKENS=1048576
TIMEOUT_KEY=30000"#,
        r#"DATABASE_URL=postgres://localhost:5432/tisty
REDIS_URL=redis://localhost:6379/0"#,
    ] {
        assert_eq!(told(plain), None, "lo tomo por secreto: {plain}");
    }
}

#[test]
fn a_credential_with_a_value_that_is_really_one_is_still_caught() {
    for plain in [
        r#"AWS_SECRET_ACCESS_KEY=wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY
AWS_REGION=us-east-1"#,
        r#"DB_PASSWORD=Tr0ub4dor3xK
SMTP_PASSWORD=h4nsel4ndGretel"#,
        r#"DATABASE_URL=postgres://tisty:s3cr3tp4ss@db.ejemplo.cl:5432/prod
OTRO_TOKEN=aZ9kQ2mV7pL4wX8n"#,
    ] {
        assert!(told(plain).is_some(), "se le escapo: {plain}");
    }
}

#[test]
fn one_line_is_enough_when_the_value_names_its_provider() {
    for plain in [
        r#"GITHUB=ghp_16C7e42F292c6912E7710c838347Ae178B4a"#,
        r#"ANTHROPIC=sk-ant-api03-aaaaaaaaaaaaaaaaaaaaaaaaaaaa"#,
        r#"AMAZON=AKIAIOSFODNN7EXAMPLE"#,
    ] {
        assert!(told(plain).is_some(), "se le escapo: {plain}");
    }
}

#[test]
fn what_it_turned_away_says_which_line_and_which_name() {
    let plain = r#"# Despliegue

Primero esto:

DB_PASSWORD=Tr0ub4dor3xK
SMTP_PASSWORD=h4nsel4ndGretel"#;

    let said = told(plain).expect("lo vio");

    assert!(said.contains("line 5"), "{said}");
    assert!(said.contains("DB_PASSWORD"), "{said}");
    assert!(
        !said.contains("Tr0ub4dor3xK"),
        "el valor no se repite: {said}"
    );
}

#[test]
fn a_key_itself_is_a_key_wherever_it_sits() {
    let pem = r#"-----BEGIN RSA PRIVATE KEY-----
MIIEpAIBAAKCAQEA
-----END RSA PRIVATE KEY-----"#;

    assert!(a_key_itself(pem.as_bytes()).is_some());
    assert!(a_key_itself(&[0x30, 0x82, 0x04, 0xa4]).is_some());
    assert!(
        a_key_itself(r#"DB_PASSWORD=Tr0ub4dor3xK"#.as_bytes()).is_none(),
        "una linea de configuracion no es una clave en si"
    );
}

#[test]
fn a_json_of_credentials_is_seen_as_one() {
    let plain = r#"{
  "type": "service_account",
  "client_email": "tisty@ejemplo.iam.gserviceaccount.com",
  "private_key_id": "9f2a4c1d7b3e5a8c0f6d2b4e1a9c3f7d5b8e0a2c"
}"#;

    assert!(secret_in(plain.as_bytes()).is_some(), "no la vio");
}

#[test]
fn a_json_of_data_comes_through_untouched() {
    let plain = r#"{
  "name": "Tisty",
  "version": "1.4.1",
  "description": "una lista de tareas en esta maquina",
  "homepage": "https://ejemplo.cl/tisty"
}"#;

    assert_eq!(secret_in(plain.as_bytes()).map(|one| one.said()), None);
}

#[test]
fn nothing_is_turned_away_for_what_it_holds() {
    let dir = tempfile::tempdir().unwrap();
    for (named, bytes) in [
        ("subir.py", b"print(1)".as_slice()),
        ("paquete.json", b"{}"),
        ("pila.yaml", b"services:"),
        ("main.rs", b"fn main() {}"),
        ("arrancar.sh", b"#!/bin/sh"),
        ("tisty.exe", b"MZ\x90\x00binario"),
        ("firma.desconocida", b"cualquier cosa"),
        ("clave.pem", b"-----BEGIN RSA PRIVATE KEY-----"),
    ] {
        let at = dir.path().join(named);
        std::fs::write(&at, bytes).unwrap();
        assert!(
            tisty_core::agent::fit_to_keep(&at).is_ok(),
            "lo bloqueo: {named}"
        );
    }

    let lying = dir.path().join("holiday.png");
    std::fs::write(&lying, [0x30, 0x82, 0x0A, 0x00]).unwrap();
    assert!(
        tisty_core::agent::fit_to_keep(&lying).is_err(),
        "un png que no es png entro igual"
    );
}

#[test]
fn a_token_nobody_assigned_to_anything_is_still_a_token() {
    for plain in [
        r#"Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxIn0"#,
        r#"ghp_16C7e42F292c6912E7710c838347Ae178B4a"#,
        r#"postgres://admin:Pa55word123@db.example.com:5432/prod"#,
        r#"https://hooks.slack.com/services/EJEMPLO/EJEMPLO/EJEMPLO"#,
    ] {
        assert!(told(plain).is_some(), "se le escapo: {plain}");
    }
}

#[test]
fn prose_and_paths_that_merely_look_long_are_left_alone() {
    for plain in [
        r#"Descarga el instalador desde https://ejemplo.cl/descargas/tisty-1.4.1-x64.msi"#,
        r#"El fichero vive en /home/mario/proyectos/tisty/crates/tisty-core/src/agent.rs"#,
        r#"git clone https://github.com/rgdevment/tisty.git"#,
    ] {
        assert_eq!(told(plain), None, "lo tomo por secreto: {plain}");
    }
}

#[test]
fn tidying_a_document_keeps_the_name_a_fence_carries() {
    let raw = r#"---
title: viejo
---

# Guia

```rust title="src/walk.rs"
fn main() {}
```

```mermaid title="el flujo"
graph TD
  A --> B
```
"#;

    let made = tisty_core::arriving::tidied(raw);

    assert!(
        made.body.contains(r#"```rust title="src/walk.rs""#),
        "se perdio el nombre: {}",
        made.body
    );
    assert!(
        made.body.contains(r#"```mermaid title="el flujo""#),
        "se perdio el nombre del diagrama: {}",
        made.body
    );
    assert!(
        !made.changed.iter().any(|one| one.contains("fence")),
        "dijo que cambio el fence sin cambiarlo: {:?}",
        made.changed
    );
}

#[test]
fn a_fence_that_says_more_than_it_should_is_still_trimmed() {
    let raw = r#"---
title: viejo
---

```rust vamos a ver que pasa
fn main() {}
```
"#;

    let made = tisty_core::arriving::tidied(raw);

    assert!(
        made.body.contains(
            "```rust
"
        ),
        "{}",
        made.body
    );
    assert!(!made.body.contains("vamos a ver"), "{}", made.body);
}

#[test]
fn a_name_with_a_backtick_never_leaves_a_fence_a_document_cannot_reopen() {
    for raw in [
        r#"---
title: viejo
---

```rust title="usar `map`"
fn main() {}
```
"#,
        r#"---
title: viejo
---

```sh title="echo `date`"
ls
```
"#,
    ] {
        let made = tisty_core::arriving::tidied(raw);

        assert!(
            tisty_core::docs::survives(&made.body).is_ok(),
            "lo que salio del arreglo ya no se puede abrir: {}",
            made.body
        );
    }
}

#[test]
fn a_note_of_what_a_document_said_is_one_an_older_build_can_walk_past() {
    let op = tisty_core::Op::DocSaid {
        id: ulid::Ulid::generate(),
        d: tisty_core::event::Said {
            title: "Lo que dice".into(),
            bytes: Some(12),
        },
    };

    assert!(op.is_optional(), "un lector viejo se atragantaria con ella");
    assert!(op.settles(), "sin esto, deshacer se rompe tras escribir");
}

#[test]
fn no_other_operation_is_written_as_one_to_skip() {
    let told = tisty_core::Op::DocDelete {
        id: ulid::Ulid::generate(),
    };
    let moved = tisty_core::Op::DocMove {
        id: ulid::Ulid::generate(),
        d: tisty_core::event::Filed::default(),
    };

    assert!(!told.is_optional(), "borrar no se puede saltar");
    assert!(!moved.is_optional(), "mover no se puede saltar");
}

#[test]
fn a_note_reaches_the_document_it_speaks_of() {
    let room = tempfile::tempdir().unwrap();
    let store = room.path().join("store");
    let device = tisty_core::DeviceId("uno".into());
    let mut open = tisty_core::Store::open(&store, device).unwrap();

    let id = ulid::Ulid::generate();
    open.append(tisty_core::Op::DocAdd {
        id,
        d: tisty_core::event::DocAdd {
            file: "aaaa-0001".into(),
            order: "a0".into(),
            said: Some(tisty_core::event::Said {
                title: "Como nacio".into(),
                bytes: None,
            }),
            folder: None,
            page_of: None,
        },
    })
    .unwrap();

    let events = tisty_core::store::read_all(&store).unwrap();
    let state = tisty_core::State::replay(&events);
    assert_eq!(
        state.docs.get(&id).unwrap().title.as_deref(),
        Some("Como nacio")
    );

    open.append(tisty_core::Op::DocSaid {
        id,
        d: tisty_core::event::Said {
            title: "Como se llama ahora".into(),
            bytes: Some(40),
        },
    })
    .unwrap();

    let events = tisty_core::store::read_all(&store).unwrap();
    let state = tisty_core::State::replay(&events);
    let kept = state.docs.get(&id).unwrap();
    assert_eq!(kept.title.as_deref(), Some("Como se llama ahora"));
    assert_eq!(kept.bytes, Some(40));
    assert!(kept.wrote.is_some(), "la fecha sale del propio evento");
}

#[test]
fn a_log_written_before_the_note_existed_still_opens() {
    let room = tempfile::tempdir().unwrap();
    let store = room.path().join("store");
    let mut open = tisty_core::Store::open(&store, tisty_core::DeviceId("uno".into())).unwrap();

    let id = ulid::Ulid::generate();
    open.append(tisty_core::Op::DocAdd {
        id,
        d: tisty_core::event::DocAdd {
            file: "aaaa-0001".into(),
            order: "a0".into(),
            said: None,
            folder: None,
            page_of: None,
        },
    })
    .unwrap();

    let raw = std::fs::read_to_string(store.join("uno").join("active.tisty")).unwrap();
    assert!(
        !raw.contains("said"),
        "un documento sin nota escribe lo mismo que antes: {raw}"
    );

    let state = tisty_core::State::replay(&tisty_core::store::read_all(&store).unwrap());
    assert_eq!(state.docs.get(&id).unwrap().title, None);
}

#[test]
fn the_note_goes_out_marked_so_an_older_build_skips_it() {
    let room = tempfile::tempdir().unwrap();
    let store = room.path().join("store");
    let mut open = tisty_core::Store::open(&store, tisty_core::DeviceId("uno".into())).unwrap();

    open.append(tisty_core::Op::DocSaid {
        id: ulid::Ulid::generate(),
        d: tisty_core::event::Said {
            title: "Algo".into(),
            bytes: None,
        },
    })
    .unwrap();

    let raw = std::fs::read_to_string(store.join("uno").join("active.tisty")).unwrap();

    assert!(raw.contains("doc.said"), "{raw}");
    assert!(
        raw.contains("\"opt\":true"),
        "sin la marca, un Tisty anterior se niega a abrir el almacen: {raw}"
    );
    assert!(
        raw.contains(&format!("\"v\":{}", tisty_core::event::SCHEMA_VERSION)),
        "la nota va con la version del formato en vigor: {raw}"
    );
}
