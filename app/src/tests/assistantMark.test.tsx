import { render } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { AssistantMark, FolderMark, levelOf } from "../ui/AssistantMark";

const folder = (name: string, parent: string | null, icon: string | null = null) => ({
  name,
  parent,
  icon,
});

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
  it("draws each assistant's logo filled in the colour around it", () => {
    for (const id of ["claude", "copilot", "gemini", "cursor"]) {
      const { container, unmount } = render(<AssistantMark id={id} />);
      const svg = container.querySelector(`svg[data-mark="${id}"]`);
      expect(svg?.getAttribute("viewBox")).toBe("0 0 24 24");
      expect(svg?.querySelector("path.solid")?.getAttribute("d")).toMatch(/^[Mm]/);
      unmount();
    }
  });

  it("draws Global as a blue globe", () => {
    const { container } = render(<AssistantMark id="global" />);
    expect(container.querySelector(".text-hue-blue svg.glyph")).toBeTruthy();
  });

  it("draws a neutral glyph for an assistant with no logo to borrow", () => {
    const { container } = render(<AssistantMark id="codex" />);
    expect(container.querySelector("svg.glyph")).toBeTruthy();
    expect(container.querySelector("[data-mark]")).toBeNull();
  });
});

describe("a folder's mark", () => {
  it("is the assistant's for a folder in Agents and the folder's own anywhere else", () => {
    const one = render(<FolderMark folder={folder("Claude", "agents")} root="agents" />);
    expect(one.container.querySelector('[data-mark="claude"]')).toBeTruthy();
    one.unmount();
    const other = render(<FolderMark folder={folder("Claude", "work")} root="agents" />);
    expect(other.container.querySelector("[data-mark]")).toBeNull();
    expect(other.container.querySelector("svg.glyph")).toBeTruthy();
  });
});

describe("the tone of an assistant's mark", () => {
  it("draws every assistant in the sidebar's secondary grey, and only Global in blue", () => {
    for (const id of ["claude", "copilot", "gemini", "cursor", "codex"]) {
      const { container, unmount } = render(<AssistantMark id={id} />);
      expect(container.querySelector(".text-soft")).toBeTruthy();
      expect(container.querySelector(".text-hue-blue")).toBeNull();
      unmount();
    }
  });
});
