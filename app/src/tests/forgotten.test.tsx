import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Filed, Folded } from "../core";
import Folder from "../ui/Folder";
import { clearOfChrome } from "../ui/WindowChrome";
import type { Forgotten } from "../useForgotten";

const DAY = 86_400;
const now = () => Math.floor(Date.now() / 1000);

const ipc = vi.hoisted(() => ({
  forgotten: [] as unknown[],
  picked: null as string | null,
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) => {
    if (cmd === "docs_forgotten") return Promise.resolve(ipc.forgotten);
    if (cmd === "doc_at_random") return Promise.resolve(ipc.picked);
    return Promise.resolve(null);
  },
}));

const folders: Folded[] = [
  {
    id: "f1",
    name: "Cocina",
    parent: null,
    icon: null,
    color: null,
    holds: 1,
    archived: false,
    away: false,
  },
];

const paper = (id: string, title: string, more: Partial<Filed> = {}): Filed => ({
  id,
  file: `hevp3wf9-${id}`,
  title,
  folder: null,
  archived: false,
  away: false,
  ...more,
});

const lost = (doc: string, daysAgo: number, lastOpenedHere = false): Forgotten => ({
  doc,
  last: now() - daysAgo * DAY,
  lastOpenedHere,
});

const show = (docs: Filed[], folder: Folded | null = null) => {
  const onOpen = vi.fn();
  render(<Folder folder={folder} folders={folders} docs={docs} onOpen={onOpen} onHere={vi.fn()} />);
  return { onOpen };
};

beforeEach(() => {
  ipc.forgotten = [];
  ipc.picked = null;
});

describe("the documents not opened in a while", () => {
  it("lists them at the top of the Library with their folder and how long ago", async () => {
    ipc.forgotten = [lost("1", 200), lost("2", 70)];
    show([paper("1", "Recetas de la abuela", { folder: "f1" }), paper("2", "Garantías")]);

    expect(await screen.findByRole("heading", { name: "Not opened in a while" })).toBeTruthy();
    const oldest = screen.getByRole("button", { name: /Recetas de la abuela/ });
    expect(oldest.textContent).toContain("Cocina · 6 months ago");
    expect(oldest.getAttribute("title")).toMatch(/^Last written on /);
  });

  it("says when the last day was an opening on this machine", async () => {
    ipc.forgotten = [lost("1", 90, true)];
    show([paper("1", "Vacunas de Rina")]);

    const strip = await screen.findByRole("region", { name: "Not opened in a while" });
    const row = within(strip).getByRole("button", { name: /Vacunas de Rina/ });
    expect(row.getAttribute("title")).toMatch(/^You last opened it here on /);
  });

  it("shows five and unfolds the rest on asking", async () => {
    const docs = Array.from({ length: 7 }, (_, n) => paper(String(n), `Hoja ${n}`));
    ipc.forgotten = docs.map((one, n) => lost(one.id, 100 - n));
    show(docs);

    const more = await screen.findByRole("button", { name: "See all 7" });
    const strip = screen.getByRole("region", { name: "Not opened in a while" });
    expect(strip.querySelectorAll("li")).toHaveLength(5);

    await userEvent.click(more);

    expect(strip.querySelectorAll("li")).toHaveLength(7);
    expect(screen.getByRole("button", { name: "See fewer" })).toBeTruthy();
  });

  it("leaves out what was put away or marked since the list was read", async () => {
    ipc.forgotten = [lost("1", 100), lost("2", 100)];
    show([
      paper("1", "Guardada", { archived: true, away: true }),
      paper("2", "Marcada", { flagged: { at: "2026-06-01T00:00:00Z", said: "Una copia" } }),
    ]);

    await screen.findByRole("button", { name: "A document at random" });
    expect(screen.queryByRole("heading", { name: "Not opened in a while" })).toBeNull();
  });

  it("finds a document by the language the reader reads it in", async () => {
    ipc.forgotten = [lost("anchor", 100)];
    show([paper("en", "Recipe", { tongue: ["anchor", 1] })]);

    expect(await screen.findByRole("button", { name: /Recipe/ })).toBeTruthy();
  });

  it("opens a document at random", async () => {
    ipc.picked = "2";
    const { onOpen } = show([paper("1", "Una"), paper("2", "Otra")]);

    await userEvent.click(await screen.findByRole("button", { name: "A document at random" }));

    await vi.waitFor(() =>
      expect(onOpen).toHaveBeenCalledWith(expect.objectContaining({ id: "2" })),
    );
  });

  it("keeps the dice clear of the window buttons Windows draws at the top right", async () => {
    show([paper("1", "Una")]);

    const dice = await screen.findByRole("button", { name: "A document at random" });
    expect(dice.parentElement?.className).toContain(clearOfChrome);
  });

  it("stays out of a folder", async () => {
    ipc.forgotten = [lost("1", 100)];
    show([paper("1", "Recetas", { folder: "f1" })], folders[0]);

    await screen.findByRole("button", { name: /Recetas/ });
    expect(screen.queryByRole("button", { name: "A document at random" })).toBeNull();
    expect(screen.queryByRole("region", { name: "Not opened in a while" })).toBeNull();
  });
});
