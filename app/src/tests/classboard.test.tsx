import { fireEvent, render, screen, within } from "@testing-library/react";
import { sayIn } from "tisty-diagrams/read";
import { beforeEach, describe, expect, it, vi } from "vitest";
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

const CLASSES =
  "classDiagram\n  class Animal {\n    +String name\n  }\n  class Duck\n  Animal <|-- Duck : hereda\n";

const paper = { id: "D1", file: "f1", title: "Zoo", place: "" };
const opened = (source = CLASSES) => render(<Diagramming paper={paper} source={source} />);
const text = () => (screen.getByLabelText(say("diagramText")) as HTMLTextAreaElement).value;
const card = (name: string) => document.querySelector(`[data-node="${name}"]`) as HTMLElement;
const pickClass = (name: string) => fireEvent.click(within(card(name)).getByRole("button"));
const leave = (label: string, value: string) => {
  const field = screen.getByLabelText(label) as HTMLInputElement;
  fireEvent.change(field, { target: { value } });
  fireEvent.blur(field);
};

beforeEach(() => {
  vi.restoreAllMocks();
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
    const grip = card("A").querySelector("[data-grip]") as HTMLElement;
    document.elementFromPoint = () => card("B");
    fireEvent.pointerDown(grip, { clientX: 0, clientY: 0, pointerId: 1 });
    fireEvent.pointerUp(grip, { clientX: 40, clientY: 0, pointerId: 1 });
    expect(text()).toBe("classDiagram\n  class A\n  class B\n  A --> B\n");

    fireEvent.change(screen.getByLabelText(say("classKind")), { target: { value: "*--" } });
    leave(`${say("classCard")} · A`, "1");
    leave(`${say("classCard")} · B`, "many");
    leave(say("flowLabel"), "tiene");

    expect(text()).toBe('classDiagram\n  class A\n  class B\n  A "1" *-- "many" B : tiene\n');
  });

  it("adds a class and deletes the chosen one with Delete, relations and all", () => {
    opened();

    fireEvent.click(screen.getByRole("button", { name: say("classAdd") }));
    expect(text()).toBe(`${CLASSES}  class ${say("classNew")}1\n`);

    pickClass("Duck");
    fireEvent.keyDown(within(card("Duck")).getByRole("button"), { key: "Delete" });
    expect(text()).toBe(
      `classDiagram\n  class Animal {\n    +String name\n  }\n  class ${say("classNew")}1\n`,
    );
  });

  it("leaves as Mermaid draws it a class diagram it cannot keep", () => {
    opened("classDiagram\n  namespace Zoo {\n    class A\n  }\n");

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
