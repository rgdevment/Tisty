import { afterEach, describe, expect, it, vi } from "vitest";
import { edited } from "../../src/edit/blocking";
import type { Hosted } from "../../src/edit/hosted";

const OWN = "text/x-tisty-markdown";

const host = (fromHtml: Hosted["fromHtml"], fromText: Hosted["fromText"]): Hosted => ({
  say: (word) => word,
  tableEditNear: () => Promise.reject(new Error("no table here")),
  fromHtml,
  fromText,
});

const editors: { leave: () => void }[] = [];

afterEach(() => {
  for (const one of editors.splice(0)) one.leave();
  document.body.innerHTML = "";
});

const opened = (said: string, owner: Hosted) => {
  const block = document.createElement("p");
  block.textContent = said;
  document.body.append(block);
  const editing = edited(
    owner,
    block,
    said,
    () => {},
    () => {},
  );
  editors.push(editing);
  return editing.view;
};

const pasted = (view: { contentDOM: HTMLElement }, faces: Record<string, string>) => {
  const event = new Event("paste", { bubbles: true, cancelable: true });
  Object.defineProperty(event, "clipboardData", {
    value: { files: [], getData: (kind: string) => faces[kind] ?? "" },
  });
  view.contentDOM.dispatchEvent(event);
  return event;
};

const settled = () => new Promise((done) => setTimeout(done, 0));

describe("a paste in the Markdown view", () => {
  it("lands Tisty's own Markdown as written and asks nobody to translate", async () => {
    const fromHtml = vi.fn(async () => "HTML");
    const fromText = vi.fn(async () => "TEXT");
    const view = opened("", host(fromHtml, fromText));
    const event = pasted(view, {
      [OWN]: "**Hola** [sitio](https://x.y)",
      "text/plain": "Hola sitio",
      "text/html": "<p><b>Hola</b> sitio</p>",
    });
    await settled();
    expect(view.state.doc.toString()).toBe("**Hola** [sitio](https://x.y)");
    expect(event.defaultPrevented).toBe(true);
    expect(fromHtml).not.toHaveBeenCalled();
    expect(fromText).not.toHaveBeenCalled();
  });

  it("lands its lines with LF when they came with CRLF", async () => {
    const view = opened("", host(vi.fn(), vi.fn()));
    pasted(view, { [OWN]: "uno\r\ndos" });
    await settled();
    expect(view.state.doc.toString()).not.toContain("\r");
    expect(view.state.doc.toString()).toContain("uno");
    expect(view.state.doc.toString()).toContain("dos");
  });

  it("still goes through the html translation when there is no own type", async () => {
    const fromHtml = vi.fn(async () => "**traducido**");
    const fromText = vi.fn(async () => "TEXT");
    const view = opened("", host(fromHtml, fromText));
    pasted(view, { "text/plain": "traducido", "text/html": "<p><b>traducido</b></p>" });
    await settled();
    expect(fromHtml).toHaveBeenCalledWith("<p><b>traducido</b></p>");
    expect(fromText).not.toHaveBeenCalled();
    expect(view.state.doc.toString()).toBe("**traducido**");
  });

  it("translates nothing for a plain paste of words with no own type", async () => {
    const fromHtml = vi.fn(async () => "HTML");
    const fromText = vi.fn(async () => "TEXT");
    const view = opened("", host(fromHtml, fromText));
    pasted(view, { "text/plain": "solo palabras" });
    await settled();
    expect(fromHtml).not.toHaveBeenCalled();
    expect(fromText).not.toHaveBeenCalled();
  });
});
