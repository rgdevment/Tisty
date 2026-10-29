import { fireEvent, render, screen, within } from "@testing-library/react";
import { sayIn } from "tisty-diagrams/read";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import Diagramming from "../ui/drawing/Diagramming";

// The studio says its own words; these tests read them as it does, in English.
const say = sayIn("en");

vi.mock("@tauri-apps/api/core", () => ({
  invoke: () => Promise.resolve(null),
  convertFileSrc: (path: string) => path,
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({ ask: () => Promise.resolve(true) }));

vi.mock("@tauri-apps/api/window", () => ({
  getCurrentWindow: () => ({
    label: "view-0",
    minimize: () => Promise.resolve(),
    toggleMaximize: () => Promise.resolve(),
    close: () => Promise.resolve(),
    onCloseRequested: () => Promise.resolve(() => {}),
    listen: () => Promise.resolve(() => {}),
  }),
}));

vi.mock("tisty-reader/page", async (real) => ({
  ...(await real<typeof import("tisty-reader/page")>()),
  drawing: () => Promise.resolve('<svg viewBox="0 0 10 10"><text>mermaid</text></svg>'),
}));

const MAP = "mindmap\n  root((Share))\n    Cifrado\n      Frase\n    Servidor\n";
const paper = { id: "D1", file: "f1", title: "Ideas", place: "" };

const opened = (source = MAP) => render(<Diagramming paper={paper} source={source} />);
const text = () => (screen.getByLabelText(say("diagramText")) as HTMLTextAreaElement).value;
const idea = (words: string) => screen.getByRole("button", { name: words });
const chosenWords = () => (screen.getByLabelText(say("diagramIdea")) as HTMLInputElement).value;
const key = (on: Element, which: string) => fireEvent.keyDown(on, { key: which });
const palette = () => within(screen.getByRole("complementary", { name: say("flowPalette") }));
const tool = (name: string) =>
  within(screen.getByRole("toolbar", { name: say("flowTools") })).getByRole("button", {
    name,
  }) as HTMLButtonElement;
const named = (words: string) => {
  const box = document.querySelector(".diagram-drawn input") as HTMLInputElement;
  fireEvent.change(box, { target: { value: words } });
  key(box, "Enter");
};

// Tests stub where the pointer lands; each starts from the page's own answer.
const pointed = document.elementFromPoint;

beforeEach(() => {
  vi.restoreAllMocks();
});

afterEach(() => {
  document.elementFromPoint = pointed;
});

describe("a mind map in the diagram window", () => {
  it("draws every idea itself and starts on the root", () => {
    const { container } = opened();

    expect(container.querySelectorAll("[data-idea]")).toHaveLength(4);
    expect(idea("Frase")).toBeTruthy();
    expect(chosenWords()).toBe("Share");
    expect(tool(say("flowUndo")).disabled).toBe(true);
    expect(tool(say("flowDelete")).disabled).toBe(true);
  });

  it("puts an idea of the shape pressed in the palette under the chosen one, named at once", () => {
    opened();
    fireEvent.click(idea("Servidor"));

    fireEvent.click(palette().getByRole("button", { name: say("diagramShapeCloud") }));
    named("Nube");

    expect(text()).toBe(
      "mindmap\n  root((Share))\n    Cifrado\n      Frase\n    Servidor\n      )Nube(\n",
    );
  });

  it("puts an idea dragged from the palette under the idea it is let go on", () => {
    opened();
    const shape = palette().getByRole("button", { name: say("diagramShapeSquare") });
    document.elementFromPoint = () => idea("Frase");

    fireEvent.pointerDown(shape, { clientX: 0, clientY: 0 });
    fireEvent.pointerMove(window, { clientX: 300, clientY: 200 });
    fireEvent.pointerUp(window, { clientX: 300, clientY: 200 });
    named("Clave");

    expect(text()).toBe(
      "mindmap\n  root((Share))\n    Cifrado\n      Frase\n        [Clave]\n    Servidor\n",
    );
  });

  it("grows, prunes and undoes from the toolbar and with Ctrl+Z", () => {
    opened();
    fireEvent.click(idea("Cifrado"));

    fireEvent.click(tool(say("diagramAddChild")));
    named("Hoja");
    expect(text()).toContain("      Frase\n      Hoja\n");
    fireEvent.click(tool(say("diagramAddSibling")));
    named("Rama");
    expect(text()).toContain("      Hoja\n      Rama\n");

    fireEvent.click(tool(say("flowDelete")));
    expect(text()).not.toContain("Rama");
    fireEvent.click(tool(say("flowUndo")));
    expect(text()).toContain("Rama");
    fireEvent.keyDown(idea("Rama"), { key: "z", ctrlKey: true });
    expect(text()).not.toContain("Rama");
    expect(text()).not.toContain(say("diagramNewIdea"));
    fireEvent.keyDown(idea("Hoja"), { key: "Z", ctrlKey: true, shiftKey: true });
    expect(text()).toContain("Rama");
    fireEvent.keyDown(idea("Hoja"), { key: "z", ctrlKey: true });
    fireEvent.keyDown(idea("Hoja"), { key: "y", ctrlKey: true });
    expect(text()).toContain("Rama");
  });

  it("leaves Ctrl+Z to the box while an idea is being named", () => {
    opened();
    fireEvent.click(palette().getByRole("button", { name: say("diagramShapeCloud") }));
    const box = document.querySelector(".diagram-drawn input") as HTMLInputElement;

    fireEvent.keyDown(box, { key: "z", ctrlKey: true });

    expect(text()).toContain(`)${say("diagramNewIdea")}(`);
  });

  it("puts nothing where a shape is let go over the toolbar or the palette", () => {
    opened();
    const shape = palette().getByRole("button", { name: say("diagramShapeSquare") });
    for (const under of [tool(say("flowZoomIn")), shape]) {
      document.elementFromPoint = () => under;
      fireEvent.pointerDown(shape, { clientX: 0, clientY: 0 });
      fireEvent.pointerMove(window, { clientX: 300, clientY: 200 });
      fireEvent.pointerUp(window, { clientX: 300, clientY: 200 });
    }

    expect(text()).toBe(MAP);
  });

  it("chooses an idea by pressing it and renames it from the panel, into the text", () => {
    opened();

    fireEvent.click(idea("Cifrado"));
    expect(chosenWords()).toBe("Cifrado");
    fireEvent.change(screen.getByLabelText(say("diagramIdea")), {
      target: { value: "Cifrado (24 palabras)" },
    });

    expect(text()).toBe(
      'mindmap\n  root((Share))\n    ("Cifrado (24 palabras)")\n      Frase\n    Servidor\n',
    );
  });

  it("adds a sibling with Enter and a child with Tab, named right away", () => {
    opened();
    fireEvent.click(idea("Cifrado"));

    key(idea("Cifrado"), "Enter");
    const box = document.querySelector(".diagram-drawn input") as HTMLInputElement;
    expect(box.value).toBe(say("diagramNewIdea"));
    fireEvent.change(box, { target: { value: "Visor" } });
    key(box, "Enter");
    expect(text()).toBe(
      "mindmap\n  root((Share))\n    Cifrado\n      Frase\n    Visor\n    Servidor\n",
    );

    key(idea("Visor"), "Tab");
    const inner = document.querySelector(".diagram-drawn input") as HTMLInputElement;
    key(inner, "Escape");
    expect(text()).toBe(
      `mindmap\n  root((Share))\n    Cifrado\n      Frase\n    Visor\n      ${say("diagramNewIdea")}\n    Servidor\n`,
    );
  });

  it("deletes the chosen idea with its branch, never the root", () => {
    opened();

    key(idea("Share"), "Delete");
    expect(text()).toBe(MAP);
    fireEvent.click(idea("Cifrado"));
    key(idea("Cifrado"), "Backspace");

    expect(text()).toBe("mindmap\n  root((Share))\n    Servidor\n");
    expect(chosenWords()).toBe("Share");
  });

  it("renames with a double press, and Escape leaves the words as they were", () => {
    opened();

    fireEvent.doubleClick(idea("Servidor"));
    const box = document.querySelector(".diagram-drawn input") as HTMLInputElement;
    fireEvent.change(box, { target: { value: "Worker" } });
    key(box, "Escape");
    expect(text()).toBe(MAP);

    fireEvent.doubleClick(idea("Servidor"));
    const again = document.querySelector(".diagram-drawn input") as HTMLInputElement;
    fireEvent.change(again, { target: { value: "Worker" } });
    fireEvent.blur(again);
    expect(text()).toBe("mindmap\n  root((Share))\n    Cifrado\n      Frase\n    Worker\n");
  });

  it("gives the chosen idea another shape", () => {
    opened();
    fireEvent.click(idea("Servidor"));

    const shapes = screen.getByText(say("diagramShape")).nextElementSibling as HTMLElement;
    fireEvent.click(within(shapes).getByRole("button", { name: say("diagramShapeHexagon") }));

    expect(text()).toBe("mindmap\n  root((Share))\n    Cifrado\n      Frase\n    {{Servidor}}\n");
    expect(
      within(shapes)
        .getByRole("button", { name: say("diagramShapeHexagon") })
        .getAttribute("aria-pressed"),
    ).toBe("true");
  });

  it("moves an idea under another by dragging it there", () => {
    opened();
    const from = idea("Frase");
    const onto = idea("Servidor");
    document.elementFromPoint = () => onto;

    fireEvent.pointerDown(from, { clientX: 0, clientY: 0, pointerId: 1 });
    fireEvent.pointerMove(from, { clientX: 30, clientY: 30, pointerId: 1, buttons: 1 });
    fireEvent.pointerUp(from, { clientX: 30, clientY: 30, pointerId: 1 });

    expect(text()).toBe("mindmap\n  root((Share))\n    Cifrado\n    Servidor\n      Frase\n");
  });

  it("keeps its keys to the ideas: Enter on a zoom button zooms, it adds nothing", () => {
    opened();

    key(tool(say("flowZoomIn")), "Enter");
    key(tool(say("flowFit")), "Delete");

    expect(text()).toBe(MAP);
  });

  it("gives the focus back to the chosen idea, so keys go on working idea after idea", () => {
    opened();
    fireEvent.click(idea("Servidor"));

    key(idea("Servidor"), "Delete");

    expect(document.activeElement?.textContent).toBe("Cifrado");
    expect(chosenWords()).toBe("Cifrado");
  });

  it("names a new idea over its placeholder, which comes chosen", () => {
    opened();

    key(idea("Share"), "Tab");
    const box = document.querySelector(".diagram-drawn input") as HTMLInputElement;

    expect(document.activeElement).toBe(box);
    expect([box.selectionStart, box.selectionEnd]).toEqual([0, box.value.length]);
  });

  it("lets the panel's field be emptied without writing an idea with no words", () => {
    opened();
    fireEvent.click(idea("Servidor"));
    const field = screen.getByLabelText(say("diagramIdea")) as HTMLInputElement;

    fireEvent.change(field, { target: { value: "" } });
    expect(field.value).toBe("");
    expect(text()).toBe(MAP);
    fireEvent.change(field, { target: { value: "Worker" } });
    expect(text()).toBe("mindmap\n  root((Share))\n    Cifrado\n      Frase\n    Worker\n");
  });

  it("keeps the idea it moved chosen, not the place it was dragged from", () => {
    opened();
    const from = idea("Frase");
    document.elementFromPoint = () => idea("Servidor");

    fireEvent.pointerDown(from, { clientX: 0, clientY: 0, pointerId: 1 });
    fireEvent.pointerMove(from, { clientX: 30, clientY: 30, pointerId: 1, buttons: 1 });
    fireEvent.pointerUp(from, { clientX: 30, clientY: 30, pointerId: 1 });
    fireEvent.click(from);

    expect(chosenWords()).toBe("Frase");
  });

  it("follows the chosen idea by its words when the text is changed by hand", () => {
    opened();
    fireEvent.click(idea("Servidor"));

    fireEvent.change(screen.getByLabelText(say("diagramText")), {
      target: { value: "mindmap\n  root((Share))\n    Nueva\n    Cifrado\n    Servidor\n" },
    });

    expect(chosenWords()).toBe("Servidor");
  });

  it("follows the text typed by hand, and leaves as Mermaid draws it a map it cannot keep", () => {
    const { container } = opened();
    const typed = screen.getByLabelText(say("diagramText"));

    fireEvent.change(typed, { target: { value: "mindmap\n  root((Share))\n    Solo una" } });
    expect(idea("Solo una")).toBeTruthy();

    fireEvent.change(typed, { target: { value: "mindmap\n  Una\n Otra raíz" } });
    expect(container.querySelector("[data-idea]")).toBeNull();
    expect(screen.queryByLabelText(say("diagramIdea"))).toBeNull();
  });
});
