export const steps = ({ quick, bash }) => [
  ["the Rust is formatted", "cargo", ["fmt", "--all", "--check"]],
  ["clippy has nothing to say", "cargo", ["clippy", "--workspace", "--all-targets", "--", "-D", "warnings"]],
  ["advisories and licences", "cargo", ["deny", "check"]],
  ["the window is linted", "npm", ["run", "lint"], "app"],
  ["the window builds", "npm", ["run", "build"], "app"],
  ["each window reaches only its own code", "node", ["scripts/windows-apart.mjs"], "app"],
  ["the light viewer stays light, with no React, and the app has no Solid", "node", ["scripts/light-viewer.mjs"], "app"],
  ["the documents are linted", "npm", ["run", "lint:md"]],
  ["the house rules hold", bash, ["scripts/rules.sh"]],
  ["the notices match the lockfiles", "node", ["scripts/third-party.mjs"]],
  ["the notices did not move", "git", ["diff", "--exit-code", "--", "THIRD-PARTY-LICENSES.md", "THIRD-PARTY-BUNDLED.md"]],
  ...(quick
    ? []
    : [
        ["the Rust tests pass", "cargo", ["nextest", "run", "--workspace", "--no-tests=pass"]],
        ["the window tests pass", "npm", ["test"], "app"],
        ["the light viewer's tests pass", "npm", ["test", "-w", "tisty-light-viewer"], "app"],
      ]),
];
