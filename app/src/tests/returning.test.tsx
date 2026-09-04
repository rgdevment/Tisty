import { render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
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
  tall = 4000;
  restore = measured();
  vi.stubGlobal("ResizeObserver", Watcher);
  store.bodies = {
    "a3f1-0001": `${largo("El documento")}

![La Rina](tisty:doc/a3f1-0002)
`,
    "a3f1-0002": `# Su pagina\n\nUna hoja aparte, corta y con sus propias palabras.`,
  };
});

const scroller = () => document.querySelector<HTMLElement>(".scroller");

const SEEN = 600;
let tall = 4000;
const puts = new WeakMap<HTMLElement, number>();

const rolls = (one: HTMLElement) => one.classList?.contains("scroller") ?? false;

const measured = () => {
  const proto = HTMLElement.prototype;
  const kept = {
    clientHeight: Object.getOwnPropertyDescriptor(proto, "clientHeight"),
    scrollHeight: Object.getOwnPropertyDescriptor(proto, "scrollHeight"),
    scrollTop: Object.getOwnPropertyDescriptor(proto, "scrollTop"),
  };
  Object.defineProperty(proto, "clientHeight", {
    configurable: true,
    get(this: HTMLElement) {
      return rolls(this) ? SEEN : 0;
    },
  });
  Object.defineProperty(proto, "scrollHeight", {
    configurable: true,
    get(this: HTMLElement) {
      return rolls(this) ? tall : 0;
    },
  });
  Object.defineProperty(proto, "scrollTop", {
    configurable: true,
    get(this: HTMLElement) {
      return puts.get(this) ?? 0;
    },
    set(this: HTMLElement, asked: number) {
      puts.set(this, rolls(this) ? Math.max(0, Math.min(asked, tall - SEEN)) : asked);
    },
  });
  return () => {
    for (const [name, one] of Object.entries(kept)) {
      if (one) Object.defineProperty(proto, name, one);
      else delete (proto as unknown as Record<string, unknown>)[name];
    }
  };
};

let restore = () => {};

afterEach(() => restore());

describe("volver de una pagina a su documento", () => {
  it("deja el documento donde estaba cuando la altura ya esta", async () => {
    const shown = render(
      <Docs open="a3f1-0001" known={known} onKept={vi.fn()} onError={vi.fn()} />,
    );
    await waitFor(() => expect(scroller()).not.toBeNull(), { timeout: 3000 });
    await screen.findByText(/Parrafo numero 3 /);

    const at = scroller();
    if (!at) throw new Error("sin scroller");
    at.scrollTop = 800;
    at.dispatchEvent(new Event("scroll", { bubbles: true }));

    shown.rerender(<Docs open="a3f1-0002" known={known} onKept={vi.fn()} onError={vi.fn()} />);
    await screen.findByText(/Su pagina/);

    shown.rerender(<Docs open="a3f1-0001" known={known} onKept={vi.fn()} onError={vi.fn()} />);
    await screen.findByText(/Parrafo numero 3 /);
    const back = scroller();
    if (!back) throw new Error("sin scroller al volver");

    await waitFor(() => expect(back.scrollTop).toBe(800), { timeout: 3000 });
  });

  it("lo deja donde estaba aunque la lista de documentos se rehaga por el camino", async () => {
    const again = () => known.map((one) => ({ ...one }));
    const shown = render(
      <Docs open="a3f1-0001" known={again()} onKept={vi.fn()} onError={vi.fn()} />,
    );
    await waitFor(() => expect(scroller()).not.toBeNull(), { timeout: 3000 });
    await screen.findByText(/Parrafo numero 3 /);

    const at = scroller();
    if (!at) throw new Error("sin scroller");
    at.scrollTop = 800;
    at.dispatchEvent(new Event("scroll", { bubbles: true }));

    shown.rerender(<Docs open="a3f1-0002" known={again()} onKept={vi.fn()} onError={vi.fn()} />);
    await screen.findByText(/Su pagina/);

    shown.rerender(<Docs open="a3f1-0001" known={again()} onKept={vi.fn()} onError={vi.fn()} />);
    await screen.findByText(/Parrafo numero 3 /);
    const back = scroller();
    if (!back) throw new Error("sin scroller al volver");

    await waitFor(() => expect(back.scrollTop).toBe(800), { timeout: 3000 });
  });

  it("da el teclado a la hoja, nunca al texto, para que nada arrastre la vista", async () => {
    render(<Docs open="a3f1-0001" known={known} onKept={vi.fn()} onError={vi.fn()} />);
    await waitFor(() => expect(scroller()).not.toBeNull(), { timeout: 3000 });
    await screen.findByText(/Parrafo numero 3 /);

    const at = scroller();
    if (!at) throw new Error("sin scroller");

    expect(at.getAttribute("tabindex")).toBe("0");
    expect(document.activeElement).toBe(at);
    expect(at.querySelector(".ProseMirror")).not.toBe(document.activeElement);
  });

  it("nombra en el texto la pagina a la que se puede volver", async () => {
    render(<Docs open="a3f1-0001" known={known} onKept={vi.fn()} onError={vi.fn()} />);
    await waitFor(() => expect(scroller()).not.toBeNull(), { timeout: 3000 });

    await waitFor(() => expect(document.querySelector('[data-doc="a3f1-0002"]')).not.toBeNull());
  });

  it("no toma por tuya la posicion que deja el documento al vaciarse", async () => {
    const shown = render(
      <Docs open="a3f1-0001" known={known} onKept={vi.fn()} onError={vi.fn()} />,
    );
    await waitFor(() => expect(scroller()).not.toBeNull(), { timeout: 3000 });
    await screen.findByText(/Parrafo numero 3 /);

    const at = scroller();
    if (!at) throw new Error("sin scroller");
    at.scrollTop = 800;
    at.dispatchEvent(new Event("scroll", { bubbles: true }));

    tall = SEEN;
    at.scrollTop = 0;
    at.dispatchEvent(new Event("scroll", { bubbles: true }));
    tall = 4000;

    shown.rerender(<Docs open="a3f1-0002" known={known} onKept={vi.fn()} onError={vi.fn()} />);
    await screen.findByText(/Su pagina/);

    shown.rerender(<Docs open="a3f1-0001" known={known} onKept={vi.fn()} onError={vi.fn()} />);
    await screen.findByText(/Parrafo numero 3 /);
    const back = scroller();
    if (!back) throw new Error("sin scroller al volver");

    await waitFor(() => expect(back.scrollTop).toBe(800), { timeout: 3000 });
  });

  it("lo deja donde estaba tambien cuando la altura tarda en llegar", async () => {
    const shown = render(
      <Docs open="a3f1-0001" known={known} onKept={vi.fn()} onError={vi.fn()} />,
    );
    await waitFor(() => expect(scroller()).not.toBeNull(), { timeout: 3000 });
    await screen.findByText(/Parrafo numero 3 /);

    const at = scroller();
    if (!at) throw new Error("sin scroller");
    at.scrollTop = 800;
    at.dispatchEvent(new Event("scroll", { bubbles: true }));

    shown.rerender(<Docs open="a3f1-0002" known={known} onKept={vi.fn()} onError={vi.fn()} />);
    await screen.findByText(/Su pagina/);

    tall = SEEN;
    shown.rerender(<Docs open="a3f1-0001" known={known} onKept={vi.fn()} onError={vi.fn()} />);
    await screen.findByText(/Parrafo numero 3 /);
    const back = scroller();
    if (!back) throw new Error("sin scroller al volver");

    await waitFor(() => expect(back.scrollTop).toBe(0));
    tall = 4000;
    grew();

    await waitFor(() => expect(back.scrollTop).toBe(800), { timeout: 3000 });
  });
});
