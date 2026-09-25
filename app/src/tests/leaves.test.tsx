import { fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import type { Filed } from "../core";
import Contents from "../ui/Contents";
import Ribbon, { Onward } from "../ui/Ribbon";

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => Promise.resolve(null) }));

const page = (id: string, file: string, title: string): Filed => ({
  id,
  file,
  title,
  folder: null,
  archived: false,
  away: false,
  pageOf: "01A",
});

const pages = [
  page("01B", "a3f1-0002", "El pod"),
  page("01C", "a3f1-0003", "El túnel"),
  page("01D", "a3f1-0004", "La VPN cae"),
];

describe("the index at the end of a document", () => {
  const show = (told: string[]) => {
    const onOpen = vi.fn();
    const onPut = vi.fn();
    render(<Contents pages={pages} told={told} onOpen={onOpen} onPut={onPut} />);
    return { onOpen, onPut };
  };

  const dragged = (told: string[]) => {
    const onMove = vi.fn();
    render(<Contents pages={pages} told={told} onOpen={vi.fn()} onPut={vi.fn()} onMove={onMove} />);
    return onMove;
  };

  const carried = (told: string[]) => {
    const onMove = vi.fn();
    const { container } = render(
      <Contents pages={pages} told={told} onOpen={vi.fn()} onPut={vi.fn()} onMove={onMove} />,
    );
    return { onMove, container };
  };

  it("asks for the page to go last when it is dropped past the last row", () => {
    const { onMove, container } = carried(["a3f1-0002", "a3f1-0003", "a3f1-0004"]);
    const rows = screen.getAllByRole("listitem");
    const end = container.querySelector(".leaf-end");
    if (!end) throw new Error("the list ends with nowhere to drop");

    fireEvent.dragStart(rows[0].querySelector("[data-leaf]") as HTMLElement);
    fireEvent.dragOver(end);
    fireEvent.drop(end);

    expect(onMove).toHaveBeenCalledTimes(1);
    expect(onMove.mock.calls[0][0].file).toBe("a3f1-0002");
    expect(onMove.mock.calls[0][1]).toBeNull();
  });

  it("stops marking a row once the page is carried off it", () => {
    const { container } = carried(["a3f1-0002", "a3f1-0003", "a3f1-0004"]);
    const rows = screen.getAllByRole("listitem");

    fireEvent.dragStart(rows[2].querySelector("[data-leaf]") as HTMLElement);
    fireEvent.dragOver(rows[0]);
    expect(rows[0].className).toContain("leaf-over");

    fireEvent.dragLeave(rows[0]);
    expect(rows[0].className).not.toContain("leaf-over");

    const end = container.querySelector(".leaf-end");
    if (!end) throw new Error("the list ends with nowhere to drop");
    fireEvent.dragOver(end);
    expect(end.className).toContain("leaf-over");
    fireEvent.dragLeave(end);
    expect(end.className).not.toContain("leaf-over");
  });

  it("lets go of what it was carrying when the drag ends anywhere", () => {
    const { container } = carried(["a3f1-0002", "a3f1-0003", "a3f1-0004"]);
    const rows = screen.getAllByRole("listitem");

    fireEvent.dragStart(rows[2].querySelector("[data-leaf]") as HTMLElement);
    fireEvent.dragOver(rows[0]);
    fireEvent.dragEnd(rows[2].querySelector("[data-leaf]") as HTMLElement);

    expect(rows[0].className).not.toContain("leaf-over");
    const end = container.querySelector(".leaf-end");
    if (!end) throw new Error("the list ends with nowhere to drop");
    fireEvent.dragOver(end);
    expect(end.className).not.toContain("leaf-over");
  });

  it("offers nowhere to drop past the last row when one page alone is named", () => {
    const { container } = carried(["a3f1-0002"]);

    expect(container.querySelector(".leaf-end")).toBeNull();
  });

  it("asks for the page to go before the one it was dropped on", () => {
    const onMove = dragged(["a3f1-0002", "a3f1-0003", "a3f1-0004"]);
    const rows = screen.getAllByRole("listitem");

    fireEvent.dragStart(rows[2].querySelector("[data-leaf]") as HTMLElement);
    fireEvent.dragOver(rows[0]);
    fireEvent.drop(rows[0]);

    expect(onMove).toHaveBeenCalledTimes(1);
    expect(onMove.mock.calls[0][0].file).toBe("a3f1-0004");
    expect(onMove.mock.calls[0][1]).toBe("a3f1-0002");
  });

  it("asks nothing when a page is dropped on itself", () => {
    const onMove = dragged(["a3f1-0002", "a3f1-0003", "a3f1-0004"]);
    const rows = screen.getAllByRole("listitem");

    fireEvent.dragStart(rows[1].querySelector("[data-leaf]") as HTMLElement);
    fireEvent.drop(rows[1]);

    expect(onMove).not.toHaveBeenCalled();
  });

  it("does not let a page the text never names be dragged", () => {
    dragged(["a3f1-0002", "a3f1-0003"]);
    const rows = screen.getAllByRole("listitem");

    expect(rows[0].querySelector("[data-leaf]")?.getAttribute("draggable")).toBe("true");
    expect(rows[2].querySelector("[data-leaf]")?.getAttribute("draggable")).toBe("false");
  });

  it("numbers the pages the text names and leaves the rest without a number", () => {
    show(["a3f1-0002", "a3f1-0003"]);
    const rows = screen.getAllByRole("listitem");

    expect(rows).toHaveLength(3);
    expect(rows.map((one) => one.textContent)).toEqual([
      "01El pod",
      "02El túnel",
      "—La VPN caePut it in the text",
    ]);
  });

  it("offers to put a loose page in the text, and only a loose one", () => {
    show(["a3f1-0002", "a3f1-0003"]);

    expect(screen.getAllByRole("button", { name: "Put it in the text" })).toHaveLength(1);
  });

  it("puts the page the person picked, not another", async () => {
    const { onPut } = show([]);
    await userEvent.click(screen.getAllByRole("button", { name: "Put it in the text" })[1]);

    expect(onPut).toHaveBeenCalledWith(pages[1]);
  });

  it("opens the page from its row", async () => {
    const { onOpen } = show(["a3f1-0002"]);
    await userEvent.click(screen.getByText("El pod"));

    expect(onOpen).toHaveBeenCalledWith(pages[0]);
  });

  it("says nothing at all when the document holds no pages", () => {
    const { container } = render(
      <Contents pages={[]} told={[]} onOpen={vi.fn()} onPut={vi.fn()} />,
    );

    expect(container.innerHTML).toBe("");
  });
});

describe("the head of a page", () => {
  const of: Filed = {
    id: "01A",
    file: "a3f1-0001",
    title: "Bases de datos",
    folder: null,
    archived: false,
    away: false,
  };

  const told = pages.map((one) => one.file);

  const show = (here: string) => {
    const onOpen = vi.fn();
    render(<Ribbon of={of} sisters={pages} told={told} here={here} onOpen={onOpen} />);
    return { onOpen };
  };

  it("names the document it belongs to and where it sits among its sisters", () => {
    show("a3f1-0003");

    expect(screen.getByText("Bases de datos")).toBeTruthy();
    expect(screen.getByText("Page 2 of 3")).toBeTruthy();
  });
  it("walks the sisters in the order the text names them, not the order they arrived", () => {
    const onOpen = vi.fn();
    render(
      <Ribbon
        of={of}
        sisters={pages}
        told={["a3f1-0004", "a3f1-0002", "a3f1-0003"]}
        here="a3f1-0002"
        onOpen={onOpen}
      />,
    );

    expect(screen.getByText("Page 2 of 3")).toBeTruthy();
  });

  it("says when the page it would open next is one the archive holds", () => {
    const shelved: Filed[] = pages.map((one, at) =>
      at === 2 ? { ...one, archived: true, away: true } : one,
    );
    render(
      <Ribbon
        of={of}
        sisters={shelved}
        told={shelved.map((one) => one.file)}
        here={shelved[1].file}
        onOpen={vi.fn()}
      />,
    );

    const on = screen.getAllByRole("button", { name: "Page after" }).slice(-1)[0];
    expect(on.getAttribute("title")).toContain("in the archive");
  });

  it("goes back to the document from its name", async () => {
    const { onOpen } = show("a3f1-0003");
    await userEvent.click(screen.getByText("Bases de datos"));

    expect(onOpen).toHaveBeenCalledWith(of);
  });

  it("has nowhere to go back to on the first page, and nowhere on from the last", () => {
    show("a3f1-0002");
    expect(screen.getByRole<HTMLButtonElement>("button", { name: "Page before" }).disabled).toBe(
      true,
    );

    render(<Ribbon of={of} sisters={pages} told={told} here="a3f1-0004" onOpen={vi.fn()} />);
    expect(
      screen.getAllByRole<HTMLButtonElement>("button", { name: "Page after" })[1].disabled,
    ).toBe(true);
  });

  it("gives a page its document never names no number and nowhere to step", () => {
    render(
      <Ribbon of={of} sisters={pages} told={["a3f1-0002"]} here="a3f1-0003" onOpen={vi.fn()} />,
    );

    expect(screen.getByText("Loose page")).toBeTruthy();
    expect(screen.getByRole<HTMLButtonElement>("button", { name: "Page after" }).disabled).toBe(
      true,
    );
  });

  it("steps to the sister the arrow points at", async () => {
    const { onOpen } = show("a3f1-0003");
    await userEvent.click(screen.getByRole("button", { name: "Page after" }));

    expect(onOpen).toHaveBeenCalledWith(pages[2]);
  });
});

describe("the step at the foot of a page", () => {
  it("names the one that follows and opens it", async () => {
    const onOpen = vi.fn();
    render(<Onward next={pages[1]} onOpen={onOpen} />);
    await userEvent.click(screen.getByText("El túnel"));

    expect(onOpen).toHaveBeenCalledWith(pages[1]);
  });
});

describe("moving a chapter one place at a time", () => {
  const shown = (told: string[]) => {
    const onMove = vi.fn();
    const { container } = render(
      <Contents pages={pages} told={told} onOpen={vi.fn()} onPut={vi.fn()} onMove={onMove} />,
    );
    const rows = () => [...container.querySelectorAll<HTMLElement>("[data-leaf]")];
    const arrow = (file: string, by: -1 | 1) =>
      container.querySelector<HTMLButtonElement>(`[data-move="${file}:${by}"]`);
    return { onMove, rows, arrow, container };
  };

  const all = ["a3f1-0002", "a3f1-0003", "a3f1-0004"];

  it("tells every chapter which keys move it", () => {
    const { rows } = shown(all);

    for (const one of rows()) {
      expect(one.getAttribute("aria-keyshortcuts")).toBe("Alt+ArrowUp Alt+ArrowDown");
    }
  });

  it("keeps the buttons outside the part that drags, so a press on one is a press", () => {
    const { arrow, container } = shown(all);
    const down = arrow("a3f1-0002", 1);
    if (!down) throw new Error("no way down");

    expect(down.closest('[draggable="true"]')).toBeNull();
    expect(container.querySelector('[data-leaf="a3f1-0002"]')?.getAttribute("draggable")).toBe(
      "true",
    );
  });

  it("offers a button for each way a chapter can go, and none off the ends", () => {
    const { arrow } = shown(all);

    expect(arrow("a3f1-0002", -1)?.disabled).toBe(true);
    expect(arrow("a3f1-0002", 1)?.disabled).toBe(false);
    expect(arrow("a3f1-0004", -1)?.disabled).toBe(false);
    expect(arrow("a3f1-0004", 1)?.disabled).toBe(true);
  });

  it("moves a chapter from the button, with no keyboard at all", () => {
    const { onMove, arrow } = shown(all);
    const down = arrow("a3f1-0002", 1);
    if (!down) throw new Error("no way down");

    fireEvent.click(down);

    expect(onMove).toHaveBeenCalledTimes(1);
    expect(onMove.mock.calls[0][0].file).toBe("a3f1-0002");
    expect(onMove.mock.calls[0][1]).toBe("a3f1-0004");
  });

  it("names each button for what it does to which chapter", () => {
    const { arrow } = shown(all);

    expect(arrow("a3f1-0003", -1)?.getAttribute("aria-label")).toContain("El túnel");
    expect(arrow("a3f1-0003", 1)?.getAttribute("aria-label")).toContain("El túnel");
  });

  it("takes a chapter up one place, before the one above it", () => {
    const { onMove, rows } = shown(all);

    fireEvent.keyDown(rows()[2], { key: "ArrowUp", altKey: true });

    expect(onMove).toHaveBeenCalledTimes(1);
    expect(onMove.mock.calls[0][0].file).toBe("a3f1-0004");
    expect(onMove.mock.calls[0][1]).toBe("a3f1-0003");
  });

  it("takes a chapter down one place, before the one after the next", () => {
    const { onMove, rows } = shown(all);

    fireEvent.keyDown(rows()[0], { key: "ArrowDown", altKey: true });

    expect(onMove).toHaveBeenCalledTimes(1);
    expect(onMove.mock.calls[0][0].file).toBe("a3f1-0002");
    expect(onMove.mock.calls[0][1]).toBe("a3f1-0004");
  });

  it("sends the last chapter down to nowhere in particular, which is where last is", () => {
    const { onMove, rows } = shown(all);

    fireEvent.keyDown(rows()[1], { key: "ArrowDown", altKey: true });

    expect(onMove.mock.calls[0][0].file).toBe("a3f1-0003");
    expect(onMove.mock.calls[0][1]).toBeNull();
  });

  it("moves nothing off either end", () => {
    const { onMove, rows } = shown(all);

    fireEvent.keyDown(rows()[0], { key: "ArrowUp", altKey: true });
    fireEvent.keyDown(rows()[2], { key: "ArrowDown", altKey: true });

    expect(onMove).not.toHaveBeenCalled();
  });

  it("says where the chapter went, in a region that was there all along", () => {
    const { rows } = shown(all);
    const aloud = screen.getByRole("status");
    expect(aloud.textContent).toBe("");

    fireEvent.keyDown(rows()[0], { key: "ArrowDown", altKey: true });

    expect(aloud.textContent).toContain("El pod");
    expect(aloud.textContent).toContain("2");
  });

  it("leaves the arrows alone when they come without Alt", () => {
    const { onMove, rows } = shown(all);

    fireEvent.keyDown(rows()[0], { key: "ArrowDown" });

    expect(onMove).not.toHaveBeenCalled();
  });

  it("offers no keys, and moves nothing, on a page the text never names", () => {
    const { onMove } = shown(["a3f1-0002"]);
    const rows = screen.getAllByRole("listitem");
    const loose = rows[1].querySelector("button");
    if (!loose) throw new Error("the loose row has no button");

    expect(loose.getAttribute("aria-keyshortcuts")).toBeNull();
    fireEvent.keyDown(loose, { key: "ArrowUp", altKey: true });

    expect(onMove).not.toHaveBeenCalled();
  });

  it("numbers the chapters in the order the text names them, not the order they arrived", () => {
    shown(["a3f1-0004", "a3f1-0002", "a3f1-0003"]);

    expect(
      screen.getAllByRole("listitem").map((one) => one.textContent?.replace(/\s+/g, " ").trim()),
    ).toEqual(["01La VPN cae↑↓", "02El pod↑↓", "03El túnel↑↓"]);
  });

  it("promises only what is on the row, and names no key it cannot offer", () => {
    shown(all);

    const said = screen.getByText(/To move one/).textContent ?? "";
    expect(said).toContain("arrows on its row");
    expect(said).not.toMatch(/Alt|drag/);
  });
});
