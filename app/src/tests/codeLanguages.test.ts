import { describe, expect, it } from "vitest";
import { languageAt, spokenIn } from "../ui/codeLanguages";

const BODY = 'Uno\n\n```rs title="a.rs"\nfn a() {}\n```\n\n~~~\nplano\n~~~\n';
const TITLED = BODY.indexOf("```");
const BARE = BODY.indexOf("~~~");

describe("the language a code block is written in", () => {
  it("is read from the line that opens it, or none", () => {
    expect(languageAt(BODY, TITLED)).toBe("rs");
    expect(languageAt(BODY, BARE)).toBe("");
    expect(languageAt(BODY, 0)).toBeNull();
  });

  it("changes on that line alone, the title kept", () => {
    expect(spokenIn(BODY, TITLED, "rust")).toBe(BODY.replace("```rs", "```rust"));
    expect(spokenIn(BODY, BARE, "python")).toBe(BODY.replace("~~~\nplano", "~~~python\nplano"));
    expect(spokenIn(BODY, 0, "rust")).toBeNull();
  });

  it("keeps a word before a title when the language is taken away, and none without one", () => {
    expect(spokenIn(BODY, TITLED, "")).toBe(BODY.replace("```rs", "```text"));
    const named = spokenIn(BODY, BARE, "go") ?? "";
    expect(spokenIn(named, BARE, "")).toBe(BODY);
  });

  it("keeps a Windows line end where the line had one", () => {
    const crlf = "```js\r\nx\r\n```\r\n";
    expect(spokenIn(crlf, 0, "ts")).toBe("```ts\r\nx\r\n```\r\n");
  });
});
