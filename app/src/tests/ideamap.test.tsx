import { fireEvent, render, screen, within } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { t } from "../locales";
import Diagramming from "../ui/drawing/Diagramming";

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

vi.mock("../ui/diagrams", () => ({
  drawing: () => Promise.resolve('<svg viewBox="0 0 10 10"><text>mermaid</text></svg>'),
}));

const MAP = "mindmap\n  root((Share))\n    Cifrado\n      Frase\n    Servidor\n";
const paper = { id: "D1", file: "f1", title: "Ideas", place: "" };

const opened = (source = MAP) => render(<Diagramming paper={paper} source={source} />);
const text = () => (screen.getByLabelText(t("diagramText")) as HTMLTextAreaElement).value;
const idea = (words: string) => screen.getByRole("button", { name: words });
const chosenWords = () => (screen.getByLabelText(t("diagramIdea")) as HTMLInputElement).value;
const key = (on: Element, which: string) => fireEvent.keyDown(on, { key: which });

beforeEach(() => {
  vi.restoreAllMocks();
});

describe("a mind map in the diagram window", () => {
  it("draws every idea itself and starts on the root", () => {
    const { container } = opened();

    expect(container.querySelectorAll("[data-idea]")).toHaveLength(4);
    expect(idea("Frase")).toBeTruthy();
    expect(chosenWords()).toBe("Share");
    expect(screen.getByText(t("diagramMindKeys"))).toBeTruthy();
  });

  it("chooses an idea by pressing it and renames it from the panel, into the text", () => {
    opened();

    fireEvent.click(idea("Cifrado"));
    expect(chosenWords()).toBe("Cifrado");
    fireEvent.change(screen.getByLabelText(t("diagramIdea")), {
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
    expect(box.value).toBe(t("diagramNewIdea"));
    fireEvent.change(box, { target: { value: "Visor" } });
    key(box, "Enter");
    expect(text()).toBe(
      "mindmap\n  root((Share))\n    Cifrado\n      Frase\n    Visor\n    Servidor\n",
    );

    key(idea("Visor"), "Tab");
    const inner = document.querySelector(".diagram-drawn input") as HTMLInputElement;
    key(inner, "Escape");
    expect(text()).toBe(
      `mindmap\n  root((Share))\n    Cifrado\n      Frase\n    Visor\n      ${t("diagramNewIdea")}\n    Servidor\n`,
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

    const shapes = screen.getByText(t("diagramShape")).nextElementSibling as HTMLElement;
    fireEvent.click(within(shapes).getByRole("button", { name: t("diagramShapeHexagon") }));

    expect(text()).toBe("mindmap\n  root((Share))\n    Cifrado\n      Frase\n    {{Servidor}}\n");
    expect(
      within(shapes)
        .getByRole("button", { name: t("diagramShapeHexagon") })
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

    key(screen.getByRole("button", { name: t("diagramZoomIn") }), "Enter");
    key(screen.getByRole("button", { name: t("diagramFit") }), "Delete");

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
    const field = screen.getByLabelText(t("diagramIdea")) as HTMLInputElement;

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

    fireEvent.change(screen.getByLabelText(t("diagramText")), {
      target: { value: "mindmap\n  root((Share))\n    Nueva\n    Cifrado\n    Servidor\n" },
    });

    expect(chosenWords()).toBe("Servidor");
  });

  it("follows the text typed by hand, and leaves as Mermaid draws it a map it cannot keep", () => {
    const { container } = opened();
    const typed = screen.getByLabelText(t("diagramText"));

    fireEvent.change(typed, { target: { value: "mindmap\n  root((Share))\n    Solo una" } });
    expect(idea("Solo una")).toBeTruthy();

    fireEvent.change(typed, { target: { value: "mindmap\n  Una\n Otra raíz" } });
    expect(container.querySelector("[data-idea]")).toBeNull();
    expect(screen.queryByLabelText(t("diagramIdea"))).toBeNull();
  });
});
