import { render, screen } from "@testing-library/react";
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
    render(<Contents pages={pages} told={new Set(told)} onOpen={onOpen} onPut={onPut} />);
    return { onOpen, onPut };
  };

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
      <Contents pages={[]} told={new Set()} onOpen={vi.fn()} onPut={vi.fn()} />,
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
  };

  const told = new Set(pages.map((one) => one.file));

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
      <Ribbon
        of={of}
        sisters={pages}
        told={new Set(["a3f1-0002"])}
        here="a3f1-0003"
        onOpen={vi.fn()}
      />,
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
