import { cleanup, render, screen, waitFor } from "@solidjs/testing-library";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import App from "../src/App";
import type { Opened } from "../src/host";

const host = vi.hoisted(() => ({
  opened: vi.fn(),
  theRest: vi.fn(),
  source: vi.fn(),
  facts: vi.fn(),
  editCopy: vi.fn(),
  editAsked: vi.fn(),
  shown: vi.fn(),
  openLink: vi.fn(),
  reveal: vi.fn(),
  themed: vi.fn(),
  pictureNear: vi.fn(),
  pictureFar: vi.fn(),
  follow: vi.fn(),
}));
const drawing = vi.hoisted(() => ({ diagrams: vi.fn(), readAs: vi.fn() }));

vi.mock("../src/host", () => host);
vi.mock("../src/drawing", () => drawing);

const PATH = "C:/Descargas/notas.md";
const jumped: string[] = [];

const drawn = (first = '<h2 id="one">One</h2><h2 id="two">Two</h2><p>all</p>'): Opened => ({
  kind: "drawn",
  name: "notas.md",
  place: "Descargas",
  path: PATH,
  first,
  more: false,
  heads: [
    { level: 2, said: "One", id: "one" },
    { level: 2, said: "Two", id: "two" },
  ],
});

const button = (name: string | RegExp) => screen.findByRole("button", { name });
const menuItem = (name: RegExp) => screen.findByRole("menuitem", { name });
const pressed = (init: KeyboardEventInit) =>
  window.dispatchEvent(new KeyboardEvent("keydown", init));

const open = async (one: Opened = drawn()) => {
  host.opened.mockResolvedValue(one);
  const close = vi.fn();
  render(() => <App asked="es" close={close} />);
  await waitFor(() => expect(host.shown).toHaveBeenCalled());
  return close;
};

const page = () => document.querySelector(".page") as HTMLElement;

const placedAt = (id: string, top: number) => {
  (document.getElementById(id) as HTMLElement).getBoundingClientRect = () => ({ top }) as DOMRect;
};

beforeEach(() => {
  vi.clearAllMocks();
  localStorage.clear();
  sessionStorage.clear();
  host.facts.mockResolvedValue({ wrote: 1_700_000_000, bytes: 3 * 1024 * 1024 });
  host.editAsked.mockResolvedValue(() => {});
  host.themed.mockResolvedValue(undefined);
  host.shown.mockResolvedValue(undefined);
  host.theRest.mockResolvedValue([]);
  host.reveal.mockResolvedValue(undefined);
  host.follow.mockResolvedValue(undefined);
  drawing.diagrams.mockResolvedValue(undefined);
  drawing.readAs.mockReturnValue({ kept: [], drawn: Promise.resolve() });
  window.print = vi.fn();
  jumped.length = 0;
  Element.prototype.scrollIntoView = function (this: Element) {
    jumped.push(this.id);
  };
});

afterEach(() => {
  cleanup();
  document.documentElement.classList.remove("dark");
  delete document.documentElement.dataset.theme;
  vi.unstubAllGlobals();
});

describe("the menu", () => {
  it("sizes the text each way and back, within its limits", async () => {
    await open();
    const size = async (name: RegExp) => {
      (await button("Más")).click();
      (await menuItem(name)).click();
      return page().style.getPropertyValue("zoom");
    };
    expect(await size(/más grande/)).toBe("1.1");
    expect(await size(/normal/)).toBe("1");
    expect(await size(/más pequeño/)).toBe("0.9");
    for (let at = 0; at < 8; at++) pressed({ key: "-", ctrlKey: true });
    await waitFor(() => expect(page().style.getPropertyValue("zoom")).toBe("0.8"));
  });

  it("copies what was written", async () => {
    const writeText = vi.fn().mockResolvedValue(undefined);
    vi.stubGlobal("navigator", { clipboard: { writeText } });
    host.source.mockResolvedValue("# written");
    await open();
    (await button("Más")).click();
    (await menuItem(/Copiar/)).click();
    await waitFor(() => expect(writeText).toHaveBeenCalledWith("# written"));
  });

  it("shows the folder, and says so when it cannot", async () => {
    await open();
    (await button("Más")).click();
    (await menuItem(/carpeta/)).click();
    await waitFor(() => expect(host.reveal).toHaveBeenCalledWith(PATH));
    host.reveal.mockRejectedValue({ code: "internal" });
    (await button("Más")).click();
    (await menuItem(/carpeta/)).click();
    expect((await screen.findByRole("alert")).textContent).toContain("No se pudo hacer");
  });

  it("prints, and finds from itself", async () => {
    await open();
    (await button("Más")).click();
    (await menuItem(/Imprimir/)).click();
    await waitFor(() => expect(window.print).toHaveBeenCalled());
    (await button("Más")).click();
    (await menuItem(/Buscar/)).click();
    expect(await screen.findByRole("searchbox")).toBeTruthy();
  });

  it("goes away when pressed outside, and stays when pressed inside", async () => {
    await open();
    (await button("Más")).click();
    const menu = await screen.findByRole("menu");
    menu.dispatchEvent(new MouseEvent("pointerdown", { bubbles: true }));
    expect(screen.queryByRole("menu")).not.toBeNull();
    page().dispatchEvent(new MouseEvent("pointerdown", { bubbles: true }));
    await waitFor(() => expect(screen.queryByRole("menu")).toBeNull());
  });
});

describe("Escape", () => {
  it("leaves what is on top, one thing at a time", async () => {
    host.source.mockResolvedValue("# written");
    await open();
    (await button("Ver Markdown")).click();
    await waitFor(() => expect(document.querySelector(".written")).not.toBeNull());
    pressed({ key: "f", ctrlKey: true });
    await screen.findByRole("searchbox");
    (await button("Más")).click();
    await screen.findByRole("menu");

    pressed({ key: "Escape" });
    await waitFor(() => expect(screen.queryByRole("menu")).toBeNull());
    expect(screen.queryByRole("searchbox")).not.toBeNull();
    pressed({ key: "Escape" });
    await waitFor(() => expect(screen.queryByRole("searchbox")).toBeNull());
    expect(document.querySelector(".written")).not.toBeNull();
    pressed({ key: "Escape" });
    await waitFor(() => expect(document.querySelector(".written")).toBeNull());
  });

  it("folds the panels that came over the page", async () => {
    await open();
    (await button("Índice y detalles")).click();
    await waitFor(() =>
      expect(document.querySelector(".panels")?.hasAttribute("data-open")).toBe(true),
    );
    pressed({ key: "Escape" });
    await waitFor(() =>
      expect(document.querySelector(".panels")?.hasAttribute("data-open")).toBe(false),
    );
  });
});

describe("the bar", () => {
  it("leaves the window's buttons to the system, and still closes by keys", async () => {
    const close = await open();
    await button("Editar");
    expect(screen.queryByRole("button", { name: "Minimizar" })).toBeNull();
    expect(screen.queryByRole("button", { name: "Maximizar" })).toBeNull();
    expect(screen.queryByRole("button", { name: "Cerrar" })).toBeNull();
    pressed({ key: "w", ctrlKey: true });
    expect(close).toHaveBeenCalled();
  });

  it("changes the theme, remembers it and draws the diagrams again once the page is whole", async () => {
    await open();
    await waitFor(() => expect(drawing.readAs).toHaveBeenCalled());
    (await button("Cambiar el tema")).click();
    expect(document.documentElement.dataset.theme).toBe("dark");
    expect(localStorage.getItem("tisty.viewer.theme")).toBe("dark");
    expect(host.themed).toHaveBeenLastCalledWith(true);
    await waitFor(() => expect(drawing.diagrams).toHaveBeenCalled());
    pressed({ key: "L", ctrlKey: true, shiftKey: true });
    expect(document.documentElement.dataset.theme).toBe("light");
  });

  it("offers the way back only when there is one", async () => {
    await open();
    await button("Editar");
    expect(screen.queryByRole("button", { name: "Volver" })).toBeNull();
    cleanup();
    vi.stubGlobal("location", { reload: vi.fn(), search: "" });
    sessionStorage.setItem("tisty.viewer.behind", JSON.stringify(["C:/antes.md"]));
    await open();
    (await button("Volver")).click();
    await waitFor(() => expect(host.follow).toHaveBeenCalledWith("C:/antes.md"));
    await waitFor(() => expect(location.reload).toHaveBeenCalled());
  });
});

describe("the index", () => {
  it("jumps at once, leaving what was written if it was showing", async () => {
    host.source.mockResolvedValue("# written");
    await open();
    (await button("Ver Markdown")).click();
    await waitFor(() => expect(document.querySelector(".written")).not.toBeNull());
    (await button("Two")).click();
    await waitFor(() => expect(document.querySelector(".written")).toBeNull());
    await waitFor(() => expect(jumped).toContain("two"));
  });

  it("marks the section being read as the page scrolls", async () => {
    await open();
    placedAt("one", -100);
    placedAt("two", 50);
    window.dispatchEvent(new Event("scroll"));
    window.dispatchEvent(new Event("scroll"));
    const two = await button("Two");
    await waitFor(() => expect(two.getAttribute("aria-current")).toBe("location"));
    placedAt("two", 5000);
    window.dispatchEvent(new Event("scroll"));
    await waitFor(() => expect(two.getAttribute("aria-current")).toBeNull());
  });

  it("marks the last section at the very end of the page", async () => {
    await open();
    placedAt("one", 5000);
    placedAt("two", 5000);
    vi.spyOn(window, "scrollY", "get").mockReturnValue(100);
    Object.defineProperty(document.documentElement, "scrollHeight", {
      configurable: true,
      value: 100,
    });
    window.dispatchEvent(new Event("scroll"));
    const two = await button("Two");
    await waitFor(() => expect(two.getAttribute("aria-current")).toBe("location"));
    delete (document.documentElement as unknown as Record<string, unknown>).scrollHeight;
    vi.restoreAllMocks();
  });

  it("keeps the section being read in sight inside its own list", async () => {
    await open();
    const list = document.querySelector(".index") as HTMLElement;
    Object.defineProperty(list, "clientHeight", { configurable: true, value: 100 });
    Object.defineProperty(list.children[1], "offsetTop", { configurable: true, value: 300 });
    placedAt("one", -500);
    placedAt("two", 0);
    window.dispatchEvent(new Event("scroll"));
    await waitFor(() => expect(list.scrollTop).toBe(250));
    list.scrollTop = 40;
    window.dispatchEvent(new Event("scroll"));
    await new Promise((done) => requestAnimationFrame(done));
    expect(list.scrollTop).toBe(40);
  });

  it("goes from heading to heading by keys", async () => {
    await open();
    await waitFor(() => expect(drawing.readAs).toHaveBeenCalled());
    placedAt("one", 40);
    placedAt("two", 500);
    pressed({ key: "ArrowDown", altKey: true });
    expect(jumped).toContain("two");
    pressed({ key: "ArrowUp", altKey: true });
    expect(jumped).toContain("one");
  });
});

describe("the height of the page", () => {
  it("stops being a guess for every block, without blocking on a long page", async () => {
    const blocks = Array.from({ length: 450 }, (_, at) => `<p>${at}</p>`).join("");
    await open(drawn(blocks));
    await waitFor(() => {
      const all = Array.from(page().children) as HTMLElement[];
      expect(all).toHaveLength(450);
      expect(all.every((one) => one.style.contentVisibility === "visible")).toBe(true);
    });
  });
});

describe("the details", () => {
  it("say the folder, the size, the date and the file's own properties", async () => {
    await open(
      drawn(
        '<dl class="front"><div><dt>title</dt><dd>Mi nota</dd></div></dl><h2 id="one">One</h2>',
      ),
    );
    expect(await screen.findByText("3.0 MB")).toBeTruthy();
    expect(screen.getByText("Descargas", { selector: "dd" })).toBeTruthy();
    expect(screen.getByText("Mi nota", { selector: ".facts dd" })).toBeTruthy();
    expect(screen.getByText(new Date(1_700_000_000_000).toLocaleDateString("es"))).toBeTruthy();
  });

  it("say a small file's size in its own unit", async () => {
    host.facts.mockResolvedValue({ wrote: null, bytes: 512 });
    await open();
    expect(await screen.findByText("512 B")).toBeTruthy();
    host.facts.mockResolvedValue({ wrote: null, bytes: 2048 });
    cleanup();
    await open();
    expect(await screen.findByText("2 KB")).toBeTruthy();
  });
});

describe("coming back to a file", () => {
  it("returns to the block it was left at", async () => {
    localStorage.setItem("tisty.viewer.places", JSON.stringify({ [PATH]: 1 }));
    await open();
    await waitFor(() => expect(jumped).toContain("two"));
  });

  it("goes to the anchor a link asked for before the place it was left at", async () => {
    localStorage.setItem("tisty.viewer.places", JSON.stringify({ [PATH]: 1 }));
    sessionStorage.setItem("tisty.viewer.arriving", "one");
    await open();
    await waitFor(() => expect(jumped).toContain("one"));
    expect(jumped).not.toContain("two");
  });

  it("writes down where the reading is, but not while the source is showing", async () => {
    host.source.mockResolvedValue("# written");
    await open();
    await waitFor(() => expect(drawing.readAs).toHaveBeenCalled());
    (document.querySelector("#two + p") as HTMLElement).getBoundingClientRect = () =>
      ({ bottom: 500 }) as DOMRect;
    placedAt("one", -50);
    (document.getElementById("one") as HTMLElement).getBoundingClientRect = () =>
      ({ bottom: 10 }) as DOMRect;
    (document.getElementById("two") as HTMLElement).getBoundingClientRect = () =>
      ({ bottom: 200 }) as DOMRect;
    window.dispatchEvent(new Event("scroll"));
    await waitFor(
      () =>
        expect(JSON.parse(localStorage.getItem("tisty.viewer.places") ?? "{}")).toEqual({
          [PATH]: 1,
        }),
      { timeout: 1500 },
    );
    (await button("Ver Markdown")).click();
    await waitFor(() => expect(document.querySelector(".written")).not.toBeNull());
    localStorage.clear();
    window.dispatchEvent(new Event("scroll"));
    await new Promise((done) => setTimeout(done, 400));
    expect(localStorage.getItem("tisty.viewer.places")).toBeNull();
  });
});

describe("a file that did not open", () => {
  const missed: Opened = {
    kind: "missed",
    name: "notas.md",
    place: "Descargas",
    path: PATH,
    refusal: { code: "cannotRead" },
  };

  it("says so, shows its folder and tries again", async () => {
    const reload = vi.fn();
    vi.stubGlobal("location", { reload, search: "" });
    await open(missed);
    expect(await screen.findByText(/No se pudo leer/)).toBeTruthy();
    (await button("Mostrar en la carpeta")).click();
    await waitFor(() => expect(host.reveal).toHaveBeenCalledWith(PATH));
    (await button("Intentar de nuevo")).click();
    expect(reload).toHaveBeenCalled();
  });
});

describe("searching", () => {
  it("steps through what it found, around and back", async () => {
    const painted = new Map<string, unknown>();
    vi.stubGlobal("CSS", { highlights: painted });
    vi.stubGlobal(
      "Highlight",
      class {
        ranges: unknown[];
        constructor(...ranges: unknown[]) {
          this.ranges = ranges;
        }
      },
    );
    await open(drawn("<p>dos y dos</p><p>tres</p>"));
    pressed({ key: "f", ctrlKey: true });
    const field = (await screen.findByRole("searchbox")) as HTMLInputElement;
    const typed = (said: string) => {
      field.value = said;
      field.dispatchEvent(new InputEvent("input", { bubbles: true }));
    };
    const count = () => screen.findByRole("status");

    typed("dos");
    await waitFor(async () => expect((await count()).textContent).toBe("1 / 2"));
    expect((painted.get("tisty-found") as { ranges: unknown[] }).ranges).toHaveLength(2);
    (await button("Siguiente")).click();
    await waitFor(async () => expect((await count()).textContent).toBe("2 / 2"));
    field.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));
    await waitFor(async () => expect((await count()).textContent).toBe("1 / 2"));
    (await button("Anterior")).click();
    await waitFor(async () => expect((await count()).textContent).toBe("2 / 2"));
    field.dispatchEvent(
      new KeyboardEvent("keydown", { key: "Enter", shiftKey: true, bubbles: true }),
    );
    await waitFor(async () => expect((await count()).textContent).toBe("1 / 2"));

    typed("nada");
    await waitFor(async () => expect((await count()).textContent).toBe("Sin resultados"));
    (await button("Siguiente")).click();
    typed("");
    await waitFor(() => expect(screen.queryByRole("status")).toBeNull());
    (document.querySelector('.finder [aria-label="Cerrar"]') as HTMLElement).click();
    await waitFor(() => expect(screen.queryByRole("searchbox")).toBeNull());
    expect(painted.size).toBe(0);
  });
});
