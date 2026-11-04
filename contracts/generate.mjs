// Turns the contracts into a package for each language. With --check it only says whether what is kept is up to date.
import { existsSync, mkdirSync, readdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { loadAll } from "js-yaml";

const here = dirname(fileURLToPath(import.meta.url));
const dist = join(here, "dist");
const VERSION = readFileSync(join(here, "VERSION"), "utf8").trim();

const PRIMITIVES = new Set(["text", "bool", "whole", "number", "time", "language", "key", "signed"]);

const yamlFiles = (folder) =>
  readdirSync(folder).flatMap((name) => {
    const path = join(folder, name);
    if (statSync(path).isDirectory()) return name === "dist" ? [] : yamlFiles(path);
    return name.endsWith(".yaml") ? [path] : [];
  });

const contracts = yamlFiles(here)
  .sort()
  .flatMap((path) => loadAll(readFileSync(path, "utf8")));

const stop = (said) => {
  console.error(`x  contracts: ${said}`);
  process.exit(1);
};

const names = new Map();
const enums = new Map();
for (const one of contracts) {
  if (!one.contract || !one.group || !one.says || !Array.isArray(one.checked_by)) {
    stop(`${one.contract ?? "a contract"} lacks contract, group, says or checked_by`);
  }
  if (names.has(one.contract)) stop(`${one.contract} is written twice`);
  names.set(one.contract, one);
  for (const [name, values] of Object.entries(one.enums ?? {})) {
    if (enums.has(name)) stop(`the enum ${name} is written twice`);
    enums.set(name, values);
  }
}

const hasFields = (name) => Boolean(names.get(name)?.fields);

const known = (type) => PRIMITIVES.has(type) || enums.has(type) || hasFields(type);

// A wrong type here would pass unseen in every language, so nothing is guessed: what is not known stops the publish.
for (const one of contracts) {
  for (const [field, said] of Object.entries(one.fields ?? {})) {
    const inner = said.type === "list" ? said.of : said.type;
    if (!known(inner)) stop(`${one.contract}.${field} says it is ${inner}, which no contract defines`);
  }
  for (const part of one.contains ?? []) {
    if (!names.has(part)) stop(`${one.contract} contains ${part}, which no contract defines`);
  }
  for (const [role, functions] of Object.entries(one.roles ?? {})) {
    for (const [name, fn] of Object.entries(functions)) {
      const types = [...Object.values(fn.takes ?? {}), ...(fn.gives ? [fn.gives.replace(/^list:/, "")] : [])];
      for (const type of types) if (!known(type)) stop(`${role}.${name} names ${type}, which no contract defines`);
    }
  }
}

const allowed = () => {
  const tags = new Map();
  for (const one of contracts) {
    for (const draw of one.draws ?? []) {
      const held = tags.get(draw.tag) ?? { attributes: new Set(), classes: new Set(), class_starts: new Set() };
      for (const name of draw.attributes ?? []) held.attributes.add(name);
      for (const name of draw.classes ?? []) held.classes.add(name);
      for (const name of draw.class_starts ?? []) held.class_starts.add(name);
      tags.set(draw.tag, held);
    }
  }
  const out = {};
  for (const tag of [...tags.keys()].sort()) {
    const held = tags.get(tag);
    out[tag] = {
      attributes: [...held.attributes].sort(),
      classes: [...held.classes].sort(),
      class_starts: [...held.class_starts].sort(),
    };
  }
  return out;
};

const whole = {
  version: VERSION,
  contracts: Object.fromEntries(contracts.map((one) => [one.contract, one])),
  enums: Object.fromEntries(enums),
  allowed: allowed(),
};

const pascal = (name) => name.replace(/(^|_)([a-z])/g, (_, __, letter) => letter.toUpperCase());
const camel = (name) => name.replace(/_([a-z])/g, (_, letter) => letter.toUpperCase());

const tsType = (type) => {
  if (type.startsWith("list:")) return `${tsType(type.slice(5))}[]`;
  if (type === "bool") return "boolean";
  if (type === "whole" || type === "number") return "number";
  if (PRIMITIVES.has(type)) return "string";
  return pascal(type);
};

const typescriptTypes = () => {
  const out = [`export declare const VERSION: "${VERSION}";`, ""];
  for (const [name, values] of enums) {
    out.push(`export type ${pascal(name)} = ${values.map((one) => JSON.stringify(one)).join(" | ")};`, "");
  }
  for (const one of contracts) {
    if (one.fields) {
      out.push(`/** ${one.says} */`, `export interface ${pascal(one.contract)} {`);
      for (const [field, said] of Object.entries(one.fields)) {
        const type = said.type === "list" ? `list:${said.of}` : said.type;
        out.push(`  ${field}${said.required ? "" : "?"}: ${tsType(type)};`);
      }
      out.push("}", "");
    }
    for (const [role, functions] of Object.entries(one.roles ?? {})) {
      out.push(`export interface ${pascal(one.contract)}${pascal(role)} {`);
      for (const [name, fn] of Object.entries(functions)) {
        const takes = Object.entries(fn.takes ?? {}).map(([arg, type]) => `${arg}: ${tsType(type)}`);
        out.push(`  ${camel(name)}(${takes.join(", ")}): ${fn.gives ? tsType(fn.gives) : "void"};`);
      }
      out.push("}", "");
    }
  }
  out.push(
    `export type ContractName = ${contracts.map((one) => JSON.stringify(one.contract)).join(" | ")};`,
    "export declare const CONTRACTS: Record<ContractName, unknown>;",
    "export declare const ALLOWED: Record<string, { attributes: string[]; classes: string[]; class_starts: string[] }>;",
    "/** What is wrong with a value under this contract: nothing when it holds. */",
    "export declare function check(contract: ContractName, value: unknown): string[];",
    "",
  );
  return out.join("\n");
};

const typescriptRuntime = () => `const WHOLE = ${JSON.stringify(whole)};
export const VERSION = WHOLE.version;
export const CONTRACTS = WHOLE.contracts;
export const ALLOWED = WHOLE.allowed;

const isTime = (said) => {
  const found = /^(\\d{4})-(\\d{2})-(\\d{2})T(\\d{2}):(\\d{2}):(\\d{2})Z$/.exec(said);
  if (!found) return false;
  const [month, day, hour, minute, second] = found.slice(2).map(Number);
  return month >= 1 && month <= 12 && day >= 1 && day <= 31 && hour <= 23 && minute <= 59 && second <= 59;
};

const base64urlOf = (said, length) => said.length === length && /^[A-Za-z0-9_-]+$/.test(said);

const lengthOf = (said) => [...said].length;

const checkType = (type, value, at, out) => {
  if (type === "text") {
    if (typeof value !== "string") out.push(at + " is not text");
  } else if (type === "bool") {
    if (typeof value !== "boolean") out.push(at + " is not true or false");
  } else if (type === "whole") {
    if (!Number.isSafeInteger(value) || value < 0) out.push(at + " is not a whole number");
  } else if (type === "number") {
    if (typeof value !== "number") out.push(at + " is not a number");
  } else if (type === "time") {
    if (typeof value !== "string" || !isTime(value)) out.push(at + " is not a date in whole seconds");
  } else if (type === "language") {
    if (typeof value !== "string" || !/^[A-Za-z0-9-]{2,35}$/.test(value)) out.push(at + " is not a language tag");
  } else if (type === "key") {
    if (typeof value !== "string" || !base64urlOf(value, 43)) out.push(at + " is not a key");
  } else if (type === "signed") {
    if (typeof value !== "string" || !base64urlOf(value, 86)) out.push(at + " is not a signature");
  } else if (WHOLE.enums[type]) {
    if (!WHOLE.enums[type].includes(value)) out.push(at + " is none of " + WHOLE.enums[type].join(", "));
  } else {
    checkFields(type, value, at, out);
  }
};

function checkFields(name, value, at, out) {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    out.push(at + " is not a " + name);
    return;
  }
  for (const [field, said] of Object.entries(WHOLE.contracts[name].fields ?? {})) {
    const here = at === "" ? field : at + "." + field;
    const given = value[field];
    if (given === undefined || given === null) {
      if (said.required) out.push(here + " is missing");
      continue;
    }
    if (said.type === "list") {
      if (!Array.isArray(given)) {
        out.push(here + " is not a list");
        continue;
      }
      if (said.at_least !== undefined && given.length < said.at_least) out.push(here + " has too few");
      if (said.at_most !== undefined && given.length > said.at_most) out.push(here + " has too many");
      given.forEach((one, n) => checkType(said.of, one, here + "[" + n + "]", out));
      continue;
    }
    checkType(said.type, given, here, out);
    const size = typeof given === "string" ? lengthOf(given) : given;
    if (typeof size === "number" && (typeof given === "string" || said.type === "whole" || said.type === "number")) {
      if (said.at_least !== undefined && size < said.at_least) out.push(here + " is too short or too low");
      if (said.at_most !== undefined && size > said.at_most) out.push(here + " is too long or too high");
    }
  }
}

export function check(contract, value) {
  if (!WHOLE.contracts[contract]) return ["there is no contract called " + contract];
  const out = [];
  checkFields(contract, value, "", out);
  return out;
}
`;

const typescriptCases = () => `import assert from "node:assert/strict";
import { test } from "node:test";
import { check, CONTRACTS } from "./index.js";

for (const [name, contract] of Object.entries(CONTRACTS)) {
  for (const one of contract.cases ?? []) {
    test(name + ": " + one.says, () => {
      assert.equal(check(name, one.value).length === 0, one.holds, check(name, one.value).join("; "));
    });
  }
}
`;

const rustType = (type) => {
  if (type.startsWith("list:")) return `Vec<${rustType(type.slice(5))}>`;
  if (type === "bool") return "bool";
  if (type === "whole") return "u64";
  if (type === "number") return "f64";
  if (PRIMITIVES.has(type)) return "String";
  return pascal(type);
};

const RESERVED = new Set(["type", "ref", "move", "loop", "as", "in", "use", "do", "final", "override"]);
const rustField = (name) => (RESERVED.has(name) ? `r#${name}` : name);

const rustLib = () => {
  const out = [
    "//! The contracts of Tisty, generated from their YAML. Change the YAML, never this.",
    "",
    "use serde::{Deserialize, Serialize};",
    "use serde_json::Value;",
    "",
    `pub const VERSION: &str = "${VERSION}";`,
    'pub const CONTRACT: &str = include_str!("contract.json");',
    "",
  ];
  for (const [name, values] of enums) {
    out.push("#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]");
    out.push('#[serde(rename_all = "kebab-case")]', `pub enum ${pascal(name)} {`);
    for (const value of values) out.push(`    ${pascal(value)},`);
    out.push("}", "");
  }
  for (const one of contracts) {
    if (one.fields) {
      out.push(`/// ${one.says}`, "#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]");
      out.push(`pub struct ${pascal(one.contract)} {`);
      for (const [field, said] of Object.entries(one.fields)) {
        const type = rustType(said.type === "list" ? `list:${said.of}` : said.type);
        if (said.required) {
          out.push(`    pub ${rustField(field)}: ${type},`);
        } else {
          out.push('    #[serde(default, skip_serializing_if = "Option::is_none")]');
          out.push(`    pub ${rustField(field)}: Option<${type}>,`);
        }
      }
      out.push("}", "");
    }
    for (const [role, functions] of Object.entries(one.roles ?? {})) {
      out.push(`pub trait ${pascal(one.contract)}${pascal(role)} {`);
      for (const [name, fn] of Object.entries(functions)) {
        const takes = Object.entries(fn.takes ?? {}).map(([arg, type]) => `${arg}: ${rustType(type)}`);
        const gives = fn.gives ? ` -> ${rustType(fn.gives)}` : "";
        out.push(`    fn ${name}(${["&mut self", ...takes].join(", ")})${gives};`);
      }
      out.push("}", "");
    }
  }
  out.push(`fn whole() -> Value {
    serde_json::from_str(CONTRACT).expect("the contract is generated as JSON")
}

fn is_time(said: &str) -> bool {
    let b = said.as_bytes();
    let digits = |from: usize, to: usize| b[from..to].iter().all(u8::is_ascii_digit);
    let number = |from: usize, to: usize| said[from..to].parse::<u32>().unwrap_or(99);
    b.len() == 20
        && digits(0, 4)
        && b[4] == b'-'
        && digits(5, 7)
        && b[7] == b'-'
        && digits(8, 10)
        && b[10] == b'T'
        && digits(11, 13)
        && b[13] == b':'
        && digits(14, 16)
        && b[16] == b':'
        && digits(17, 19)
        && b[19] == b'Z'
        && (1..=12).contains(&number(5, 7))
        && (1..=31).contains(&number(8, 10))
        && number(11, 13) <= 23
        && number(14, 16) <= 59
        && number(17, 19) <= 59
}

fn base64url_of(said: &str, length: usize) -> bool {
    said.len() == length
        && said
            .bytes()
            .all(|one| one.is_ascii_alphanumeric() || one == b'-' || one == b'_')
}

fn is_language(said: &str) -> bool {
    (2..=35).contains(&said.len())
        && said
            .bytes()
            .all(|one| one.is_ascii_alphanumeric() || one == b'-')
}

fn check_type(all: &Value, kind: &str, value: &Value, at: &str, out: &mut Vec<String>) {
    let wrong = |said: &str| format!("{at} {said}");
    match kind {
        "text" => {
            if !value.is_string() {
                out.push(wrong("is not text"));
            }
        }
        "bool" => {
            if !value.is_boolean() {
                out.push(wrong("is not true or false"));
            }
        }
        "whole" => {
            if !value.is_u64() {
                out.push(wrong("is not a whole number"));
            }
        }
        "number" => {
            if !value.is_number() {
                out.push(wrong("is not a number"));
            }
        }
        "time" => {
            if !value.as_str().is_some_and(is_time) {
                out.push(wrong("is not a date in whole seconds"));
            }
        }
        "language" => {
            if !value.as_str().is_some_and(is_language) {
                out.push(wrong("is not a language tag"));
            }
        }
        "key" => {
            if !value.as_str().is_some_and(|one| base64url_of(one, 43)) {
                out.push(wrong("is not a key"));
            }
        }
        "signed" => {
            if !value.as_str().is_some_and(|one| base64url_of(one, 86)) {
                out.push(wrong("is not a signature"));
            }
        }
        _ => match all["enums"].get(kind) {
            Some(values) => {
                let listed = values.as_array().is_some_and(|many| many.contains(value));
                if !listed {
                    out.push(wrong("is none of what its list allows"));
                }
            }
            None => check_fields(all, kind, value, at, out),
        },
    }
}

fn check_fields(all: &Value, name: &str, value: &Value, at: &str, out: &mut Vec<String>) {
    let Some(given) = value.as_object() else {
        out.push(format!("{at} is not a {name}"));
        return;
    };
    let Some(fields) = all["contracts"][name]["fields"].as_object() else {
        return;
    };
    for (field, said) in fields {
        let here = if at.is_empty() { field.clone() } else { format!("{at}.{field}") };
        let one = given.get(field).filter(|one| !one.is_null());
        let Some(one) = one else {
            if said["required"].as_bool() == Some(true) {
                out.push(format!("{here} is missing"));
            }
            continue;
        };
        let least = said["at_least"].as_u64();
        let most = said["at_most"].as_u64();
        if said["type"] == "list" {
            let Some(many) = one.as_array() else {
                out.push(format!("{here} is not a list"));
                continue;
            };
            if least.is_some_and(|floor| (many.len() as u64) < floor) {
                out.push(format!("{here} has too few"));
            }
            if most.is_some_and(|top| many.len() as u64 > top) {
                out.push(format!("{here} has too many"));
            }
            let of = said["of"].as_str().unwrap_or_default();
            for (n, item) in many.iter().enumerate() {
                check_type(all, of, item, &format!("{here}[{n}]"), out);
            }
            continue;
        }
        let kind = said["type"].as_str().unwrap_or_default();
        check_type(all, kind, one, &here, out);
        let size = match one {
            Value::String(text) => Some(text.chars().count() as u64),
            Value::Number(number) => number.as_u64(),
            _ => None,
        };
        if let Some(size) = size {
            if least.is_some_and(|floor| size < floor) {
                out.push(format!("{here} is too short or too low"));
            }
            if most.is_some_and(|top| size > top) {
                out.push(format!("{here} is too long or too high"));
            }
        }
    }
}

/// What is wrong with a value under this contract: nothing when it holds.
pub fn check(contract: &str, value: &Value) -> Vec<String> {
    let all = whole();
    if all["contracts"].get(contract).is_none() {
        return vec![format!("there is no contract called {contract}")];
    }
    let mut out = Vec::new();
    check_fields(&all, contract, value, "", &mut out);
    out
}
`);
  return out.join("\n");
};

const rustCases = `use serde_json::Value;
use tisty_contracts::{CONTRACT, check};

#[test]
fn every_case_says_what_the_contract_says() {
    let all: Value = serde_json::from_str(CONTRACT).unwrap();
    let contracts = all["contracts"].as_object().unwrap();
    let mut ran = 0;
    for (name, contract) in contracts {
        for one in contract["cases"].as_array().into_iter().flatten() {
            let wrong = check(name, &one["value"]);
            let holds = one["holds"].as_bool().unwrap();
            assert_eq!(wrong.is_empty(), holds, "{name}: {}: {wrong:?}", one["says"]);
            ran += 1;
        }
    }
    assert!(ran > 0);
}
`;

const files = {
  "contract.json": `${JSON.stringify(whole, null, 2)}\n`,
  "typescript/package.json": `${JSON.stringify(
    {
      name: "@tisty/contracts",
      version: VERSION,
      description: "What a Tisty document and task are. Generated from Tisty's contracts.",
      license: "AGPL-3.0-only",
      repository: { type: "git", url: "git+https://github.com/rgdevment/Tisty.git", directory: "contracts/dist/typescript" },
      type: "module",
      main: "index.js",
      types: "index.d.ts",
      exports: { ".": { types: "./index.d.ts", default: "./index.js" } },
      files: ["index.js", "index.d.ts"],
    },
    null,
    2,
  )}\n`,
  "typescript/index.js": typescriptRuntime(),
  "typescript/index.d.ts": typescriptTypes(),
  "typescript/cases.test.js": typescriptCases(),
  "rust/Cargo.toml": `[package]
name = "tisty-contracts"
version = "${VERSION.replace(/-.*/, "")}"
edition = "2024"
license = "AGPL-3.0-only"
description = "What a Tisty document and task are. Generated from Tisty's contracts."

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"

[workspace]
`,
  "rust/src/lib.rs": rustLib(),
  "rust/src/contract.json": `${JSON.stringify(whole)}\n`,
  "rust/tests/cases.rs": rustCases,
};

const checking = process.argv.includes("--check");
const stale = [];
for (const [name, said] of Object.entries(files)) {
  const path = join(dist, name);
  if (checking) {
    if (!existsSync(path) || readFileSync(path, "utf8") !== said) stale.push(name);
    continue;
  }
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, said);
}
if (stale.length > 0) stop(`what is kept in dist is not what the YAML says: ${stale.join(", ")}; run node contracts/generate.mjs`);
