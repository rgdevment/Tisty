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

const FLOW =
  "flowchart LR\n  A[Inicio] --> B{¿Pago?}\n  B -->|sí| C(Cobro)\n  subgraph g1 [Caja]\n    C\n  end\n  C --> D --> E\n";

const paper = { id: "D1", file: "f1", title: "Pagos", place: "" };
const opened = (source = FLOW) => render(<Diagramming paper={paper} source={source} />);
const text = () => (screen.getByLabelText(t("diagramText")) as HTMLTextAreaElement).value;
const node = (id: string) => document.querySelector(`[data-node="${id}"]`) as HTMLElement;
const pickNode = (id: string) => fireEvent.click(within(node(id)).getByRole("button"));
const leave = (label: string, value: string) => {
  const field = screen.getByLabelText(label) as HTMLInputElement;
  fireEvent.change(field, { target: { value } });
  fireEvent.blur(field);
};
const choose = (label: string, value: string) =>
  fireEvent.change(screen.getByLabelText(label), { target: { value } });

beforeEach(() => {
  vi.restoreAllMocks();
});

describe("a flowchart in the diagram window", () => {
  it("draws every node and group itself and asks for a choice", () => {
    opened();

    expect(document.querySelectorAll("[data-node]")).toHaveLength(5);
    expect(screen.getByRole("button", { name: "Caja" })).toBeTruthy();
    expect(screen.getByText(t("flowPick"))).toBeTruthy();
  });

  it("renames and reshapes a node from the panel, where its words are written", () => {
    opened();
    pickNode("B");

    leave(t("flowText"), "¿Pagó?");
    choose(t("flowShape"), "hexagon");

    expect(text()).toBe(FLOW.replace("B{¿Pago?}", "B{{¿Pagó?}}"));
  });

  it("adds a node and a group, and joins two nodes by dragging a dot", () => {
    opened();

    fireEvent.click(screen.getByRole("button", { name: t("flowAddNode") }));
    fireEvent.click(screen.getByRole("button", { name: t("flowAddGroup") }));
    const grip = node("A").querySelector("[data-grip]") as HTMLElement;
    document.elementFromPoint = () => node("E");
    fireEvent.pointerDown(grip, { clientX: 0, clientY: 0, pointerId: 1 });
    fireEvent.pointerMove(grip, { clientX: 40, clientY: 0, pointerId: 1, buttons: 1 });
    fireEvent.pointerUp(grip, { clientX: 40, clientY: 0, pointerId: 1 });

    expect(text()).toBe(
      `${FLOW}  n1[${t("flowNewNode")}]\n  subgraph g2 [${t("flowNewGroup")}]\n  end\n  A --> E\n`,
    );
    expect(screen.getByText(`${t("flowEdge")} · A → E`)).toBeTruthy();
  });

  it("labels an arrow and changes its kind, and takes out a lone one with Delete", () => {
    opened();
    fireEvent.click(screen.getByRole("button", { name: "A → B" }));

    leave(t("flowLabel"), "empieza");
    choose(t("flowArrow"), "-.->");
    expect(text()).toBe(FLOW.replace("A[Inicio] --> B", "A[Inicio] -.->|empieza| B"));

    fireEvent.keyDown(screen.getByRole("button", { name: "empieza" }), { key: "Delete" });
    expect(text()).toBe(FLOW.replace("  A[Inicio] --> B{¿Pago?}\n", "  A[Inicio]\n  B{¿Pago?}\n"));
  });

  it("will not take out one link of a chain, and says why", () => {
    opened();
    fireEvent.click(screen.getByRole("button", { name: "C → D" }));

    const remove = screen.getByRole("button", { name: t("flowRemoveEdge") }) as HTMLButtonElement;
    expect(remove.disabled).toBe(true);
    expect(remove.title).toBe(t("flowChained"));
  });

  it("renames a group, moves a node into it, and takes the group apart", () => {
    opened();
    fireEvent.click(screen.getByRole("button", { name: "Caja" }));
    leave(t("flowText"), "Caja fuerte");
    expect(text()).toBe(FLOW.replace("[Caja]", "[Caja fuerte]"));

    pickNode("A");
    choose(t("flowGroup"), "g1");
    expect(text()).toContain("    C\n    A\n  end");

    fireEvent.click(screen.getByRole("button", { name: "Caja fuerte" }));
    fireEvent.click(screen.getByRole("button", { name: t("flowUngroup") }));
    expect(text()).not.toContain("subgraph");
  });

  it("turns the chart another way from the panel", () => {
    opened();

    choose(t("flowDirection"), "TB");

    expect(text()).toBe(FLOW.replace("flowchart LR", "flowchart TB"));
  });

  it("leaves as Mermaid draws it a flowchart it cannot keep", () => {
    opened("flowchart LR\n  A --> B\n  subgraph g\n    A\n");

    expect(document.querySelector("[data-node]")).toBeNull();
    expect(screen.queryByText(t("flowPick"))).toBeNull();
  });
});
