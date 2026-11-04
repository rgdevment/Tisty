# Tisty's contracts

What a document and a task are. Whoever works with them speaks this language,
and implements only the contracts it needs.

Change the YAML, then run `node contracts/generate.mjs`. It writes a package for
each language into `dist`, with the types, the interfaces and a `check` that
holds a value to its contract. Each contract's `cases` run in every language.

```sh
node contracts/generate.mjs
node --test contracts/dist/typescript/cases.test.js contracts/tisty.test.mjs
cargo test --manifest-path contracts/dist/rust/Cargo.toml
```
