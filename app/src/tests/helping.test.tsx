import { readFileSync } from "node:fs";
import { act, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { topicsOf } from "../guide/reference";
import { kindOf, slugOf, TOPICS } from "../guide/topicOf";
import { inTheRepo } from "./repo";

const reference = (lang: string) =>
  readFileSync(inTheRepo("crates/tisty-markdown/reference", `${lang}.md`), "utf8");

vi.mock("../core", async (original) => ({
  ...(await original<typeof import("../core")>()),
  helpReference: () =>
    Promise.resolve(
      "# Guide\n\nIntro.\n\n## Glance\n\n| a |\n\n## Pens\n\nA pen.\n\n### Colours\n\nSome.\n\n## Tables\n\nA table.",
    ),
  helpDrawn: (body: string) => Promise.resolve(`<p>${body.length}</p>`),
  helpAnchors: () => Promise.resolve([["guide", "glance"], ["pens", "colours"], ["tables"]]),
  openLink: () => Promise.resolve(),
}));

vi.mock("../ui/Page", () => ({
  default: ({ label, onHere }: { label: string; onHere?: (id: string) => void }) => (
    <section aria-label="page">
      <h2>{label}</h2>
      <button type="button" onClick={() => onHere?.("colours")}>
        jump
      </button>
    </section>
  ),
}));

import Help from "../ui/Help";
import HelpKey from "../ui/HelpKey";

describe("the topic a block of writing asks about", () => {
  it("names topics both references have, in both languages", () => {
    const english = new Set(topicsOf(reference("en")).map((one) => one.slug));
    const spanish = new Set(topicsOf(reference("es")).map((one) => one.slug));
    for (const [en, es] of Object.values(TOPICS)) {
      expect(english.has(en), en).toBe(true);
      expect(spanish.has(es), es).toBe(true);
    }
  });

  it("reads a block's shape first, then what the caret's line carries", () => {
    expect(kindOf("```mermaid\ngraph TD\n```")).toBe("drawn");
    expect(kindOf("```rust\nfn x() {}\n```")).toBe("code");
    expect(kindOf("| a | b |\n| - | - |")).toBe("tables");
    expect(kindOf("> [!NOTE]\n> ojo")).toBe("callouts");
    expect(kindOf(":::details Más\nx\n:::")).toBe("folds");
    expect(kindOf("## Título")).toBe("headings");
    expect(kindOf("- [ ] algo")).toBe("lists");
    expect(kindOf("---")).toBe("breaks");
    expect(kindOf("---\ntitle: Plan\n---")).toBe("properties");
    expect(kindOf("un ==rotulador==")).toBe("pens");
    expect(kindOf("[[Plan]] aquí")).toBe("references");
    expect(kindOf("ver https://x.y")).toBe("links");
    expect(kindOf("sobre #viaje")).toBe("tags");
    expect(kindOf("solo palabras")).toBeNull();
    expect(kindOf(":::nota rara\nx\n:::")).toBeNull();
    expect(kindOf("ticket #1234")).toBeNull();
    expect(kindOf("a las 10:30:00 en punto")).toBeNull();
    expect(kindOf("listo :tada:")).toBe("emoji");
    expect(kindOf(":::tip\nojo\n:::")).toBe("callouts");
    expect(slugOf("tables", true)).toBe("tablas");
    expect(slugOf("tables", false)).toBe("tables");
  });
});

describe("asking for the guide", () => {
  it("opens it on F1", () => {
    const onHelp = vi.fn();
    render(<HelpKey onHelp={onHelp} />);
    act(() => {
      window.dispatchEvent(new KeyboardEvent("keydown", { key: "F1" }));
    });
    expect(onHelp).toHaveBeenCalledWith(undefined);
  });
});

describe("the guide inside the window", () => {
  it("lists every topic, opens the one asked for and walks to the next", async () => {
    render(<Help topic="pens" />);
    expect(await screen.findByRole("heading", { name: "Pens" })).toBeTruthy();
    const nav = screen.getByRole("navigation", { name: /topics|temas/i });
    expect(nav.textContent).toContain("Tables");
    await userEvent.click(within(nav).getByRole("button", { name: "Tables" }));
    expect(await screen.findByRole("heading", { name: "Tables" })).toBeTruthy();
  });

  it("follows a link to a section of another topic", async () => {
    render(<Help topic="tables" />);
    await screen.findByRole("heading", { name: "Tables" });
    await userEvent.click(screen.getByRole("button", { name: "jump" }));
    await waitFor(() => expect(screen.getByRole("heading", { name: "Pens" })).toBeTruthy());
  });
});
