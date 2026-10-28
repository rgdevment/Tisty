import { fireEvent, render, screen, within } from "@testing-library/react";
import { sayIn } from "tisty-diagrams/read";
import { beforeAll, beforeEach, describe, expect, it, vi } from "vitest";
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

vi.mock("../ui/diagrams", () => ({
  drawing: () => Promise.resolve('<svg viewBox="0 0 10 10"><text>mermaid</text></svg>'),
}));

// React Flow measures and moves with what a browser has and jsdom does not.
class Watcher {
  observe() {}
  unobserve() {}
  disconnect() {}
}
class Matrix {
  m22 = 1;
  constructor(readonly transform = "") {}
}

beforeAll(() => {
  vi.stubGlobal("ResizeObserver", Watcher);
  vi.stubGlobal("DOMMatrixReadOnly", Matrix);
});

const FLOW =
  "flowchart LR\n  A[Inicio] --> B{¿Pago?}\n  B -->|sí| C(Cobro)\n  subgraph g1 [Caja]\n    C\n  end\n  C --> D --> E\n";

const paper = { id: "D1", file: "f1", title: "Pagos", place: "" };
const opened = (source = FLOW) => render(<Diagramming paper={paper} source={source} />);
const text = () => (screen.getByLabelText(say("diagramText")) as HTMLTextAreaElement).value;
const node = (id: string) => document.querySelector(`[data-node="${id}"]`) as HTMLElement;
const pickNode = (id: string) => fireEvent.click(within(node(id)).getByRole("button"));
const palette = () => within(screen.getByRole("complementary", { name: say("flowPalette") }));
const canvas = () => document.querySelector(".react-flow") as HTMLElement;
const leave = (label: string, value: string) => {
  const field = screen.getByLabelText(label) as HTMLInputElement;
  fireEvent.change(field, { target: { value } });
  fireEvent.blur(field);
};
const typed = (field: HTMLElement, value: string) => {
  fireEvent.change(field, { target: { value } });
  fireEvent.keyDown(field, { key: "Enter" });
};
const choose = (label: string, value: string) =>
  fireEvent.change(screen.getByLabelText(label), { target: { value } });

beforeEach(() => {
  vi.restoreAllMocks();
});

describe("a flowchart in the diagram window", () => {
  it("draws every node, arrow and group itself, beside the palette, and asks for a choice", () => {
    opened();

    expect(document.querySelectorAll("[data-node]")).toHaveLength(5);
    expect(document.querySelectorAll(".react-flow__edge")).toHaveLength(4);
    expect(screen.getByRole("button", { name: "Caja" })).toBeTruthy();
    expect(screen.getByText(say("flowPick"))).toBeTruthy();
    expect(palette().getByRole("button", { name: say("flowShapePerson") })).toBeTruthy();
  });

  it("renames and reshapes a node from the panel, where its words are written", () => {
    opened();
    pickNode("B");

    leave(say("flowText"), "¿Pagó?");
    choose(say("flowShape"), "hexagon");

    expect(text()).toBe(FLOW.replace("B{¿Pago?}", "B{{¿Pagó?}}"));
  });

  it("renames a node in place with a double click, and Escape leaves it as it was", () => {
    opened();

    fireEvent.doubleClick(within(node("A")).getByRole("button"));
    typed(within(node("A")).getByRole("textbox"), "Comienzo");
    expect(text()).toBe(FLOW.replace("A[Inicio]", "A[Comienzo]"));

    fireEvent.doubleClick(within(node("A")).getByRole("button"));
    fireEvent.keyDown(within(node("A")).getByRole("textbox"), { key: "Escape" });
    expect(text()).toBe(FLOW.replace("A[Inicio]", "A[Comienzo]"));
  });

  it("puts a shape from the palette in the middle, writes where everything stands, and names it", () => {
    opened();

    fireEvent.click(palette().getByRole("button", { name: say("flowShapePerson") }));
    typed(within(node("n1")).getByRole("textbox"), "Cliente");

    expect(text()).toContain('  n1@{ shape: person, label: "Cliente" }\n');
    for (const id of ["A", "B", "C", "D", "E", "n1"]) expect(text()).toMatch(`%% tisty: ${id} `);
  });

  it("drops a shape dragged from the palette onto the canvas, and nowhere else", () => {
    opened();
    const cloud = palette().getByRole("button", { name: say("flowShapeCloud") });
    const carry = (under: Element | null) => {
      document.elementFromPoint = () => under;
      fireEvent.pointerDown(cloud, { clientX: 0, clientY: 0 });
      fireEvent.pointerMove(window, { clientX: 200, clientY: 120 });
      fireEvent.pointerUp(window, { clientX: 200, clientY: 120 });
      fireEvent.click(cloud);
    };

    carry(document.body);
    expect(text()).toBe(FLOW);

    carry(canvas());
    expect(text()).toContain(`  n1@{ shape: cloud, label: "${say("flowShapeCloud")}" }\n`);
    expect(text().match(/@\{/g)).toHaveLength(1);
  });

  it("writes where a shape is nudged with the arrow keys", () => {
    opened();
    pickNode("A");

    fireEvent.keyDown(node("A").closest(".react-flow__node") as HTMLElement, { key: "ArrowRight" });

    expect(text()).toMatch(/%% tisty: A -?\d+ -?\d+/);
    for (const id of ["B", "C", "D", "E"]) expect(text()).toMatch(`%% tisty: ${id} `);
  });

  it("makes a joined shape where an arrow is let go on nothing, and joins one let go on a shape", () => {
    opened();
    const pull = (from: string, under: HTMLElement | null, to = 300) => {
      document.elementFromPoint = () => under;
      const dot = node(from).querySelector(".react-flow__handle-right") as HTMLElement;
      fireEvent.mouseDown(dot, { clientX: 0, clientY: 0, button: 0 });
      fireEvent.mouseMove(document, { clientX: to, clientY: to });
      fireEvent.mouseUp(document, { clientX: to, clientY: to });
    };

    pull("E", null);
    typed(within(node("n1")).getByRole("textbox"), "Envío");
    expect(text()).toContain("  E --> n1[Envío]\n");

    pull("E", node("A"));
    expect(text()).toContain("  E --> A\n");

    const before = text();
    pull("D", node("D"), 900);
    expect(text()).toBe(before);
  });

  it("labels an arrow in place with a double click, and changes its kind from the panel", () => {
    opened();

    fireEvent.doubleClick(screen.getByRole("button", { name: "sí" }));
    typed(screen.getByRole("textbox", { name: say("flowLabel") }), "pagó");
    expect(text()).toBe(FLOW.replace("B -->|sí| C", "B -->|pagó| C"));

    fireEvent.doubleClick(document.querySelector(".react-flow__edge") as Element);
    typed(within(canvas()).getByRole("textbox", { name: say("flowLabel") }), "empieza");
    expect(text()).toContain("A[Inicio] -->|empieza| B");

    fireEvent.click(screen.getByRole("button", { name: "empieza" }));
    choose(say("flowArrow"), "-.->");
    expect(text()).toContain("A[Inicio] -.->|empieza| B");
  });

  it("leaves the text as written when a name is kept as it was", () => {
    const worded = FLOW.replace("B -->|sí| C", "B -- sí --> C").replace("A[Inicio]", 'A["Inicio"]');
    opened(worded);

    fireEvent.doubleClick(screen.getByRole("button", { name: "sí" }));
    typed(screen.getByRole("textbox", { name: say("flowLabel") }), "sí");
    fireEvent.doubleClick(within(node("A")).getByRole("button"));
    fireEvent.blur(within(node("A")).getByRole("textbox"));

    expect(text()).toBe(worded);
  });

  it("takes out what is chosen with Delete, but not while typing or on the canvas's buttons", () => {
    opened();
    fireEvent.click(screen.getByRole("button", { name: "A → B" }));

    fireEvent.keyDown(screen.getByLabelText(say("flowLabel")), { key: "Delete" });
    fireEvent.keyDown(screen.getByRole("button", { name: say("flowAddGroup") }), {
      key: "Backspace",
    });
    expect(text()).toBe(FLOW);

    fireEvent.keyDown(screen.getByRole("button", { name: "A → B" }), { key: "Delete" });
    expect(text()).toBe(FLOW.replace("  A[Inicio] --> B{¿Pago?}\n", "  A[Inicio]\n  B{¿Pago?}\n"));

    pickNode("E");
    fireEvent.keyDown(node("E"), { key: "Backspace" });
    expect(text()).toBe(FLOW.replace("  A[Inicio] --> B{¿Pago?}\n", "  A[Inicio]\n  B{¿Pago?}\n"));
  });

  it("will not take out one link of a chain, and says why", () => {
    opened();
    fireEvent.click(screen.getByRole("button", { name: "C → D" }));

    const remove = screen.getByRole("button", { name: say("flowRemoveEdge") }) as HTMLButtonElement;
    expect(remove.disabled).toBe(true);
    expect(remove.title).toBe(say("flowChained"));
  });

  it("adds a group, renames it, moves a node into it, and takes it apart", () => {
    opened();
    fireEvent.click(screen.getByRole("button", { name: say("flowAddGroup") }));
    expect(text()).toContain(`  subgraph g2 [${say("flowNewGroup")}]\n  end`);

    fireEvent.click(screen.getByRole("button", { name: "Caja" }));
    leave(say("flowText"), "Caja fuerte");
    expect(text()).toContain("[Caja fuerte]");

    pickNode("A");
    choose(say("flowGroup"), "g1");
    expect(text()).toContain("    C\n    A\n  end");

    fireEvent.click(screen.getByRole("button", { name: "Caja fuerte" }));
    fireEvent.click(screen.getByRole("button", { name: say("flowUngroup") }));
    expect(text()).not.toContain("[Caja fuerte]");
  });

  it("forgets a choice when the empty canvas is pressed", () => {
    opened();
    pickNode("A");

    fireEvent.click(document.querySelector(".react-flow__pane") as HTMLElement);

    expect(screen.getByText(say("flowPick"))).toBeTruthy();
  });

  it("turns the chart another way from the panel", () => {
    opened();

    choose(say("flowDirection"), "TB");

    expect(text()).toBe(FLOW.replace("flowchart LR", "flowchart TB"));
  });

  it("leaves as Mermaid draws it a flowchart it cannot keep", () => {
    opened("flowchart LR\n  A --> B\n  subgraph g\n    A\n");

    expect(document.querySelector("[data-node]")).toBeNull();
    expect(screen.queryByText(say("flowPick"))).toBeNull();
  });
});
