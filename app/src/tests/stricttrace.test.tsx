import { render, waitFor } from "@testing-library/react";
import { createElement, StrictMode } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Filed } from "../core";
import Docs from "../ui/Docs";

type Order = (file: string, before: string | null) => string;

const store = vi.hoisted(() => ({
  bodies: {} as Record<string, string>,
  log: [] as string[],
  movers: [] as unknown[],
  writes: [] as string[],
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    if (cmd === "doc_read") return Promise.resolve(store.bodies[String(args?.id)] ?? "");
    if (cmd === "doc_write") return Promise.resolve({ id: String(args?.id), title: "Libro" });
    return Promise.resolve(null);
  },
}));

vi.mock("../ui/Editor", async () => {
  const actual = await vi.importActual<typeof import("../ui/Editor")>("../ui/Editor");
  const Real = actual.default;
  return {
    ...actual,
    default: (props: Record<string, unknown>) =>
      createElement(Real, {
        ...props,
        onOrder: (move: unknown) => {
          store.log.push(`order:${move === null ? "null" : typeof move}`);
          store.movers.push(move);
          (props.onOrder as ((m: unknown) => void) | undefined)?.(move);
        },
        onWrite: (text: string) => {
          store.writes.push(text);
          (props.onWrite as (text: string) => void)(text);
        },
      } as never),
  };
});

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
];

const book = [
  "# Libro",
  "",
  "Algo.",
  "",
  "![El pod](tisty:doc/a3f1-0002)",
  "",
  "![El túnel](tisty:doc/a3f1-0003)",
  "",
].join("\n");

const alts = (root: ParentNode) =>
  Array.from(root.querySelectorAll(".tisty-doc img")).map((one) => one.getAttribute("alt"));

describe("which editor the mover Docs keeps belongs to", () => {
  beforeEach(() => {
    store.bodies = { "a3f1-0001": book };
    store.log = [];
    store.movers = [];
    store.writes = [];
  });

  const open = async (strict: boolean) => {
    const tree = (
      <Docs open="a3f1-0001" known={known} onKept={vi.fn()} onError={vi.fn()} onDoc={vi.fn()} />
    );
    const { container } = render(strict ? <StrictMode>{tree}</StrictMode> : tree);
    await waitFor(() => expect(container.querySelector('[data-move="a3f1-0002:1"]')).toBeTruthy());
    await new Promise((go) => setTimeout(go, 60));
    return container;
  };

  const tried = async (strict: boolean) => {
    const container = await open(strict);
    const held = store.movers.filter(Boolean) as Order[];
    store.writes = [];
    const said = held.length ? held[held.length - 1]("a3f1-0002", null) : "nothing";
    await new Promise((go) => setTimeout(go, 20));
    return { said, writes: store.writes.length, dom: alts(container) };
  };

  /// A render React throws away builds an editor of its own and fires its `create`, so the last
  /// mover handed up could be one that answers about a document nobody can see. It must not be.
  it("the mover it keeps moves the editor on screen, mounted once", async () => {
    expect(await tried(false)).toEqual({
      said: "done",
      writes: 1,
      dom: ["El túnel", "El pod"],
    });
  });

  it("the mover it keeps moves the editor on screen, mounted the way the app mounts", async () => {
    expect(await tried(true)).toEqual({
      said: "done",
      writes: 1,
      dom: ["El túnel", "El pod"],
    });
  });
});
