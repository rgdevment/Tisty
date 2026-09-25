import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { StrictMode } from "react";
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
  page("01E", "a3f1-0005", "La suelta", "01A"),
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

const numbered = () =>
  screen
    .getAllByRole("listitem")
    .map((one) => one.textContent ?? "")
    .filter((one) => one.length > 0);

const opened = async (strict: boolean) => {
  const tree = (
    <Docs open="a3f1-0001" known={known} onKept={vi.fn()} onError={vi.fn()} onDoc={vi.fn()} />
  );
  const { container } = render(strict ? <StrictMode>{tree}</StrictMode> : tree);
  await waitFor(() => expect(container.querySelector('[data-move="a3f1-0002:1"]')).toBeTruthy());
  return container;
};

const both = (strict: boolean) => {
  it("moves the first chapter down when its button is pressed", async () => {
    const container = await opened(strict);

    const down = container.querySelector<HTMLButtonElement>('[data-move="a3f1-0002:1"]');
    if (!down) throw new Error("the first chapter offers no way down");
    fireEvent.click(down);

    await waitFor(() =>
      expect(numbered().slice(0, 3)).toEqual([
        expect.stringContaining("El túnel"),
        expect.stringContaining("El pod"),
        expect.stringContaining("La VPN cae"),
      ]),
    );
  });

  it("puts a loose page into the text when its button is pressed", async () => {
    const container = await opened(strict);

    const put = container.querySelector<HTMLButtonElement>(".leaf-put");
    if (!put) throw new Error("the loose page offers no way in");
    fireEvent.click(put);

    await waitFor(() => expect(numbered()[3]).toEqual(expect.stringContaining("04La suelta")));
  });
};

beforeEach(() => {
  store.bodies = { "a3f1-0001": book };
  store.wrote = [];
});

describe("the index of a document React mounts once", () => both(false));

/// The app mounts inside <React.StrictMode>, so a development build renders every component twice
/// and builds two ProseMirror editors. Both fire TipTap's onCreate, and the handles Editor pushes
/// up from there are kept last-wins, so Docs ends up holding the one belonging to the editor React
/// threw away. Both of these fail.
describe("the index of a document React mounts the way the app does", () => both(true));
