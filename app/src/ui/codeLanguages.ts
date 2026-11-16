// The languages the sheet colours, by the word a fence names them with and the name a person knows.
export const LANGUAGES: readonly (readonly [string, string])[] = [
  ["bash", "Bash"],
  ["c", "C"],
  ["cpp", "C++"],
  ["csharp", "C#"],
  ["css", "CSS"],
  ["diff", "Diff"],
  ["go", "Go"],
  ["graphql", "GraphQL"],
  ["html", "HTML"],
  ["ini", "INI / TOML"],
  ["java", "Java"],
  ["javascript", "JavaScript"],
  ["json", "JSON"],
  ["kotlin", "Kotlin"],
  ["lua", "Lua"],
  ["makefile", "Makefile"],
  ["markdown", "Markdown"],
  ["objectivec", "Objective-C"],
  ["perl", "Perl"],
  ["php", "PHP"],
  ["python", "Python"],
  ["r", "R"],
  ["ruby", "Ruby"],
  ["rust", "Rust"],
  ["scss", "SCSS"],
  ["shell", "Shell"],
  ["sql", "SQL"],
  ["swift", "Swift"],
  ["typescript", "TypeScript"],
  ["xml", "XML"],
  ["yaml", "YAML"],
];

const FENCE = /^([ \t>]*)(`{3,}|~{3,})([^\s`]*)(.*)$/;

/** The language a code block opening at `at` names, or "" when it names none. */
export const languageAt = (text: string, at: number): string | null => {
  const end = text.indexOf("\n", at);
  const fenced = FENCE.exec(text.slice(at, end < 0 ? text.length : end).replace(/\r$/, ""));
  return fenced ? (fenced[3] ?? "") : null;
};

/** The text with the code block opening at `at` naming `language`, its title and the rest kept. */
export const spokenIn = (text: string, at: number, language: string): string | null => {
  const end = text.indexOf("\n", at);
  const stop = end < 0 ? text.length : end;
  const line = text.slice(at, stop);
  const crlf = line.endsWith("\r") ? "\r" : "";
  const fenced = FENCE.exec(line.slice(0, line.length - crlf.length));
  if (!fenced) return null;
  const [, lead, fence, , rest] = fenced;
  // A title needs a word before it, or it would be read as the language.
  const named = language || (rest.trim() ? "text" : "");
  return `${text.slice(0, at)}${lead}${fence}${named}${rest}${crlf}${text.slice(stop)}`;
};
