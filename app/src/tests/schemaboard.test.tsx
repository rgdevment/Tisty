import { fireEvent, render, screen, within } from "@testing-library/react";
import { sayIn } from "tisty-diagrams/read";
import { beforeAll, beforeEach, describe, expect, it, vi } from "vitest";
import Diagramming from "../ui/drawing/Diagramming";

// The studio says its own words; these tests read them as it does, in English.
const say = sayIn("en");

const copied = vi.hoisted(() => ({ said: [] as string[] }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: () => Promise.resolve(null),
  convertFileSrc: (path: string) => path,
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({ ask: () => Promise.resolve(true) }));

vi.mock("@tauri-apps/plugin-clipboard-manager", () => ({
  writeText: (said: string) => {
    copied.said.push(said);
    return Promise.resolve();
  },
}));

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

const SCHEMA =
  "erDiagram\n  personas ||--o{ compartidos : publica\n  personas {\n    uuid id PK\n  }\n  compartidos {\n    text id PK\n    uuid persona_id\n  }\n";

const paper = { id: "D1", file: "f1", title: "Share", place: "" };
const opened = (source = SCHEMA) => render(<Diagramming paper={paper} source={source} />);
const text = () => (screen.getByLabelText(say("diagramText")) as HTMLTextAreaElement).value;
const card = (name: string) => document.querySelector(`[data-table="${name}"]`) as HTMLElement;
const title = (name: string) => within(card(name)).getAllByRole("button")[0];
const columnOf = (table: string, words: string) =>
  within(card(table)).getByRole("button", { name: new RegExp(words) });
const palette = () => within(screen.getByRole("complementary", { name: say("flowPalette") }));
const tool = (name: string) =>
  within(screen.getByRole("toolbar", { name: say("flowTools") })).getByRole("button", {
    name,
  }) as HTMLButtonElement;
const typed = (field: HTMLElement, value: string) => {
  fireEvent.change(field, { target: { value } });
  fireEvent.keyDown(field, { key: "Enter" });
};
const nudge = (name: string) =>
  fireEvent.keyDown(card(name).closest(".react-flow__node") as HTMLElement, { key: "ArrowRight" });
// A column's dot dragged and let go over what `under` is.
const pull = (table: string, column: number, under: HTMLElement | null) => {
  document.elementFromPoint = () => under;
  const dot = card(table).querySelector(`[data-handleid="column-${column}"]`) as HTMLElement;
  fireEvent.mouseDown(dot, { clientX: 0, clientY: 0, button: 0 });
  fireEvent.mouseMove(document, { clientX: 300, clientY: 300 });
  fireEvent.mouseUp(document, { clientX: 300, clientY: 300 });
};
const leave = (label: string, value: string) => {
  const field = screen.getByLabelText(label) as HTMLInputElement;
  fireEvent.change(field, { target: { value } });
  fireEvent.blur(field);
};

beforeEach(() => {
  vi.restoreAllMocks();
  copied.said = [];
});

describe("a database schema in the diagram window", () => {
  it("draws each table as a card with its columns and asks for a choice", () => {
    opened();

    expect(document.querySelectorAll("[data-table]")).toHaveLength(2);
    expect(columnOf("compartidos", "persona_id")).toBeTruthy();
    expect(screen.getByText(say("schemaPick"))).toBeTruthy();
  });

  it("renames a table from the panel, in its relations too", () => {
    opened();
    fireEvent.click(title("personas"));

    leave(say("schemaName"), "gente");

    expect(text()).toBe(SCHEMA.replace(/personas/g, "gente"));
  });

  it("adds a pressed table with an id in the next free cell, named at once", () => {
    opened();

    fireEvent.click(palette().getByRole("button", { name: say("schemaTable") }));
    const name = say("schemaNewTable");
    expect(text()).toMatch(
      new RegExp(`  ${name} \\{\\n    int id PK\\n  \\}\\n  %% tisty: ${name} required id\\n$`),
    );

    typed(within(card(name)).getByRole("textbox"), "pagos");
    expect(text()).toContain("  pagos {\n");
    expect(text()).not.toContain("%% tisty: personas");
  });

  it("gives a column a type, a key and a word that it is required", () => {
    opened();
    fireEvent.click(columnOf("compartidos", "persona_id"));

    fireEvent.click(screen.getByRole("button", { name: "varchar(255)" }));
    fireEvent.click(screen.getByLabelText("UK"));
    fireEvent.click(screen.getByLabelText(say("schemaRequired")));

    expect(text()).toBe(
      `${SCHEMA.replace("uuid persona_id", "varchar(255) persona_id UK")}  %% tisty: compartidos required persona_id\n`,
    );
  });

  it("moves a table and writes only where it sits, and undoes it", () => {
    opened();
    fireEvent.click(title("personas"));

    nudge("personas");
    expect(text()).toMatch(/^erDiagram[\s\S]*\n {2}%% tisty: personas \d+ \d+\n$/);
    expect(text()).not.toContain("%% tisty: compartidos");

    fireEvent.click(tool(say("flowUndo")));
    expect(text()).toBe(SCHEMA);
  });

  it("renames a table in place, in its relations too", () => {
    opened();

    fireEvent.doubleClick(title("personas"));
    typed(within(card("personas")).getByRole("textbox"), "gente");

    expect(text()).toBe(SCHEMA.replace(/personas/g, "gente"));
  });

  it("relates a column to another table by dragging its dot there", () => {
    opened(SCHEMA.replace("  personas ||--o{ compartidos : publica\n", ""));

    pull("compartidos", 1, card("personas"));

    expect(text()).toBe(
      "erDiagram\n  personas {\n    uuid id PK\n  }\n  compartidos {\n    text id PK\n    uuid persona_id FK\n  }\n  personas ||--o{ compartidos : persona_id\n",
    );
    expect(screen.getByText(say("schemaRelation"))).toBeTruthy();
  });

  it("makes a related table where a column's dot is let go on nothing", () => {
    opened(SCHEMA.replace("  personas ||--o{ compartidos : publica\n", ""));

    pull("compartidos", 1, null);

    const name = say("schemaNewTable");
    expect(text()).toContain(`  ${name} ||--o{ compartidos : persona_id\n`);
    expect(text()).toContain("    uuid persona_id FK\n");
    expect(within(card(name)).getByRole("textbox")).toBeTruthy();
  });

  it("chooses several tables, lines them up and puts them back in the grid", () => {
    opened(`${SCHEMA}  %% tisty: personas 40 40\n  %% tisty: compartidos 400 200\n`);
    fireEvent.click(title("personas"));
    fireEvent.keyDown(window, { key: "Control" });
    fireEvent.click(title("compartidos"), { ctrlKey: true });
    fireEvent.keyUp(window, { key: "Control" });

    fireEvent.click(tool(say("flowAlignLeft")));
    expect(text()).toContain("%% tisty: compartidos 40 200");

    fireEvent.click(tool(say("flowRelaid")));
    expect(text()).toBe(SCHEMA);

    fireEvent.click(tool(say("flowUndo")));
    fireEvent.click(tool(say("flowUndo")));
    fireEvent.click(title("personas"));
    fireEvent.keyDown(window, { key: "Control" });
    fireEvent.click(title("compartidos"), { ctrlKey: true });
    fireEvent.keyUp(window, { key: "Control" });
    fireEvent.keyDown(title("personas"), { key: "Delete" });
    expect(text()).toBe("erDiagram\n");
  });

  it("changes a relation's ends from the panel", () => {
    opened();
    fireEvent.click(screen.getByRole("button", { name: /publica/ }));

    fireEvent.change(screen.getByLabelText(say("schemaSide", "compartidos")), {
      target: { value: "one-many" },
    });

    expect(text()).toBe(SCHEMA.replace("||--o{", "||--|{"));
  });

  it("deletes what is chosen with the Delete key", () => {
    opened();
    const column = columnOf("compartidos", "persona_id");
    fireEvent.click(column);

    fireEvent.keyDown(column, { key: "Delete" });

    expect(text()).toBe(SCHEMA.replace("    uuid persona_id\n", ""));
  });

  it("copies the schema as SQL and as DBML", async () => {
    opened();

    fireEvent.click(screen.getByRole("button", { name: say("schemaCopySql") }));
    fireEvent.click(screen.getByRole("button", { name: say("schemaCopyDbml") }));

    expect(await screen.findByText(say("schemaCopied", "DBML"))).toBeTruthy();
    expect(copied.said[0]).toContain("CREATE TABLE personas (");
    expect(copied.said[1]).toContain("Table compartidos {");
  });

  it("grows and prunes a table from the panel: a column, its words and note, then the table", () => {
    opened();
    fireEvent.click(title("personas"));

    fireEvent.click(screen.getByRole("button", { name: say("schemaAddColumn") }));
    leave(say("schemaName"), "correo electrónico");
    leave(say("schemaType"), "text");
    leave(say("schemaNote"), "para avisos");
    fireEvent.click(screen.getByLabelText("PK"));
    expect(text()).toBe(
      SCHEMA.replace(
        "    uuid id PK\n",
        '    uuid id PK\n    text correo_electrónico PK "para avisos"\n',
      ),
    );

    fireEvent.click(screen.getByLabelText("PK"));
    fireEvent.click(screen.getByRole("button", { name: say("schemaRemoveColumn") }));
    expect(text()).toBe(SCHEMA);

    fireEvent.click(screen.getByRole("button", { name: say("schemaRemoveTable") }));
    expect(text()).toBe("erDiagram\n  compartidos {\n    text id PK\n    uuid persona_id\n  }\n");
    expect(screen.getByText(say("schemaPick"))).toBeTruthy();
  });

  it("names a relation, makes it dashed and removes it from the panel", () => {
    opened();
    fireEvent.click(screen.getByRole("button", { name: /publica/ }));

    leave(say("schemaLabel"), "lo publica");
    fireEvent.click(screen.getByLabelText(say("schemaIdentifying")));
    expect(text()).toBe(
      SCHEMA.replace("||--o{ compartidos : publica", '||..o{ compartidos : "lo publica"'),
    );

    fireEvent.click(screen.getByRole("button", { name: say("schemaRemoveRelation") }));
    expect(text()).toBe(SCHEMA.replace("  personas ||--o{ compartidos : publica\n", ""));
  });

  it("leaves as Mermaid draws it a schema it cannot keep", () => {
    opened("erDiagram\n  A {\n    int id\n  }\n  A {\n    int otra\n  }\n");

    expect(document.querySelector("[data-table]")).toBeNull();
    expect(screen.queryByText(say("schemaPick"))).toBeNull();
  });
});

describe("choosing in a schema edited by hand", () => {
  it("keeps choosing the same column when a line is added above it", () => {
    opened();
    fireEvent.click(columnOf("compartidos", "persona_id"));

    fireEvent.change(screen.getByLabelText(say("diagramText")), {
      target: { value: SCHEMA.replace("    text id PK\n", "    text id PK\n    date nueva\n") },
    });

    expect((screen.getByLabelText(say("schemaName")) as HTMLInputElement).value).toBe("persona_id");
    fireEvent.keyDown(columnOf("compartidos", "persona_id"), { key: "Delete" });
    expect(text()).toContain("    date nueva\n  }");
    expect(text()).not.toContain("persona_id");
  });
});

describe("two relations alike in a schema edited by hand", () => {
  it("keeps choosing the second of two identical relations", () => {
    const twice = "erDiagram\n  a ||--o{ b : tiene\n  a ||--o{ b : tiene\n";
    opened(twice);
    fireEvent.click(screen.getAllByRole("button", { name: /tiene/ })[1]);

    fireEvent.change(screen.getByLabelText(say("diagramText")), {
      target: { value: twice.replace("erDiagram\n", "erDiagram\n  direction LR\n") },
    });

    const edges = document.querySelectorAll(".react-flow__edge");
    expect(Array.from(edges, (one) => one.classList.contains("selected"))).toEqual([false, true]);
  });
});
