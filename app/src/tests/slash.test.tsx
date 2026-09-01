import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import Slash, { asked, type Block, narrowed } from "../ui/Slash";

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => Promise.resolve(null) }));

describe("when the slash menu should appear", () => {
  it("opens on a slash that starts a word", () => {
    expect(asked("/")).toBe("");
    expect(asked("hola /")).toBe("");
    expect(asked("/tab")).toBe("tab");
    expect(asked("hola /quo")).toBe("quo");
  });

  it("stays shut inside a path or a date, where a slash means itself", () => {
    expect(asked("C:/Users")).toBeNull();
    expect(asked("13/08")).toBeNull();
    expect(asked("https://ejemplo.org")).toBeNull();
    expect(asked("y/o")).toBeNull();
  });

  it("stays shut when there is no slash at all", () => {
    expect(asked("")).toBeNull();
    expect(asked("una nota cualquiera")).toBeNull();
  });

  it("gives up once the word has a space after it", () => {
    expect(asked("/tabla ya")).toBeNull();
  });
});

describe("narrowing the blocks", () => {
  const blocks = [{ label: "Table" }, { label: "Quote" }, { label: "Task list" }];

  it("keeps everything until something is typed", () => {
    expect(narrowed(blocks, "")).toHaveLength(3);
  });

  it("matches without caring about case", () => {
    expect(narrowed(blocks, "TAB")).toEqual([{ label: "Table" }]);
  });

  it("matches anywhere in the name, not only at the start", () => {
    expect(narrowed(blocks, "list")).toEqual([{ label: "Task list" }]);
  });

  it("can come back empty, which is what closes the menu", () => {
    expect(narrowed(blocks, "zzz")).toHaveLength(0);
  });

  it("finds a block by what it is called in the code, whatever it says on screen", () => {
    const spoken = [
      { key: "newpage", label: "Una página nueva" },
      { key: "page", label: "Una página de este" },
      { key: "table", label: "Una tabla" },
    ];

    expect(narrowed(spoken, "page").map((one) => one.key)).toEqual(["newpage", "page"]);
    expect(narrowed(spoken, "table").map((one) => one.key)).toEqual(["table"]);
  });

  it("still finds it by the name the person reads, accents and all", () => {
    const spoken = [{ key: "newpage", label: "Una página nueva" }];

    expect(narrowed(spoken, "pagina")).toHaveLength(1);
    expect(narrowed(spoken, "página")).toHaveLength(1);
  });
});

describe("the slash menu itself", () => {
  const blocks = (run = vi.fn()): Block[] => [
    { key: "quote", label: "Quote", hint: ">", icon: "❝", run },
    { key: "table", label: "Table", hint: "|", icon: "⊞", run },
  ];

  it("marks the one in hand for whoever cannot see it", () => {
    render(<Slash at={{ x: 0, y: 0 }} blocks={blocks()} active={1} onPick={vi.fn()} />);

    const options = screen.getAllByRole("option");
    expect(options[0].getAttribute("aria-selected")).toBe("false");
    expect(options[1].getAttribute("aria-selected")).toBe("true");
  });

  it("shows what each block will write, not only its name", () => {
    render(<Slash at={{ x: 0, y: 0 }} blocks={blocks()} active={0} onPick={vi.fn()} />);

    expect(screen.getByRole("option", { name: /Quote/ }).textContent).toContain(">");
  });

  it("names itself and its options, so the editor can point a reader at them", () => {
    render(<Slash at={{ x: 0, y: 0 }} blocks={blocks()} active={1} onPick={vi.fn()} />);

    expect(screen.getByRole("listbox").id).toBe("slash-menu");
    expect(screen.getAllByRole("option").map((one) => one.id)).toEqual(["slash-0", "slash-1"]);
    expect(document.getElementById("slash-1")?.getAttribute("aria-selected")).toBe("true");
  });

  it("answers to a keyboard, which never sends a mouse press", async () => {
    const onPick = vi.fn();
    render(<Slash at={{ x: 0, y: 0 }} blocks={blocks()} active={0} onPick={onPick} />);

    screen.getByRole("option", { name: /Table/ }).click();

    expect(onPick.mock.calls[0][0].key).toBe("table");
  });

  it("keeps its options out of the tab order, which belongs to the text", () => {
    render(<Slash at={{ x: 0, y: 0 }} blocks={blocks()} active={0} onPick={vi.fn()} />);

    for (const one of screen.getAllByRole("option")) {
      expect(one.getAttribute("tabindex")).toBe("-1");
    }
  });

  it("picks without stealing the cursor from the text", async () => {
    const onPick = vi.fn();
    render(<Slash at={{ x: 0, y: 0 }} blocks={blocks()} active={0} onPick={onPick} />);
    const before = document.activeElement;

    await userEvent.click(screen.getByRole("option", { name: /Table/ }));

    expect(onPick.mock.calls[0][0].key).toBe("table");
    expect(document.activeElement).toBe(before);
  });
});
