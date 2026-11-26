import { describe, expect, it, vi } from "vitest";
import { arrival, type Clipboard, pastedWords, type Translator } from "../ui/clipboarding";
import edgeChat from "./fixtures/clipboard/edge-chat.dt.json";
import edgeChatPlain from "./fixtures/clipboard/edge-chat.dt-plain.json";
import edgePage from "./fixtures/clipboard/edge-page.dt.json";
import conhost from "./fixtures/clipboard/powershell-conhost.dt.json";
import conhostPlain from "./fixtures/clipboard/powershell-conhost.dt-plain.json";

interface Captured {
  readonly types: readonly string[];
  readonly plain: string;
  readonly html: string;
  readonly tisty: string;
}

const board = (captured: Captured): Clipboard => {
  const held: Record<string, string> = {
    "text/plain": captured.plain,
    "text/html": captured.html,
    "text/x-tisty-markdown": captured.tisty,
  };
  return { getData: (format) => (captured.types.includes(format) ? (held[format] ?? "") : "") };
};

const translator = (over: Partial<Translator> = {}): Translator => ({
  fromHtml: vi.fn(async () => "from html"),
  fromText: vi.fn(async () => null),
  ...over,
});

describe("real clipboards", () => {
  it("keeps the real CRLF in what was captured", () => {
    expect(edgePage.plain).toContain("\r\n");
    expect(conhost.plain).toContain("\r\n");
  });

  it("Ctrl+V of an Edge page arrives as html with plain text free of CR", () => {
    const arrived = arrival(board(edgePage));
    expect(arrived.kind).toBe("html");
    if (arrived.kind !== "html") return;
    expect(arrived.html).toBe(edgePage.html);
    expect(arrived.plain).not.toContain("\r");
    expect(arrived.plain).toContain("Título principal\nSección número 2\n");
  });

  it("Ctrl+Shift+V of typed Markdown arrives literal, without CR, every mark escaped", () => {
    expect(edgeChatPlain.types).toEqual(["text/plain"]);
    const arrived = arrival(board(edgeChatPlain), true);
    expect(arrived.kind).toBe("literal");
    if (arrived.kind !== "literal") return;
    expect(arrived.text).not.toContain("\r");
    expect(arrived.text).toContain(["\\# Título", "\\- uno", "\\- dos", ""].join("\n"));
    expect(arrived.text).toContain(String.raw`\*\*negrita\*\* y \[enlace\](https\://ejemplo.cl)`);
    expect(arrived.text).toContain(String.raw`\| A \| B \|`);
    expect(arrived.text).toContain(String.raw`\`código\``);
  });

  it("Ctrl+V of an Edge chat still carries the html the core must read", () => {
    const arrived = arrival(board(edgeChat));
    expect(arrived.kind).toBe("html");
  });

  it("a console copy arrives as plain text without CR", () => {
    const arrived = arrival(board(conhost));
    expect(arrived.kind).toBe("plain");
    if (arrived.kind !== "plain") return;
    expect(arrived.plain).not.toContain("\r");
    expect(arrived.plain).toContain("Windows PowerShell\nCopyright");
  });

  it("when the core has nothing to say about plain text, the page gets it without CR", async () => {
    const by = translator({ fromText: vi.fn(async () => null) });
    const said = await pastedWords(arrival(board(conhost)), by);
    expect(by.fromText).toHaveBeenCalledOnce();
    expect(said).not.toContain("\r");
    expect(said).toBe(conhost.plain.replace(/\r\n/g, "\n"));
  });

  it("Ctrl+Shift+V of a console keeps its columns and its blank lines", () => {
    const arrived = arrival(board(conhostPlain), true);
    expect(arrived.kind).toBe("literal");
    if (arrived.kind !== "literal") return;
    const nb = String.fromCharCode(160);
    const nl = String.fromCharCode(10);
    const bs = String.fromCharCode(92);
    expect(arrived.text).not.toContain(String.fromCharCode(13));
    expect(arrived.text).toContain(`Name ${nb.repeat(13)}Id`);
    expect(arrived.text).toContain(`${bs}---- ${nb.repeat(13)}--`);
    expect(arrived.text).toContain(`AggregatorHost ${nb}1956`);
    expect(arrived.text).toContain(`reserved.${nl}${nl}${nb}${nl}Name`);
    expect(arrived.text.endsWith(`PS C:${bs}${bs}harness${bs}${bs}in${bs}>`)).toBe(true);
  });
});
