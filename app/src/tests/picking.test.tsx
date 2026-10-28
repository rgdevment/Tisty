import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { nameOf, STARTS, startOf } from "tisty-diagrams/read";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Filed } from "../core";
import { t } from "../locales";
import Picking, { type Picked } from "../ui/Picking";

const made = vi.hoisted(() => ({
  asked: [] as [string, string | undefined, string | undefined][],
  fails: false,
  waits: null as null | (() => void),
}));

vi.mock("../making", () => ({
  spawned: (name: string, folder?: string, pageOf?: string) => {
    made.asked.push([name, folder, pageOf]);
    if (made.fails) return Promise.reject(new Error("refused"));
    return new Promise((done) => {
      const born = () => done({ id: "ab12-0009", said: `![${name}](tisty:doc/ab12-0009)` });
      if (made.waits) made.waits = born;
      else born();
    });
  },
}));

vi.mock("../ui/Glyphs", () => ({
  default: ({ onPick }: { onPick: (key: string, hue?: string) => void }) => (
    <>
      <button type="button" onClick={() => onPick("star", "green")}>
        star
      </button>
      <button type="button" onClick={() => onPick("star")}>
        plain
      </button>
      <button type="button" onClick={() => onPick("😀")}>
        smile
      </button>
    </>
  ),
}));

const book: Filed = {
  id: "01B",
  file: "ab12-0001",
  title: "Libro",
  folder: "01F",
  archived: false,
  away: false,
};
const rina: Filed = { ...book, id: "01R", file: "ab12-0003", title: "Rina", pageOf: "01B" };

const shown = (kind: Picked["kind"], lands = true) => {
  const put = vi.fn((_: string) => lands);
  const heard = { made: vi.fn(), lost: vi.fn(), error: vi.fn(), done: vi.fn() };
  render(
    <Picking
      picked={{ kind, at: { x: 40, y: 40 }, put }}
      own={book}
      pages={[rina]}
      others={[rina]}
      onMade={heard.made}
      onLost={heard.lost}
      onError={heard.error}
      onDone={heard.done}
    />,
  );
  return { put, ...heard };
};

beforeEach(() => {
  made.asked = [];
  made.fails = false;
  made.waits = null;
});

describe("what the / menu asks the window for", () => {
  it("makes a new page of this document and hands its card back to the block", async () => {
    const { put, made: told, done } = shown("newPage");
    await userEvent.type(screen.getByRole("textbox"), "Capítulo uno{enter}");
    await waitFor(() => expect(put).toHaveBeenCalledWith("![Capítulo uno](tisty:doc/ab12-0009)"));
    expect(made.asked).toEqual([["Capítulo uno", "01F", "01B"]]);
    expect(told).toHaveBeenCalledWith({ id: "ab12-0009", title: "Capítulo uno" });
    expect(done).toHaveBeenCalled();
  });

  it("makes a loose document when asked for one, not a page", async () => {
    shown("newDoc");
    await userEvent.type(screen.getByRole("textbox"), "Notas{enter}");
    await waitFor(() => expect(made.asked).toEqual([["Notas", "01F", undefined]]));
  });

  it("puts the card at the end when the block closed while the name was typed", async () => {
    const { lost } = shown("newPage", false);
    await userEvent.type(screen.getByRole("textbox"), "Capítulo{enter}");
    await waitFor(() =>
      expect(lost).toHaveBeenCalledWith({ file: "ab12-0009", title: "Capítulo" }),
    );
  });

  it("makes it once however often the name is sent, and stays open until it is made", async () => {
    made.waits = () => {};
    const { done } = shown("newPage");
    const field = screen.getByRole("textbox");
    await userEvent.type(field, "Uno{enter}");
    fireEvent.submit(field);
    fireEvent.mouseDown(
      document.querySelector("[aria-hidden='true'][data-keeps-writing]") as Element,
    );
    fireEvent.keyDown(screen.getByRole("dialog"), { key: "Escape" });
    expect(made.asked).toHaveLength(1);
    expect(done).not.toHaveBeenCalled();
    made.waits?.();
    await waitFor(() => expect(done).toHaveBeenCalled());
  });

  it("says why when the page could not be made", async () => {
    made.fails = true;
    const { error, done, put } = shown("newPage");
    await userEvent.type(screen.getByRole("textbox"), "Uno{enter}");
    await waitFor(() => expect(error).toHaveBeenCalled());
    expect(done).toHaveBeenCalled();
    expect(put).not.toHaveBeenCalled();
  });

  it("points at a document that already exists, and keeps its card when the block is gone", async () => {
    const { put } = shown("doc");
    await userEvent.click(screen.getByText("Rina"));
    expect(put).toHaveBeenCalledWith("![Rina](tisty:doc/ab12-0003)");

    document.body.innerHTML = "";
    const late = shown("page", false);
    await userEvent.click(screen.getByText("Rina"));
    expect(late.lost).toHaveBeenCalledWith({ file: "ab12-0003", title: "Rina" });
  });

  it("writes Tisty's icons in its own words and an emoji as itself", async () => {
    const { put } = shown("icon");
    await userEvent.click(screen.getByText("star"));
    await userEvent.click(screen.getByText("plain"));
    await userEvent.click(screen.getByText("smile"));
    expect(put.mock.calls.map(([text]) => text)).toEqual(["::star green::", "::star::", "😀"]);
  });

  it("closes on Escape or a press beside it when nothing is on its way", () => {
    const { done } = shown("doc");
    fireEvent.keyDown(screen.getByRole("dialog"), { key: "Escape" });
    fireEvent.mouseDown(
      document.querySelector("[aria-hidden='true'][data-keeps-writing]") as Element,
    );
    expect(done).toHaveBeenCalledTimes(2);
  });
});

describe("a new diagram asked for from the / menu", () => {
  const drawn = (onDrawn?: (source: string, write: () => boolean) => void) => {
    const put = vi.fn((_: string) => true);
    render(
      <Picking
        picked={{ kind: "diagram", at: { x: 40, y: 40 }, put }}
        own={book}
        pages={[]}
        others={[]}
        onMade={vi.fn()}
        onLost={vi.fn()}
        onError={vi.fn()}
        onDone={vi.fn()}
        onDrawn={onDrawn}
      />,
    );
    return put;
  };

  it("offers every kind and hands the window its example with the way to write it", async () => {
    const heard: string[] = [];
    const put = drawn((source, write) => {
      heard.push(source);
      write();
    });

    expect(screen.getAllByRole("menuitem").map((one) => one.textContent)).toEqual([
      ...STARTS.map((one) => nameOf(one, "en")),
      t("diagramExisting"),
    ]);
    await userEvent.click(screen.getByRole("menuitem", { name: nameOf("classDiagram", "en") }));

    expect(heard).toEqual([startOf("classDiagram", "en")]);
    expect(put).toHaveBeenCalledWith(`\`\`\`mermaid\n${startOf("classDiagram", "en")}\`\`\``);
  });

  it("only writes the block where nothing opens the window", async () => {
    const put = drawn();
    await userEvent.click(screen.getByRole("menuitem", { name: nameOf("mindmap", "en") }));

    expect(put).toHaveBeenCalledWith(`\`\`\`mermaid\n${startOf("mindmap", "en")}\`\`\``);
  });
});
