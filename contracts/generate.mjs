// Turns the contracts into a package for each language: change the YAML, run this, and run vectors/make.mjs if the envelope's format changed.
import { existsSync, mkdirSync, readdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { loadAll } from "js-yaml";

const here = dirname(fileURLToPath(import.meta.url));
const dist = join(here, "dist");
const VERSION = readFileSync(join(here, "VERSION"), "utf8").trim();

const PRIMITIVES = new Set(["text", "bool", "whole", "number", "time", "moment", "name", "language", "tag_word", "key", "hash", "signed", "bytes"]);

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

const KEYS = new Set(["contract", "group", "checked_by", "says", "is", "except", "contains", "enums", "fields", "values", "draws", "never", "styles", "html_rules", "html_cases", "roles", "cases"]);

const names = new Map();
const enums = new Map();
for (const one of contracts) {
  // A misspelt key would quietly take its rules out of every language.
  for (const key of Object.keys(one)) if (!KEYS.has(key)) stop(`${one.contract} says ${key}, which a contract does not have`);
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

// A contract that is another one holds its fields, less those it gives up, and adds its own.
for (const one of contracts.filter((each) => each.is)) {
  const base = names.get(one.is);
  if (!base?.fields) stop(`${one.contract} is ${one.is}, which has no fields`);
  for (const field of one.except ?? []) {
    if (!base.fields[field]) stop(`${one.contract} gives up ${field}, which ${one.is} does not have`);
  }
  const kept = Object.entries(base.fields).filter(([field]) => !one.except?.includes(field));
  one.fields = { ...Object.fromEntries(kept), ...one.fields };
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
      const types = [...Object.values(fn.takes ?? {}), ...(fn.gives ? [fn.gives] : [])].map((type) => type.replace(/^list:/, ""));
      for (const type of types) if (!known(type)) stop(`${role}.${name} names ${type}, which no contract defines`);
    }
  }
}

const allowed = () => {
  const tags = new Map();
  for (const one of contracts) {
    for (const draw of one.draws ?? []) {
      const held = tags.get(draw.tag) ?? { attributes: new Set(), classes: new Set(), class_starts: new Set(), class_ends: new Set() };
      for (const name of draw.attributes ?? []) held.attributes.add(name);
      for (const name of draw.classes ?? []) held.classes.add(name);
      for (const name of draw.class_starts ?? []) held.class_starts.add(name);
      for (const name of draw.class_ends ?? []) held.class_ends.add(name);
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
      class_ends: [...held.class_ends].sort(),
    };
  }
  return out;
};

const drawn = allowed();
const htmlRules = contracts.flatMap((one) => one.html_rules ?? []);
for (const rule of htmlRules) {
  for (const name of rule.requires ?? [rule.attribute]) {
    if (!drawn[rule.tag]?.attributes.includes(name)) stop(`a rule looks at ${name} on ${rule.tag}, which no contract draws`);
  }
}

// A style names its kind of value in one of three ways, and on one tag or on several.
const styleRules = (one) =>
  (one.styles ?? []).flatMap(({ on, property, unit, is, sized, color }) => {
    const said = unit ? { unit } : color ? { color } : sized ? { sizes: one.values?.[sized] } : { values: one.values?.[is] };
    if (Object.values(said).some((held) => !held)) stop(`${one.contract} says a style on ${on} is ${is ?? sized}, which it does not list`);
    return [on].flat().map((tag) => ({ on: tag, property, ...said }));
  });

const whole = {
  version: VERSION,
  contracts: Object.fromEntries(contracts.map((one) => [one.contract, one])),
  enums: Object.fromEntries(enums),
  allowed: drawn,
  passthrough: { tags: names.get("text").values.passthrough_tags, attributes: names.get("text").values.passthrough_attributes },
  urls: {
    schemes: names.get("text").values.url_schemes,
    attributes: names.get("text").values.url_attributes,
    references: Object.keys(names.get("text").values.references),
    paints: names.get("diagram").values.paint_attributes,
  },
  styles: contracts.flatMap(styleRules),
  html_rules: htmlRules,
};

const pascal = (name) => name.replace(/(^|_)([a-z])/g, (_, __, letter) => letter.toUpperCase());
const camel = (name) => name.replace(/_([a-z])/g, (_, letter) => letter.toUpperCase());

const tsType = (type) => {
  if (type.startsWith("list:")) return `${tsType(type.slice(5))}[]`;
  if (type === "bool") return "boolean";
  if (type === "whole" || type === "number") return "number";
  if (type === "bytes") return "Uint8Array";
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
        const gives = fn.gives ? tsType(fn.gives) : "void";
        out.push(`  ${camel(name)}(${takes.join(", ")}): ${fn.fails ? `Promise<${gives}>` : gives};`);
      }
      out.push("}", "");
    }
  }
  out.push(
    `export type ContractName = ${contracts.map((one) => JSON.stringify(one.contract)).join(" | ")};`,
    "export declare const CONTRACTS: Record<ContractName, unknown>;",
    "export declare const ALLOWED: Record<string, { attributes: string[]; classes: string[]; class_starts: string[]; class_ends: string[] }>;",
    "/** What is wrong with a value under this contract: nothing when it holds. */",
    "export declare function check(contract: ContractName, value: unknown): string[];",
    "/** What in the HTML of a document is outside the contracts. It is no cleaner: it says whether a document speaks this language. */",
    "export declare function checkHtml(html: string): string[];",
    "",
  );
  return out.join("\n");
};

const typescriptRuntime = () => `const WHOLE = ${JSON.stringify(whole)};\n${readFileSync(join(here, "runtime", "check.js"), "utf8")}`;

const typescriptCases = () => `import assert from "node:assert/strict";
import { test } from "node:test";
import { check, checkHtml, CONTRACTS } from "./index.js";

for (const [name, contract] of Object.entries(CONTRACTS)) {
  for (const one of contract.cases ?? []) {
    test(name + ": " + one.says, () => {
      assert.equal(check(name, one.value).length === 0, one.holds, check(name, one.value).join("; "));
    });
  }
  for (const one of contract.html_cases ?? []) {
    test(name + " html: " + one.says, () => {
      assert.equal(checkHtml(one.html).length === 0, one.holds, checkHtml(one.html).join("; "));
    });
  }
}
`;

const rustType = (type) => {
  if (type.startsWith("list:")) return `Vec<${rustType(type.slice(5))}>`;
  if (type === "bool") return "bool";
  if (type === "whole") return "u64";
  if (type === "number") return "f64";
  if (type === "bytes") return "Vec<u8>";
  if (PRIMITIVES.has(type)) return "String";
  return pascal(type);
};

const RESERVED = new Set(["type", "ref", "move", "loop", "as", "in", "use", "do", "final", "override"]);
const snake = (name) => name.replace(/[A-Z]/g, (letter) => `_${letter.toLowerCase()}`);
const rustField = (name) => (RESERVED.has(snake(name)) ? `r#${snake(name)}` : snake(name));

const rustLib = () => {
  const out = [
    "//! The contracts of Tisty, generated from their YAML. Change the YAML, never this.",
    "",
    "use serde::{Deserialize, Serialize};",
    "",
    `pub const VERSION: &str = "${VERSION}";`,
    'pub const CONTRACT: &str = include_str!("contract.json");',
    "",
  ];
  for (const [name, values] of enums) {
    out.push("#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]");
    out.push(`pub enum ${pascal(name)} {`);
    for (const value of values) out.push(`    #[serde(rename = ${JSON.stringify(value)})]`, `    ${pascal(value.replace(/[^a-zA-Z0-9]/g, "_"))},`);
    out.push("}", "");
  }
  for (const one of contracts) {
    if (one.fields) {
      out.push(`/// ${one.says}`, "#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]");
      out.push(`pub struct ${pascal(one.contract)} {`);
      for (const [field, said] of Object.entries(one.fields)) {
        const type = rustType(said.type === "list" ? `list:${said.of}` : said.type);
        const renamed = snake(field) === field ? "" : `rename = "${field}", `;
        if (said.required) {
          if (renamed) out.push(`    #[serde(${renamed.slice(0, -2)})]`);
          out.push(`    pub ${rustField(field)}: ${type},`);
        } else {
          out.push(`    #[serde(${renamed}default, skip_serializing_if = "Option::is_none")]`);
          out.push(`    pub ${rustField(field)}: Option<${type}>,`);
        }
      }
      out.push("}", "");
    }
    for (const [role, functions] of Object.entries(one.roles ?? {})) {
      out.push(`pub trait ${pascal(one.contract)}${pascal(role)} {`);
      for (const [name, fn] of Object.entries(functions)) {
        const takes = Object.entries(fn.takes ?? {}).map(([arg, type]) => `${arg}: ${rustType(type)}`);
        const held = fn.gives ? rustType(fn.gives) : "()";
        const gives = fn.fails ? ` -> Result<${held}, String>` : fn.gives ? ` -> ${held}` : "";
        out.push(`    fn ${name}(${["&mut self", ...takes].join(", ")})${gives};`);
      }
      out.push("}", "");
    }
  }
  out.push(readFileSync(join(here, "runtime", "check.rs"), "utf8"));
  return out.join("\n");
};

const rustCases = `use serde_json::Value;
use tisty_contracts::{CONTRACT, check, check_html};

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
        for one in contract["html_cases"].as_array().into_iter().flatten() {
            let wrong = check_html(one["html"].as_str().unwrap());
            let holds = one["holds"].as_bool().unwrap();
            assert_eq!(wrong.is_empty(), holds, "{name} html: {}: {wrong:?}", one["says"]);
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
      exports: { ".": { types: "./index.d.ts", default: "./index.js" }, "./vectors.json": "./vectors.json" },
      files: ["index.js", "index.d.ts", "vectors.json"],
    },
    null,
    2,
  )}\n`,
  "typescript/index.js": typescriptRuntime(),
  "typescript/index.d.ts": typescriptTypes(),
  "typescript/cases.test.js": typescriptCases(),
  "typescript/vectors.test.js": readFileSync(join(here, "vectors", "open.test.js"), "utf8"),
  "typescript/vectors.json": readFileSync(join(here, "vectors", "vectors.json"), "utf8"),
  "typescript/written-by-tisty.json": readFileSync(join(here, "vectors", "written-by-tisty.json"), "utf8"),
  "rust/Cargo.toml": `[package]
name = "tisty-contracts"
version = "${VERSION}"
edition = "2024"
license = "AGPL-3.0-only"
description = "What a Tisty document and task are. Generated from Tisty's contracts."

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
unicode-normalization = "0.1"

[dev-dependencies]
aes-gcm = "0.10"
base64 = "0.23"
ed25519-dalek = "3"
flate2 = "1"
hkdf = "0.13"
hmac = "0.13"
sha2 = "0.11"

[workspace]
`,
  // Generated, so nobody formats it by hand, and a workspace that names it by path must not try.
  "rust/rustfmt.toml": `disable_all_formatting = true\n`,
  "rust/src/lib.rs": rustLib(),
  "rust/src/contract.json": `${JSON.stringify(whole)}\n`,
  "rust/tests/cases.rs": rustCases,
  "rust/tests/vectors.rs": readFileSync(join(here, "vectors", "open.rs"), "utf8"),
  "rust/vectors.json": readFileSync(join(here, "vectors", "vectors.json"), "utf8"),
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
