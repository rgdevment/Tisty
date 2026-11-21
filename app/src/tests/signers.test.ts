import { afterEach, describe, expect, it } from "vitest";
import { worn } from "../ui/dressing";
import { sourceAt, spotOf } from "../ui/pageSpots";
import { signers } from "../ui/signers";

const BODY = "## Notas del asistente\n\n**2026-10-29 12:04** · Claude Code\nJira actualizado.\n";
const NAME = BODY.indexOf("Claude Code");
const AFTER = NAME + "Claude Code".length;
const byte = (units: number) => new TextEncoder().encode(BODY.slice(0, units)).length;
const [name, after, words, end] = [NAME, AFTER, BODY.indexOf("Jira"), BODY.length - 1].map(byte);

// As the core draws a note an assistant signed, on the page an editor writes on: places in bytes.
const PAGE =
  '<h2 data-at="0-23" contenteditable="false"><span data-src="3-22">Notas del asistente</span></h2>' +
  `<p data-at="24-${end}"><strong><span data-src="26-42">2026-10-29 12:04</span></strong>` +
  `<span data-src="44-${name}"> · </span>` +
  `<span class="signer" data-by="Claude Code" data-src="${name}-${after}" contenteditable="false">Claude Code</span>` +
  `<span data-src="${after}-${after + 1}">\n</span><span data-src="${words}-${end}">Jira actualizado.</span></p>`;

afterEach(() => {
  document.body.innerHTML = "";
});

const pageOf = () => {
  document.body.innerHTML = `<article class="page">${PAGE}</article>`;
  return document;
};

const signer = () => document.querySelector<HTMLElement>(".signer") as HTMLElement;

describe("who signed an assistant's note on a task's page", () => {
  it("shows the name as written while the person wants it named", () => {
    signers(pageOf(), "named");
    expect(signer().textContent).toBe("Claude Code");
    expect(signer().hasAttribute("title")).toBe(false);
  });

  it("says only an agent when asked, with the name left on hover", () => {
    signers(pageOf(), "agent");
    expect(signer().textContent).toBe("an agent");
    expect(signer().title).toBe("Claude Code wrote it");
    expect(document.body.textContent).not.toContain("Claude Code");
  });

  it("draws only a diamond when asked, named for the screen reader", () => {
    signers(pageOf(), "mark");
    expect(signer().textContent).toBe("◆");
    expect(signer().getAttribute("aria-label")).toBe("Claude Code wrote it");
  });

  it("brings the name back when the person asks for it again", () => {
    const page = pageOf();
    signers(page, "agent");
    signers(page, "named");
    expect(signer().textContent).toBe("Claude Code");
    expect(signer().hasAttribute("title")).toBe(false);
    expect(signer().hasAttribute("aria-label")).toBe(false);
  });

  it("changes nothing when the choice did not change", () => {
    const page = pageOf();
    signers(page, "agent");
    const watch = new MutationObserver(() => {});
    watch.observe(page.body, { subtree: true, attributes: true, childList: true });
    signers(page, "agent");
    expect(watch.takeRecords()).toEqual([]);
    watch.disconnect();
  });

  it("writes around the shown name without touching the one written", () => {
    signers(pageOf(), "agent");
    expect(sourceAt(BODY, signer(), 0)).toBe(AFTER);
    const spot = spotOf(document, BODY, AFTER);
    expect(spot?.[0].parentElement?.classList.contains("signer")).toBe(false);
    expect(BODY.slice(NAME, AFTER)).toBe("Claude Code");
  });

  it("is dressed with the rest of the page as it is drawn", () => {
    worn(pageOf(), () => true, {
      body: BODY,
      onDoc: () => {},
      signing: "agent",
    });
    expect(signer().textContent).toBe("an agent");
  });
});
