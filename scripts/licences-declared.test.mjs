import assert from "node:assert/strict";
import { test } from "node:test";
import { saysItsLicence, undeclared } from "./licences-declared.mjs";

test("a package says its licence only with the project's one", () => {
  assert.ok(saysItsLicence("package.json", '{ "license": "AGPL-3.0-only" }'));
  assert.ok(!saysItsLicence("package.json", '{ "license": "MIT" }'));
  assert.ok(!saysItsLicence("package.json", '{ "name": "x" }'));
});

test("a crate takes it from the workspace or names it", () => {
  assert.ok(saysItsLicence("crates/x/Cargo.toml", "[package]\nlicense.workspace = true\n"));
  assert.ok(saysItsLicence("Cargo.toml", '[workspace.package]\nlicense = "AGPL-3.0-only"\n'));
  assert.ok(!saysItsLicence("crates/x/Cargo.toml", '[package]\nlicense = "MIT"\n'));
  assert.ok(!saysItsLicence("crates/x/Cargo.toml", "[package]\nlicense.workspace = false\n"));
  assert.ok(!saysItsLicence("crates/x/Cargo.toml", '[package]\nname = "x"\n'));
});

test("every manifest that does not say it is named", () => {
  const said = { "a/package.json": '{ "license": "MIT" }', "b/package.json": '{ "license": "AGPL-3.0-only" }', "c/Cargo.toml": "" };
  assert.deepEqual(undeclared(Object.keys(said), (path) => said[path]), ["a/package.json", "c/Cargo.toml"]);
});

test("a manifest that cannot be read stops the check", () => {
  const unreadable = () => {
    throw new Error("no such file");
  };
  assert.throws(() => undeclared(["a/package.json"], unreadable));
  assert.throws(() => undeclared(["a/package.json"], () => "{ not json"));
});
