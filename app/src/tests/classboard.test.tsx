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

vi.mock("tisty-reader/page", async (real) => ({
  ...(await real<typeof import("tisty-reader/page")>()),
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

const CLASSES =
  "classDiagram\n  class Animal {\n    +String name\n  }\n  class Duck\n  Animal <|-- Duck : hereda\n";

const paper = { id: "D1", file: "f1", title: "Zoo", place: "" };
const opened = (source = CLASSES) => render(<Diagramming paper={paper} source={source} />);
const text = () => (screen.getByLabelText(say("diagramText")) as HTMLTextAreaElement).value;
const card = (name: string) => document.querySelector(`[data-node="${name}"]`) as HTMLElement;
const pickClass = (name: string) => fireEvent.click(within(card(name)).getByRole("button"));
const palette = () => within(screen.getByRole("complementary", { name: say("flowPalette") }));
const tool = (name: string) =>
  within(screen.getByRole("toolbar", { name: say("flowTools") })).getByRole("button", {
    name,
  }) as HTMLButtonElement;
const typed = (field: HTMLElement, value: string) => {
  fireEvent.change(field, { target: { value } });
  fireEvent.keyDown(field, { key: "Enter" });
};
const pull = (from: string, under: HTMLElement | null, to = 300) => {
  document.elementFromPoint = () => under;
  const dot = card(from).querySelector(".react-flow__handle-right") as HTMLElement;
  fireEvent.mouseDown(dot, { clientX: 0, clientY: 0, button: 0 });
  fireEvent.mouseMove(document, { clientX: to, clientY: to });
  fireEvent.mouseUp(document, { clientX: to, clientY: to });
};
const leave = (label: string, value: string) => {
  const field = screen.getByLabelText(label) as HTMLInputElement;
  fireEvent.change(field, { target: { value } });
  fireEvent.blur(field);
};

beforeEach(() => {
  vi.restoreAllMocks();
});

// React Flow says what it dislikes only while developing; a test that listens turns that on.
afterEach(() => {
  vi.unstubAllEnvs();
});

describe("a class diagram in the diagram window", () => {
  it("draws each class as a card with its members and asks for a choice", () => {
    opened();

    expect(document.querySelectorAll("[data-node]")).toHaveLength(2);
    expect(within(card("Animal")).getByText("+String name")).toBeTruthy();
    expect(screen.getByText(say("classPick"))).toBeTruthy();
  });

  it("renames a class everywhere it is named", () => {
    opened();
    pickClass("Animal");

    leave(say("schemaName"), "Ser");

    expect(text()).toBe(CLASSES.replace(/Animal/g, "Ser"));
  });

  it("adds, rewrites and removes members, giving braces to a class without them", () => {
    opened();
    pickClass("Duck");

    fireEvent.click(screen.getByRole("button", { name: say("classAddMethod") }));
    expect(text()).toBe(
      CLASSES.replace("  class Duck\n", `  class Duck {\n    ${say("classNewMethod")}\n  }\n`),
    );

    pickClass("Animal");
    leave(say("classAttribute"), "+int edad");
    fireEvent.click(screen.getByRole("button", { name: say("classAddAttribute") }));
    expect(text()).toContain(`    +int edad\n    ${say("classNewAttribute")}\n  }`);

    fireEvent.click(screen.getAllByRole("button", { name: say("classRemoveMember") })[0]);
    expect(text()).not.toContain("+int edad");
  });

  it("relates two classes by dragging a dot, and gives the relation a kind, ends and words", () => {
    opened("classDiagram\n  class A\n  class B\n");
    pull("A", card("B"));
    expect(text()).toMatch(/^classDiagram\n {2}class A\n {2}class B\n {2}A --> B\n/);

    fireEvent.change(screen.getByLabelText(say("classKind")), { target: { value: "*--" } });
    leave(`${say("classCard")} · A`, "1");
    leave(`${say("classCard")} · B`, "many");
    leave(say("flowLabel"), "tiene");

    expect(text()).toContain('  A "1" --* "many" B : tiene\n');
  });

  it("adds a class from the palette, named at once, and deletes the chosen one with Delete", () => {
    opened();

    fireEvent.click(palette().getByRole("button", { name: say("classInterface") }));
    typed(within(card(`${say("classNew")}1`)).getByRole("textbox"), "Volador");
    expect(text()).toContain("  class Volador\n  <<interface>> Volador\n");
    expect(text()).toMatch(/%% tisty: Volador -?\d+ -?\d+/);

    pickClass("Duck");
    fireEvent.keyDown(within(card("Duck")).getByRole("button"), { key: "Delete" });
    expect(text()).not.toMatch(/Duck/);
  });

  it("makes a related class where a relation is let go on nothing, and none on its own class", () => {
    opened();

    pull("Duck", null);
    fireEvent.keyDown(within(card(`${say("classNew")}1`)).getByRole("textbox"), { key: "Escape" });
    expect(text()).toContain(`  Duck --> ${say("classNew")}1\n`);

    const before = text();
    pull("Duck", card("Duck"), 900);
    expect(text()).toBe(before);
  });

  it("changes a class's kind and a member's visibility from the panel", () => {
    opened();
    pickClass("Animal");

    fireEvent.change(screen.getByLabelText(say("classSort")), { target: { value: "abstract" } });
    expect(text()).toContain("  <<abstract>> Animal\n");
    fireEvent.change(screen.getByLabelText(say("classVisibility")), { target: { value: "-" } });
    expect(text()).toContain("    -String name\n");
    fireEvent.change(screen.getByLabelText(say("classSort")), { target: { value: "" } });
    expect(text()).not.toContain("<<abstract>>");
  });

  it("changes a relation's kind from the toolbar, its head kept at the same class", () => {
    opened("classDiagram\n  class A\n  class B\n  B --|> A\n");
    fireEvent.click(screen.getByRole("button", { name: "B → A" }));

    expect(tool(say("classInherits")).getAttribute("aria-pressed")).toBe("true");
    fireEvent.click(tool(say("classComposes")));
    expect(text()).toContain("  B --* A\n");
    fireEvent.click(tool(say("classAssociates")));
    expect(text()).toContain("  B --> A\n");
  });

  it("renames a class in place, keeping where it stands, and undoes it", () => {
    opened(`${CLASSES}  %% tisty: Animal 10 20\n  %% tisty: Duck 10 200\n`);

    fireEvent.doubleClick(within(card("Animal")).getByRole("button"));
    typed(within(card("Animal")).getByRole("textbox"), "Ser");
    expect(text()).toContain("%% tisty: Ser 10 20");
    expect(text()).toContain("Ser <|-- Duck");

    fireEvent.click(tool(say("flowUndo")));
    expect(text()).toContain("%% tisty: Animal 10 20");
  });

  it("chooses several classes, colours, lines them up and lays them out again", () => {
    opened(`${CLASSES}  %% tisty: Animal 10 20\n  %% tisty: Duck 300 200\n`);
    pickClass("Animal");
    fireEvent.keyDown(window, { key: "Control" });
    fireEvent.click(within(card("Duck")).getByRole("button"), { ctrlKey: true });
    fireEvent.keyUp(window, { key: "Control" });
    expect(screen.getByText(say("flowPickedMany", "2"))).toBeTruthy();

    fireEvent.click(tool(say("flowAlignLeft")));
    expect(text()).toContain("%% tisty: Duck 10 200");
    fireEvent.click(tool(say("flowFill")));
    fireEvent.click(screen.getByRole("button", { name: say("colorBlue") }));
    expect(text()).toContain("  style Animal fill:#dbeafe,color:#1e3a8a\n");
    expect(text()).toContain("  style Duck fill:#dbeafe,color:#1e3a8a\n");

    fireEvent.click(tool(say("flowRelaid")));
    expect(text()).not.toContain("%% tisty: Duck 10 200");
    expect(text()).toMatch(/%% tisty: Duck \d+ \d+/);
  });

  it("puts a package with a class in it from the palette, and moves the package with its classes", () => {
    opened();

    fireEvent.click(palette().getByRole("button", { name: say("classNamespace") }));
    typed(within(card(`${say("classNew")}1`)).getByRole("textbox"), "Pez");
    expect(text()).toContain(`  namespace ${say("classNewNamespace")}1 {\n    class Pez\n  }\n`);
    expect(text()).toMatch(
      new RegExp(`%% tisty: ${say("classNewNamespace")}1 -?\\d+ -?\\d+ 300 360`),
    );

    const lane = document.querySelector(
      `[data-lane="group:${say("classNewNamespace")}1"]`,
    ) as HTMLElement;
    const before = Number(text().match(/%% tisty: Pez (-?\d+)/)?.[1]);
    fireEvent.click(within(lane).getByRole("button", { name: `${say("classNewNamespace")}1` }));
    fireEvent.keyDown(lane.closest(".react-flow__node") as HTMLElement, { key: "ArrowRight" });
    expect(Number(text().match(/%% tisty: Pez (-?\d+)/)?.[1])).toBeGreaterThan(before);
  });

  it("moves a class into a package from the panel, and renames and takes the package apart", () => {
    opened("classDiagram\n  namespace Zoo {\n    class Animal\n  }\n  class Duck\n");
    pickClass("Duck");

    fireEvent.change(screen.getByLabelText(say("classNamespace")), { target: { value: "Zoo" } });
    expect(text()).toBe("classDiagram\n  namespace Zoo {\n    class Animal\n    class Duck\n  }\n");

    fireEvent.click(screen.getByRole("button", { name: "Zoo" }));
    leave(say("schemaName"), "Granja");
    expect(text()).toContain("  namespace Granja {");
    fireEvent.click(screen.getByRole("button", { name: say("classUngroup") }));
    expect(text()).not.toContain("namespace");
  });

  it("moves a package written by hand, which gets its size the first time", () => {
    opened("classDiagram\n  namespace Zoo {\n    class A\n  }\n  class B\n");
    const lane = document.querySelector('[data-lane="group:Zoo"]') as HTMLElement;

    fireEvent.click(within(lane).getByRole("button", { name: "Zoo" }));
    fireEvent.keyDown(lane.closest(".react-flow__node") as HTMLElement, { key: "ArrowRight" });

    expect(text()).toMatch(/%% tisty: Zoo -?\d+ -?\d+ \d+ \d+/);
  });

  it("writes where a class is nudged with the arrow keys, a class React Flow knows the size of", () => {
    vi.stubEnv("NODE_ENV", "development");
    const warned = vi.spyOn(console, "warn");
    opened();
    pickClass("Duck");

    fireEvent.keyDown(card("Duck").closest(".react-flow__node") as HTMLElement, {
      key: "ArrowRight",
    });

    expect(text()).toMatch(/%% tisty: Duck -?\d+ -?\d+/);
    expect(text()).toMatch(/%% tisty: Animal -?\d+ -?\d+/);
    expect(warned.mock.calls.flat().join(" ")).not.toContain("not initialized");
  });

  it("leaves as Mermaid draws it a class diagram it cannot keep", () => {
    opened("classDiagram\n  namespace Zoo {\n    class A\n    A --> B\n  }\n");

    expect(document.querySelector("[data-node]")).toBeNull();
    expect(screen.queryByText(say("classPick"))).toBeNull();
  });
});

describe("choosing in a class diagram edited by hand", () => {
  it("keeps the chosen member and relation when lines are added above them", () => {
    opened();
    pickClass("Animal");
    fireEvent.click(screen.getByRole("button", { name: say("classAddAttribute") }));
    const typed = screen.getByLabelText(say("diagramText"));

    fireEvent.change(typed, {
      target: { value: text().replace("    +String name\n", "    +int id\n    +String name\n") },
    });
    expect(
      (
        screen.getByLabelText(say("classAttribute"), {
          selector: `#class-member-2`,
        }) as HTMLInputElement
      ).value,
    ).toBe(say("classNewAttribute"));

    fireEvent.click(screen.getByRole("button", { name: /hereda/ }));
    fireEvent.change(typed, {
      target: { value: text().replace("  class Duck\n", "  class Duck\n  class Pez\n") },
    });
    expect(screen.getByText(`${say("classLinkTitle")} · Animal – Duck`)).toBeTruthy();
  });
});
