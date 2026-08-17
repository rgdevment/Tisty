import { Editor } from "@tiptap/core";
import { describe, expect, it, vi } from "vitest";
import { stripped } from "../ui/Editor";
import { asMarkdown, written } from "../ui/writing";

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => Promise.resolve(null) }));

const md = (html: string) => {
  const editor = new Editor({ extensions: written(), content: stripped(html) });
  const out = asMarkdown(editor) ?? "";
  editor.destroy();
  return out;
};

describe("a picture arriving from the clipboard", () => {
  it("never writes the path it came from into the file", () => {
    expect(md('<p><img src="file:///Users/someone/Desktop/secret.png" alt="x"></p>')).not.toContain(
      "Users",
    );
  });

  it("drops the placeholder a webview invents for a pasted image", () => {
    expect(md('<p><img src="webkit-fake-url://1234/image.png"></p>')).not.toContain("webkit");
  });

  it("drops a resolved asset url instead of persisting it", () => {
    expect(md('<p><img src="asset://localhost/%2FUsers%2Fsomeone%2Fx.png"></p>')).not.toContain(
      "asset:",
    );
  });

  it("keeps a picture from the web, which is a real address", () => {
    expect(md('<p><img src="https://example.org/x.png" alt="web"></p>')).toContain(
      "https://example.org/x.png",
    );
  });

  it("keeps one of our own attachments", () => {
    expect(md('<p><img src="attachments/aa/x-1234.png" alt="mine"></p>')).toContain(
      "attachments/aa/x-1234.png",
    );
  });
});
