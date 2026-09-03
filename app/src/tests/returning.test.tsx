import { render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Filed } from "../core";
import Docs from "../ui/Docs";

const store = vi.hoisted(() => ({ bodies: {} as Record<string, string> }));

let watching: (() => void)[] = [];

class Watcher {
  constructor(private tell: () => void) {}
  observe() {
    watching.push(this.tell);
  }
  disconnect() {
    watching = watching.filter((one) => one !== this.tell);
  }
}

const grew = () => {
  for (const tell of [...watching]) tell();
};

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) =>
    cmd === "doc_read"
      ? Promise.resolve(store.bodies[String(args?.id)] ?? "")
      : Promise.resolve(null),
}));

const known: Filed[] = [
  { id: "01A", file: "a3f1-0001", title: "El documento", folder: null, archived: false },
  {
    id: "01B",
    file: "a3f1-0002",
    title: "Su pagina",
    folder: null,
    archived: false,
    pageOf: "01A",
  } as Filed,
];

const largo = (title: string) =>
  `# ${title}\n\n${Array.from({ length: 80 }, (_, n) => `Parrafo numero ${n} del texto.`).join("\n\n")}`;

beforeEach(() => {
  watching = [];
  vi.stubGlobal("ResizeObserver", Watcher);
  store.bodies = {
    "a3f1-0001": largo("El documento"),
    "a3f1-0002": largo("Su pagina"),
  };
});

const scroller = () => document.querySelector<HTMLElement>(".scroller");

const SEEN = 600;

const clamped = (at: HTMLElement, room: () => number) => {
  if (Object.getOwnPropertyDescriptor(at, "scrollTop")) return;
  let put = 0;
  Object.defineProperty(at, "clientHeight", { get: () => SEEN, configurable: true });
  Object.defineProperty(at, "scrollHeight", { get: room, configurable: true });
  Object.defineProperty(at, "scrollTop", {
    configurable: true,
    get: () => put,
    set: (asked: number) => {
      put = Math.max(0, Math.min(asked, room() - SEEN));
    },
  });
};

describe("volver de una pagina a su documento", () => {
  it("deja el documento donde estaba cuando la altura ya esta", async () => {
    const shown = render(
      <Docs open="a3f1-0001" known={known} onKept={vi.fn()} onError={vi.fn()} />,
    );
    await waitFor(() => expect(scroller()).not.toBeNull());
    await screen.findByText(/Parrafo numero 3 /);

    const at = scroller();
    if (!at) throw new Error("sin scroller");
    clamped(at, () => 4000);
    at.scrollTop = 800;
    at.dispatchEvent(new Event("scroll", { bubbles: true }));

    shown.rerender(<Docs open="a3f1-0002" known={known} onKept={vi.fn()} onError={vi.fn()} />);
    await screen.findByText(/Su pagina/);

    shown.rerender(<Docs open="a3f1-0001" known={known} onKept={vi.fn()} onError={vi.fn()} />);
    await screen.findByText(/Parrafo numero 3 /);
    const back = scroller();
    if (!back) throw new Error("sin scroller al volver");
    clamped(back, () => 4000);

    await waitFor(() => expect(back.scrollTop).toBe(800), { timeout: 3000 });
  });

  it("lo deja donde estaba tambien cuando la altura tarda en llegar", async () => {
    const shown = render(
      <Docs open="a3f1-0001" known={known} onKept={vi.fn()} onError={vi.fn()} />,
    );
    await waitFor(() => expect(scroller()).not.toBeNull());
    await screen.findByText(/Parrafo numero 3 /);

    const at = scroller();
    if (!at) throw new Error("sin scroller");
    clamped(at, () => 4000);
    at.scrollTop = 800;
    at.dispatchEvent(new Event("scroll", { bubbles: true }));

    shown.rerender(<Docs open="a3f1-0002" known={known} onKept={vi.fn()} onError={vi.fn()} />);
    await screen.findByText(/Su pagina/);

    shown.rerender(<Docs open="a3f1-0001" known={known} onKept={vi.fn()} onError={vi.fn()} />);
    await screen.findByText(/Parrafo numero 3 /);
    const back = scroller();
    if (!back) throw new Error("sin scroller al volver");

    let tall = SEEN;
    clamped(back, () => tall);

    await waitFor(() => expect(back.scrollTop).toBe(0));
    tall = 4000;
    grew();

    await waitFor(() => expect(back.scrollTop).toBe(800), { timeout: 3000 });
  });
});
