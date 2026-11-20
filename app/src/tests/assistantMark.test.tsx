import { render } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { AssistantMark, FolderMark, levelOf } from "../ui/AssistantMark";
import Glyph from "../ui/Glyph";

const folder = (name: string, parent: string | null, icon: string | null = null) => ({
  name,
  parent,
  icon,
});

const ASSISTANTS = ["claude", "codex", "copilot", "gemini", "cursor"];

const glyphOf = (name: string) => {
  const { container, unmount } = render(<Glyph name={name} />);
  const said = container.querySelector("svg.glyph")?.innerHTML ?? "";
  unmount();
  return said;
};

const drawn = (id: string) => {
  const { container, unmount } = render(<AssistantMark id={id} />);
  const said = container.querySelector("svg.glyph")?.innerHTML ?? "";
  unmount();
  return said;
};

describe("which assistant a folder in Agents stands for", () => {
  it("reads it from the folder's name right under Agents, as the core files it", () => {
    expect(levelOf(folder("Global", "agents"), "agents")).toBe("global");
    expect(levelOf(folder("Claude", "agents"), "agents")).toBe("claude");
    expect(levelOf(folder("Codex", "agents"), "agents")).toBe("codex");
    expect(levelOf(folder("Copilot", "agents"), "agents")).toBe("copilot");
    expect(levelOf(folder("Gemini", "agents"), "agents")).toBe("gemini");
    expect(levelOf(folder("Cursor", "agents"), "agents")).toBe("cursor");
  });

  it("leaves every other folder alone, and one the person gave an icon of their own", () => {
    expect(levelOf(folder("claude", "agents"), "agents")).toBeNull();
    expect(levelOf(folder("Notas", "agents"), "agents")).toBeNull();
    expect(levelOf(folder("Claude", "personal"), "agents")).toBeNull();
    expect(levelOf(folder("Claude", "agents", "star"), "agents")).toBeNull();
    expect(levelOf(folder("Claude", null), null)).toBeNull();
  });
});

describe("an assistant's mark", () => {
  it("gives each assistant a glyph of the app's own set, none like another's", () => {
    const glyphs = ASSISTANTS.map(drawn);
    for (const one of glyphs) expect(one).not.toBe("");
    expect(new Set(glyphs).size).toBe(ASSISTANTS.length);
  });

  it("borrows no logo: only plain outline glyphs, never a filled shape", () => {
    for (const id of ASSISTANTS) {
      const { container, unmount } = render(<AssistantMark id={id} />);
      expect(container.querySelector(".solid")).toBeNull();
      expect(container.querySelector("svg.glyph")?.getAttribute("viewBox")).toBe("0 0 24 24");
      unmount();
    }
  });

  it("draws every assistant in the sidebar's secondary grey, and only Global as a blue globe", () => {
    for (const id of ASSISTANTS) {
      const { container, unmount } = render(<AssistantMark id={id} />);
      expect(container.querySelector(`[data-mark="${id}"]`)?.className).toContain("text-soft");
      unmount();
    }
    const { container } = render(<AssistantMark id="global" />);
    expect(container.querySelector('[data-mark="global"]')?.className).toContain("text-hue-blue");
    expect(container.querySelector("svg.glyph")?.innerHTML).toBe(glyphOf("globe-earth"));
  });

  it("falls back to the terminal for an assistant it does not know yet", () => {
    expect(drawn("windsurf")).toBe(glyphOf("terminal"));
  });
});

describe("a folder's mark", () => {
  it("is the assistant's for a folder in Agents and the folder's own anywhere else", () => {
    const one = render(<FolderMark folder={folder("Claude", "agents")} root="agents" />);
    expect(one.container.querySelector('[data-mark="claude"]')).toBeTruthy();
    one.unmount();
    const other = render(<FolderMark folder={folder("Claude", "work")} root="agents" />);
    expect(other.container.querySelector("[data-mark]")).toBeNull();
    expect(other.container.querySelector("svg.glyph")?.innerHTML).toBe(glyphOf("folder"));
  });
});
