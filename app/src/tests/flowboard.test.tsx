import { fireEvent, render, screen, within } from "@testing-library/react";
import { sayIn } from "tisty-diagrams/read";
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from "vitest";
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
const tool = (name: string) =>
  within(screen.getByRole("toolbar", { name: say("flowTools") })).getByRole("button", {
    name,
  }) as HTMLButtonElement;
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

// React Flow says what it dislikes only while developing; a test that listens turns that on.
afterEach(() => {
  vi.unstubAllEnvs();
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
    carry(screen.getByRole("toolbar", { name: say("flowTools") }));
    expect(text()).toBe(FLOW);

    carry(canvas());
    expect(text()).toContain(`  n1@{ shape: cloud, label: "${say("flowShapeCloud")}" }\n`);
    expect(text().match(/@\{/g)).toHaveLength(1);
  });

  it("writes where a shape is nudged with the arrow keys, a shape React Flow knows the size of", () => {
    vi.stubEnv("NODE_ENV", "development");
    const warned = vi.spyOn(console, "warn");
    opened();
    pickNode("A");

    fireEvent.keyDown(node("A").closest(".react-flow__node") as HTMLElement, { key: "ArrowRight" });

    expect(text()).toMatch(/%% tisty: A -?\d+ -?\d+/);
    for (const id of ["B", "C", "D", "E"]) expect(text()).toMatch(`%% tisty: ${id} `);
    expect(warned.mock.calls.flat().join(" ")).not.toContain("not initialized");
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
    fireEvent.keyDown(tool(say("flowRelaid")), { key: "Backspace" });
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

  it("adds a lane, renames it, moves a shape into it, and takes it apart", () => {
    opened();
    fireEvent.click(palette().getByRole("button", { name: say("flowLaneColumn") }));
    expect(text()).toContain(`  subgraph g2 [${say("flowNewLane")}]\n  end`);
    expect(text()).toMatch(/%% tisty: g2 [\d.-]+ [\d.-]+ [\d.]+ [\d.]+/);

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

  it("undoes and redoes what the window changed, by the toolbar and by the keys", () => {
    opened();
    const undo = tool(say("flowUndo"));
    expect(undo.disabled).toBe(true);

    fireEvent.click(palette().getByRole("button", { name: say("flowShapeCloud") }));
    fireEvent.keyDown(within(node("n1")).getByRole("textbox"), { key: "Escape" });
    expect(text()).toContain("n1@{");

    fireEvent.click(undo);
    expect(text()).toBe(FLOW);
    fireEvent.click(tool(say("flowRedo")));
    expect(text()).toContain("n1@{");
    fireEvent.keyDown(canvas(), { key: "z", ctrlKey: true });
    expect(text()).toBe(FLOW);
    fireEvent.keyDown(canvas(), { key: "z", ctrlKey: true, shiftKey: true });
    expect(text()).toContain("n1@{");

    fireEvent.change(screen.getByLabelText(say("diagramText")), { target: { value: FLOW } });
    expect(tool(say("flowUndo")).disabled).toBe(true);
  });

  it("deletes, duplicates and colours what is chosen from the toolbar", () => {
    opened();
    pickNode("A");

    fireEvent.click(tool(say("flowDuplicate")));
    expect(text()).toContain("  n1[Inicio]\n");
    fireEvent.keyDown(node("n1"), { key: "d", metaKey: true });
    expect(text()).toContain("  n2[Inicio]\n");

    fireEvent.click(tool(say("flowFill")));
    fireEvent.click(screen.getByRole("button", { name: say("colorGreen") }));
    expect(text()).toContain("  style n2 fill:#dcfce7,color:#14532d");
    fireEvent.click(tool(say("flowLine")));
    fireEvent.click(screen.getByRole("button", { name: say("colorRed") }));
    expect(text()).toContain("  style n2 fill:#dcfce7,color:#14532d,stroke:#dc2626");
    fireEvent.click(tool(say("flowFill")));
    fireEvent.click(screen.getByRole("button", { name: say("flowNoColor") }));
    expect(text()).toContain("  style n2 stroke:#dc2626");

    fireEvent.click(tool(say("flowDelete")));
    expect(text()).not.toContain("n2");
  });

  it("changes the kind of the chosen arrow from the toolbar, and says why a chain stays", () => {
    opened();
    fireEvent.click(screen.getByRole("button", { name: "A → B" }));

    fireEvent.click(tool(say("flowArrowThick")));
    expect(text()).toContain("A[Inicio] ==> B");
    expect(tool(say("flowArrowThick")).getAttribute("aria-pressed")).toBe("true");

    fireEvent.click(screen.getByRole("button", { name: "C → D" }));
    expect(tool(say("flowDelete")).disabled).toBe(true);
    expect(tool(say("flowDelete")).title).toBe(say("flowChained"));
  });

  it("chooses several shapes with Ctrl, lines them up and deletes them together", () => {
    opened();

    pickNode("D");
    fireEvent.keyDown(window, { key: "Control" });
    fireEvent.click(within(node("E")).getByRole("button"), { ctrlKey: true });
    fireEvent.keyUp(window, { key: "Control" });
    expect(screen.getByText(say("flowPickedMany", "2"))).toBeTruthy();

    fireEvent.click(tool(say("flowAlignTop")));
    const tops = ["D", "E"].map(
      (id) => text().match(new RegExp(`%% tisty: ${id} -?\\d+ (-?\\d+)`))?.[1],
    );
    expect(tops[0]).toBe(tops[1]);
    expect(tops[0]).toBeDefined();

    expect(tool(say("flowDelete")).disabled).toBe(true);
  });

  it("draws a lane with its shapes in it, and moves them together", () => {
    opened(
      "flowchart LR\n  subgraph cli [Cliente]\n    A[Pide]\n  end\n  A --> B\n  %% tisty: cli 0 0 260 480\n  %% tisty: A 40 60\n  %% tisty: B 400 60\n",
    );
    const lane = document.querySelector('[data-lane="group:cli"]') as HTMLElement;
    expect(node("A").closest(".react-flow__node")?.getAttribute("data-id")).toBe("A");

    fireEvent.click(within(lane).getByRole("button", { name: "Cliente" }));
    fireEvent.keyDown(lane.closest(".react-flow__node") as HTMLElement, { key: "ArrowRight" });

    expect(text()).toMatch(/%% tisty: cli [1-9]\d* 0 260 480/);
    expect(text()).toMatch(/%% tisty: A (4[1-9]|[5-9]\d) 60/);
    expect(text()).toContain("%% tisty: B 400 60");
  });

  it("puts a lane from the palette, sized, and lays the chart out by itself", () => {
    opened();

    fireEvent.click(palette().getByRole("button", { name: say("flowLaneColumn") }));
    expect(text()).toMatch(new RegExp(`subgraph g2 \\[${say("flowNewLane")}\\]\\n  end`));
    expect(text()).toMatch(/%% tisty: g2 -?\d+ -?\d+ 260 480/);
    fireEvent.click(screen.getByRole("button", { name: say("flowNewLane") }));
    expect(screen.getByLabelText(say("flowText"))).toBeTruthy();

    fireEvent.click(tool(say("flowRelaid")));
    expect(text()).toMatch(/%% tisty: A \d+ \d+\n/);
  });

  it("lays the chart out again running another way, from its own menu", () => {
    opened();

    fireEvent.click(tool(say("flowRelaid")));
    fireEvent.click(screen.getByRole("button", { name: say("flowDown") }));
    expect(text()).toMatch(/^flowchart TB\n/);
    expect(text()).toContain("%% tisty: A ");
    expect(screen.queryByRole("button", { name: say("flowDown") })).toBeNull();

    fireEvent.click(tool(say("flowUndo")));
    expect(text()).toBe(FLOW);
  });

  it("lays itself out after every change while asked, and keeps the places once let go", () => {
    opened(`${FLOW}  %% tisty: A 300 100\n`);
    const auto = () => tool(say("flowAuto"));
    expect(auto().getAttribute("aria-pressed")).toBe("false");

    fireEvent.click(auto());
    expect(auto().getAttribute("aria-pressed")).toBe("true");
    expect(text()).toContain("  %% tisty: auto\n");
    expect(text()).not.toMatch(/%% tisty: [A-E] /);

    fireEvent.click(palette().getByRole("button", { name: say("flowShapeCloud") }));
    fireEvent.keyDown(within(node("n1")).getByRole("textbox"), { key: "Escape" });
    expect(text()).toContain("n1@{");
    expect(text()).not.toMatch(/%% tisty: n1 /);

    fireEvent.click(auto());
    expect(text()).not.toContain("%% tisty: auto");
    for (const id of ["A", "B", "C", "D", "E", "n1"]) expect(text()).toContain(`%% tisty: ${id} `);
  });

  it("closes the colours with Escape or a press elsewhere", () => {
    opened();
    pickNode("A");
    const swatches = () => screen.queryByRole("group", { name: say("flowFill") });

    fireEvent.click(tool(say("flowFill")));
    fireEvent.keyDown(screen.getByRole("button", { name: say("colorBlue") }), { key: "Escape" });
    expect(swatches()).toBeNull();

    fireEvent.click(tool(say("flowFill")));
    fireEvent.pointerDown(canvas());
    expect(swatches()).toBeNull();
    expect(text()).toBe(FLOW);
  });

  it("leaves as Mermaid draws it a flowchart it cannot keep", () => {
    opened("flowchart LR\n  A --> B\n  subgraph g\n    A\n");

    expect(document.querySelector("[data-node]")).toBeNull();
    expect(screen.queryByText(say("flowPick"))).toBeNull();
  });
});
