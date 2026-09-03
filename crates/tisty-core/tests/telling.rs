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
