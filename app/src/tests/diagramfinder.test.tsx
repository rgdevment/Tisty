import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { t } from "../locales";
import { kindName } from "../ui/drawing/starting";
import Picking from "../ui/Picking";

const found = vi.hoisted(() => ({
  all: [] as { file: string; title: string; nth: number; source: string }[],
  fails: false,
  drawn: [] as string[],
}));

vi.mock("../drawing", () => ({
  diagramsEverywhere: () =>
    found.fails ? Promise.reject(new Error("refused")) : Promise.resolve(found.all),
}));

vi.mock("../ui/drawing/exporting", () => ({
  svgFor: (source: string) => {
    found.drawn.push(source);
    return Promise.resolve({ svg: "<svg/>", size: { wide: 10, tall: 10 } });
  },
}));

const PAGOS = "flowchart TD\n  A[Inicio] --> B[Fin]\n";
const TIENDA = "erDiagram\n  cliente {\n    int id PK\n  }\n";
const CON_FENCE = "sequenceDiagram\n  A->>B: ```código```\n";

const opened = (kind: "diagram" | "diagramFrom") => {
  const put = vi.fn((_: string) => true);
  const heard = { error: vi.fn(), done: vi.fn() };
  render(
    <Picking
      picked={{ kind, at: { x: 40, y: 40 }, put }}
      pages={[]}
      others={[]}
      onMade={vi.fn()}
      onLost={vi.fn()}
      onError={heard.error}
      onDone={heard.done}
    />,
  );
  return { put, ...heard };
};

beforeEach(() => {
  found.all = [
    { file: "ab12-0001", title: "Pagos", nth: 0, source: PAGOS },
    { file: "ab12-0002", title: "Tienda", nth: 0, source: TIENDA },
    { file: "ab12-0002", title: "Tienda", nth: 1, source: CON_FENCE },
  ];
  found.fails = false;
  found.drawn = [];
});

describe("reusing a diagram another document holds", () => {
  it("lists every diagram with its document, kind and first words, and draws only the one looked at", async () => {
    opened("diagramFrom");

    const rows = await screen.findAllByRole("option");
    expect(rows.map((one) => one.textContent)).toEqual([
      `Pagos${kindName("flowchart")} · A[Inicio] --> B[Fin]`,
      `Tienda${kindName("erDiagram")} · cliente {`,
      `Tienda${kindName("other")} · A->>B: \`\`\`código\`\`\``,
    ]);
    await waitFor(() => expect(found.drawn).toEqual([PAGOS]));

    fireEvent.mouseEnter(rows[1]);
    await waitFor(() => expect(found.drawn).toEqual([PAGOS, TIENDA]));
  });

  it("inserts a copy of the one chosen, with a fence longer than any it holds", async () => {
    const { put, done } = opened("diagramFrom");

    await userEvent.click((await screen.findAllByRole("option"))[2]);

    expect(put).toHaveBeenCalledWith(`\`\`\`\`mermaid\n${CON_FENCE}\`\`\`\``);
    expect(done).toHaveBeenCalled();
  });

  it("narrows the list by document or kind, and takes the chosen one with the keys", async () => {
    const { put } = opened("diagramFrom");
    const field = await screen.findByLabelText(t("diagramFind"));

    await userEvent.type(field, kindName("erDiagram").slice(0, 6));
    expect(screen.getAllByRole("option")).toHaveLength(1);

    await userEvent.clear(field);
    fireEvent.keyDown(field, { key: "ArrowDown" });
    fireEvent.keyDown(field, { key: "Enter" });
    expect(put).toHaveBeenCalledWith(`\`\`\`mermaid\n${TIENDA}\`\`\``);
  });

  it("is reached from the new diagram menu, and says when there is nothing to reuse", async () => {
    found.all = [];
    opened("diagram");

    await userEvent.click(screen.getByRole("menuitem", { name: t("diagramExisting") }));

    expect(await screen.findByText(t("diagramNoneYet"))).toBeTruthy();
  });

  it("says why when the diagrams cannot be read", async () => {
    found.fails = true;
    const { error } = opened("diagramFrom");

    await waitFor(() => expect(error).toHaveBeenCalled());
    expect(screen.getByText(t("diagramNoneYet"))).toBeTruthy();
  });
});
