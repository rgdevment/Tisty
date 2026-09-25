import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Filed } from "../core";
import Docs from "../ui/Docs";

const store = vi.hoisted(() => ({
  bodies: {} as Record<string, string>,
  wrote: [] as { id: string; body: string }[],
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    if (cmd === "doc_read") return Promise.resolve(store.bodies[String(args?.id)] ?? "");
    if (cmd === "doc_write") {
      store.wrote.push({ id: String(args?.id), body: String(args?.body) });
      return Promise.resolve({ id: String(args?.id), title: "Libro" });
    }
    return Promise.resolve(null);
  },
}));

const page = (id: string, file: string, title: string, pageOf?: string): Filed => ({
  id,
  file,
  title,
  folder: null,
  archived: false,
  away: false,
  pageOf: pageOf ?? null,
});

const known = [
  page("01A", "a3f1-0001", "Libro"),
  page("01B", "a3f1-0002", "El pod", "01A"),
  page("01C", "a3f1-0003", "El túnel", "01A"),
  page("01D", "a3f1-0004", "La VPN cae", "01A"),
];

const book = [
  "# Libro",
  "",
  "Algo antes de los capítulos.",
  "",
  "![El pod](tisty:doc/a3f1-0002)",
  "",
  "![El túnel](tisty:doc/a3f1-0003)",
  "",
  "![La VPN cae](tisty:doc/a3f1-0004)",
  "",
].join("\n");

describe("moving a chapter from the index, through the real editor", () => {
  beforeEach(() => {
    store.bodies = { "a3f1-0001": book };
    store.wrote = [];
  });

  const open = async () => {
    const { container } = render(
      <Docs open="a3f1-0001" known={known} onKept={vi.fn()} onError={vi.fn()} onDoc={vi.fn()} />,
    );
    await waitFor(() => expect(container.querySelector('[data-move="a3f1-0002:1"]')).toBeTruthy());
    return container;
  };

  it("numbers the chapters the text names", async () => {
    await open();

    expect(screen.getAllByRole("listitem").map((one) => one.textContent?.slice(0, 2))).toEqual([
      "01",
      "02",
      "03",
    ]);
  });

  it("moves the first chapter down when its button is pressed", async () => {
    const container = await open();

    const down = container.querySelector<HTMLButtonElement>('[data-move="a3f1-0002:1"]');
    if (!down) throw new Error("the first chapter offers no way down");
    fireEvent.click(down);

    await waitFor(() =>
      expect(screen.getAllByRole("listitem").map((one) => one.textContent)).toEqual([
        expect.stringContaining("El túnel"),
        expect.stringContaining("El pod"),
        expect.stringContaining("La VPN cae"),
      ]),
    );
  });

  it("moves it with Alt and an arrow as well", async () => {
    const container = await open();

    const row = container.querySelector<HTMLElement>('[data-leaf="a3f1-0004"]');
    if (!row) throw new Error("no row for the last chapter");
    fireEvent.keyDown(row, { key: "ArrowUp", altKey: true });

    await waitFor(() =>
      expect(screen.getAllByRole("listitem").map((one) => one.textContent)).toEqual([
        expect.stringContaining("El pod"),
        expect.stringContaining("La VPN cae"),
        expect.stringContaining("El túnel"),
      ]),
    );
  });
});
