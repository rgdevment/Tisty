import { render, screen, within } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { AgentsLook } from "../agentsSync";
import type { Folded, Papers } from "../core";
import { hide, showAll } from "../hiding";
import Tree from "../ui/Tree";

const seen = vi.hoisted(() => ({ look: null as unknown }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) => Promise.resolve(cmd === "agents_look" ? seen.look : []),
}));

const folder = (id: string, name: string, more: Partial<Folded> = {}): Folded => ({
  id,
  name,
  parent: null,
  icon: null,
  holds: 1,
  archived: false,
  away: false,
  ...more,
});

const papers: Papers = {
  folders: [folder("A", "AGENTS", { icon: "robot" }), folder("W", "trabajo")],
  docs: [{ id: "D1", file: "f1", title: "Reglas", folder: "A", archived: false, away: false }],
};

const look = (more: Partial<AgentsLook>): AgentsLook => ({
  folder: "A",
  docs: 1,
  changes: 0,
  waiting: 0,
  unagreed: 0,
  proposed: 0,
  consented: true,
  allOn: true,
  asks: false,
  assistants: [],
  both: [],
  loose: [],
  unled: [],
  ...more,
});

const hidden = () => screen.getByRole("list", { name: "Hidden" });

describe("AGENTS hidden from the sidebar", () => {
  beforeEach(() => {
    showAll();
    hide("A");
  });

  it("still says a change waits for the person", async () => {
    seen.look = look({ changes: 1 });
    render(<Tree papers={papers} onOpen={vi.fn()} onFile={vi.fn()} onHere={vi.fn()} />);
    expect(await within(hidden()).findByText("1 changed")).toBeTruthy();
  });

  it("stays quiet while everything is in sync", async () => {
    seen.look = look({});
    render(<Tree papers={papers} onOpen={vi.fn()} onFile={vi.fn()} onHere={vi.fn()} />);
    await new Promise((ready) => setTimeout(ready, 0));
    expect(within(hidden()).getByText("AGENTS")).toBeTruthy();
    expect(within(hidden()).queryByText("In sync")).toBeNull();
  });
});
