import { fireEvent, render, screen, within } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { fill, t } from "../locales";
import Diagramming from "../ui/drawing/Diagramming";

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

const SCHEMA =
  "erDiagram\n  personas ||--o{ compartidos : publica\n  personas {\n    uuid id PK\n  }\n  compartidos {\n    text id PK\n    uuid persona_id\n  }\n";

const paper = { id: "D1", file: "f1", title: "Share", place: "" };
const opened = (source = SCHEMA) => render(<Diagramming paper={paper} source={source} />);
const text = () => (screen.getByLabelText(t("diagramText")) as HTMLTextAreaElement).value;
const card = (name: string) => document.querySelector(`[data-table="${name}"]`) as HTMLElement;
const title = (name: string) => within(card(name)).getAllByRole("button")[0];
const columnOf = (table: string, words: string) =>
  within(card(table)).getByRole("button", { name: new RegExp(words) });
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
    expect(screen.getByText(t("schemaPick"))).toBeTruthy();
  });

  it("renames a table from the panel, in its relations too", () => {
    opened();
    fireEvent.click(title("personas"));

    leave(t("schemaName"), "gente");

    expect(text()).toBe(SCHEMA.replace(/personas/g, "gente"));
  });

  it("adds a table with an id where there is room", () => {
    opened();

    fireEvent.click(screen.getByRole("button", { name: t("schemaAddTable") }));

    const name = t("schemaNewTable");
    expect(text()).toMatch(
      new RegExp(
        `  ${name} \\{\\n    int id PK\\n  \\}\\n  %% tisty: ${name} \\d+ 40 required id\\n$`,
      ),
    );
  });

  it("gives a column a type, a key and a word that it is required", () => {
    opened();
    fireEvent.click(columnOf("compartidos", "persona_id"));

    fireEvent.click(screen.getByRole("button", { name: "varchar(255)" }));
    fireEvent.click(screen.getByLabelText("UK"));
    fireEvent.click(screen.getByLabelText(t("schemaRequired")));

    expect(text()).toBe(
      `${SCHEMA.replace("uuid persona_id", "varchar(255) persona_id UK")}  %% tisty: compartidos required persona_id\n`,
    );
  });

  it("moves a table by its title and writes only where it sits", () => {
    opened();
    const head = title("personas");

    fireEvent.pointerDown(head, { clientX: 0, clientY: 0, pointerId: 1 });
    fireEvent.pointerMove(head, { clientX: 100, clientY: 50, pointerId: 1, buttons: 1 });
    fireEvent.pointerUp(head, { clientX: 100, clientY: 50, pointerId: 1 });

    expect(text()).toBe(`${SCHEMA}  %% tisty: personas 140 90\n`);
  });

  it("relates a column to another table by dragging its dot there", () => {
    opened(SCHEMA.replace("  personas ||--o{ compartidos : publica\n", ""));
    const grip = card("compartidos").querySelector("[data-grip]:nth-of-type(1)") as HTMLElement;
    const dots = card("compartidos").querySelectorAll("[data-grip]");
    document.elementFromPoint = () => card("personas");

    fireEvent.pointerDown(dots[1] ?? grip, { clientX: 0, clientY: 0, pointerId: 1 });
    fireEvent.pointerMove(dots[1] ?? grip, { clientX: 50, clientY: 0, pointerId: 1, buttons: 1 });
    fireEvent.pointerUp(dots[1] ?? grip, { clientX: 50, clientY: 0, pointerId: 1 });

    expect(text()).toBe(
      "erDiagram\n  personas {\n    uuid id PK\n  }\n  compartidos {\n    text id PK\n    uuid persona_id FK\n  }\n  personas ||--o{ compartidos : persona_id\n",
    );
    expect(screen.getByText(t("schemaRelation"))).toBeTruthy();
  });

  it("changes a relation's ends from the panel", () => {
    opened();
    fireEvent.click(screen.getByRole("button", { name: /publica/ }));

    fireEvent.change(screen.getByLabelText(fill("schemaSide", "compartidos")), {
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

    fireEvent.click(screen.getByRole("button", { name: t("schemaCopySql") }));
    fireEvent.click(screen.getByRole("button", { name: t("schemaCopyDbml") }));

    expect(await screen.findByText(fill("schemaCopied", "DBML"))).toBeTruthy();
    expect(copied.said[0]).toContain("CREATE TABLE personas (");
    expect(copied.said[1]).toContain("Table compartidos {");
  });

  it("grows and prunes a table from the panel: a column, its words and note, then the table", () => {
    opened();
    fireEvent.click(title("personas"));

    fireEvent.click(screen.getByRole("button", { name: t("schemaAddColumn") }));
    leave(t("schemaName"), "correo electrónico");
    leave(t("schemaType"), "text");
    leave(t("schemaNote"), "para avisos");
    fireEvent.click(screen.getByLabelText("PK"));
    expect(text()).toBe(
      SCHEMA.replace(
        "    uuid id PK\n",
        '    uuid id PK\n    text correo_electrónico PK "para avisos"\n',
      ),
    );

    fireEvent.click(screen.getByLabelText("PK"));
    fireEvent.click(screen.getByRole("button", { name: t("schemaRemoveColumn") }));
    expect(text()).toBe(SCHEMA);

    fireEvent.click(screen.getByRole("button", { name: t("schemaRemoveTable") }));
    expect(text()).toBe("erDiagram\n  compartidos {\n    text id PK\n    uuid persona_id\n  }\n");
    expect(screen.getByText(t("schemaPick"))).toBeTruthy();
  });

  it("names a relation, makes it dashed and removes it from the panel", () => {
    opened();
    fireEvent.click(screen.getByRole("button", { name: /publica/ }));

    leave(t("schemaLabel"), "lo publica");
    fireEvent.click(screen.getByLabelText(t("schemaIdentifying")));
    expect(text()).toBe(
      SCHEMA.replace("||--o{ compartidos : publica", '||..o{ compartidos : "lo publica"'),
    );

    fireEvent.click(screen.getByRole("button", { name: t("schemaRemoveRelation") }));
    expect(text()).toBe(SCHEMA.replace("  personas ||--o{ compartidos : publica\n", ""));
  });

  it("leaves as Mermaid draws it a schema it cannot keep", () => {
    opened("erDiagram\n  A {\n    int id\n  }\n  A {\n    int otra\n  }\n");

    expect(document.querySelector("[data-table]")).toBeNull();
    expect(screen.queryByText(t("schemaPick"))).toBeNull();
  });
});
