import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Filed, IdeaMarked, Papers } from "../core";
import { asksSure } from "../sure";
import Notes from "../ui/Notes";
import Tree from "../ui/Tree";
import { knowAgents } from "../who";

const ipc = vi.hoisted(() => ({ calls: [] as { cmd: string; args: Record<string, unknown> }[] }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    ipc.calls.push({ cmd, args: args ?? {} });
    return Promise.resolve(null);
  },
}));

vi.mock("@tauri-apps/api/event", () => ({ emit: () => Promise.resolve() }));

const CONCERT = "2026-06-14 08:12 · pink";
const ROOF = "2026-09-03 18:05 · teal";
const BREAD = "2026-11-12 09:40 · yellow";

const BODY = `---\nview: notes\nbrainstorm: on\n---\n\n# Lluvia de ideas\n\n> [!IDEA] ${CONCERT}\n> Comprar entradas del concierto de julio\n\n> [!IDEA] ${ROOF}\n> Pedir presupuesto para arreglar el techo\n\n> [!IDEA] ${BREAD}\n> Probar pan con masa madre\n`;

const marked = (heading: string, text: string, reason: string, instead?: string): IdeaMarked => ({
  heading,
  text,
  mark: { at: "2026-11-12T13:31:00Z", said: "Ya pasó.", via: "claude-code", reason, instead },
});

const sheet = (ideasFlagged: IdeaMarked[]): Filed[] => [
  {
    id: "01S",
    file: "a3f1-0001",
    title: "Lluvia de ideas",
    folder: null,
    archived: false,
    away: false,
    ideasFlagged,
  },
  { id: "01H", file: "a3f1-0002", title: "Casa", folder: null, archived: false, away: false },
];

const BOTH = [
  marked(CONCERT, "Comprar entradas del concierto de julio", "obsolete"),
  marked(ROOF, "Pedir presupuesto para arreglar el techo", "moved", "a3f1-0002"),
];

beforeEach(() => {
  ipc.calls = [];
  asksSure(null);
  knowAgents({}, { clients: { "claude-code": "Claude Code" } });
});

const shown = (marks: IdeaMarked[], onWrite = vi.fn(), onMarks = vi.fn()) => {
  render(
    <Notes
      title="Lluvia de ideas"
      body={BODY}
      reading={false}
      onWrite={onWrite}
      known={sheet(marks)}
      own="a3f1-0001"
      onMarks={onMarks}
    />,
  );
  return { onWrite, onMarks };
};

const noteSaying = (words: string) =>
  within(screen.getByRole("list"))
    .getAllByRole("listitem")
    .find((one) => one.textContent?.includes(words)) as HTMLElement;

describe("an idea an agent marked", () => {
  it("wears the diamond and its reason, and an unmarked idea keeps its tools", () => {
    shown(BOTH);

    const concert = noteSaying("Comprar entradas");
    expect(concert.hasAttribute("data-marked")).toBe(true);
    expect(
      within(concert).getByTitle("Claude Code says this idea has had its day").textContent,
    ).toBe("◆");
    expect(concert.textContent).toContain("Out of date");
    expect(concert.textContent).toContain("Ya pasó.");
    expect(
      within(noteSaying("Pedir presupuesto")).getByRole("button", { name: "Casa" }),
    ).toBeTruthy();

    const bread = noteSaying("Probar pan");
    expect(bread.hasAttribute("data-marked")).toBe(false);
    expect(within(bread).getByRole("button", { name: "Remove this idea" })).toBeTruthy();
    expect(screen.getByText("Claude Code says 2 ideas have had their day")).toBeTruthy();
  });

  it("loses only its mark when the person keeps it", async () => {
    const { onWrite, onMarks } = shown(BOTH);

    await userEvent.click(
      within(noteSaying("Comprar entradas")).getByRole("button", { name: "Remove the mark" }),
    );

    await waitFor(() => expect(onMarks).toHaveBeenCalled());
    expect(ipc.calls).toEqual([
      {
        cmd: "idea_unflag",
        args: {
          file: "a3f1-0001",
          heading: CONCERT,
          text: "Comprar entradas del concierto de julio",
        },
      },
    ]);
    expect(onWrite).not.toHaveBeenCalled();
  });

  it("leaves the sheet, mark and all, when the person deletes it", async () => {
    const { onWrite } = shown(BOTH);

    await userEvent.click(
      within(noteSaying("Comprar entradas")).getByRole("button", { name: "Delete it" }),
    );

    const written: string = onWrite.mock.calls[0][0];
    expect(written).not.toContain("Comprar entradas");
    expect(written).toContain("Pedir presupuesto");
    expect(written).toContain("Probar pan");
    await waitFor(() => expect(ipc.calls.map((one) => one.cmd)).toEqual(["idea_unflag"]));
  });

  it("deletes every marked idea in one click once the person agrees", async () => {
    const asked: string[] = [];
    asksSure((said) => {
      asked.push(said);
      return Promise.resolve(true);
    });
    const { onWrite } = shown(BOTH);

    await userEvent.click(screen.getByRole("button", { name: "Delete all 2" }));

    await waitFor(() => expect(onWrite).toHaveBeenCalledTimes(1));
    expect(asked[0]).toContain("«Pedir presupuesto para arreglar el techo»");
    const written: string = onWrite.mock.calls[0][0];
    expect(written).not.toContain("Comprar entradas");
    expect(written).not.toContain("Pedir presupuesto");
    expect(written).toContain(`> [!IDEA] ${BREAD}\n> Probar pan con masa madre\n`);
  });

  it("names the idea it repeats, and a mark on an idea since changed shows nowhere", () => {
    shown([
      marked(
        CONCERT,
        "Comprar entradas del concierto de julio",
        "duplicate_of",
        `a3f1-0001#${BREAD}`,
      ),
      marked(ROOF, "otras palabras", "obsolete"),
    ]);

    expect(noteSaying("Comprar entradas").textContent).toContain(
      "A copy of «Probar pan con masa madre»",
    );
    expect(noteSaying("Pedir presupuesto").hasAttribute("data-marked")).toBe(false);
    expect(screen.getByText("Claude Code says one idea has had its day")).toBeTruthy();
  });
});

describe("Brainstorm in the Library", () => {
  it("wears the diamond beside its name when an agent marked some of its ideas", () => {
    const papers: Papers = { folders: [], docs: sheet([]) };
    render(
      <Tree
        papers={papers}
        pinned="a3f1-0001"
        ideas={3}
        ideasMarked={2}
        onOpen={() => {}}
        onFile={() => {}}
      />,
    );

    expect(screen.getByTitle("An agent says 2 ideas have had their day").textContent).toBe("◆");
  });
});
