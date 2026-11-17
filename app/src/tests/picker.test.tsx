import { fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState } from "react";
import { describe, expect, it, vi } from "vitest";
import { optionOf, type Pickable, PickList, usePicker } from "../ui/Picker";

const row = (key: string, group?: string, onPick = vi.fn()): Pickable => ({
  key,
  label: `row ${key}`,
  group,
  onPick,
});

// The way `[[` will hold it: the words are typed somewhere else and the keys are handed over.
function Held({ rows, onClose = vi.fn() }: { rows: Pickable[]; onClose?: () => void }) {
  const [asked, setAsked] = useState("");
  const { on, setAt, keyed } = usePicker(rows, asked, onClose);
  return (
    <>
      <textarea
        aria-label="words"
        aria-activedescendant={optionOf("picked", on)}
        value={asked}
        onChange={(event) => setAsked(event.target.value)}
        onKeyDown={(event) => keyed(event)}
      />
      <PickList id="picked" label="choices" rows={rows} on={on} onHover={setAt} />
    </>
  );
}

const chosen = () =>
  screen
    .getAllByRole("option")
    .filter((one) => one.getAttribute("aria-selected") === "true")
    .map((one) => one.textContent);

describe("one picker for every place that asks for a choice", () => {
  it("starts on the first row and walks with the arrows, round the ends", async () => {
    render(<Held rows={[row("a"), row("b"), row("c")]} />);
    const words = screen.getByRole("textbox", { name: "words" });
    words.focus();

    expect(chosen()).toEqual(["row a"]);
    await userEvent.keyboard("{ArrowDown}{ArrowDown}");
    expect(chosen()).toEqual(["row c"]);
    await userEvent.keyboard("{ArrowDown}");
    expect(chosen()).toEqual(["row a"]);
    await userEvent.keyboard("{ArrowUp}");
    expect(chosen()).toEqual(["row c"]);
  });

  it("picks the chosen row with Enter and puts itself away with Esc", async () => {
    const second = vi.fn();
    const shut = vi.fn();
    render(<Held rows={[row("a"), row("b", undefined, second)]} onClose={shut} />);
    screen.getByRole("textbox", { name: "words" }).focus();

    await userEvent.keyboard("{ArrowDown}{Enter}");
    expect(second).toHaveBeenCalledOnce();

    await userEvent.keyboard("{Escape}");
    expect(shut).toHaveBeenCalledOnce();
  });

  it("goes back to the first row when the words change", async () => {
    render(<Held rows={[row("a"), row("b")]} />);
    screen.getByRole("textbox", { name: "words" }).focus();

    await userEvent.keyboard("{ArrowDown}");
    expect(chosen()).toEqual(["row b"]);
    await userEvent.keyboard("x");
    expect(chosen()).toEqual(["row a"]);
  });

  it("names the chosen row to whoever holds the words", async () => {
    render(<Held rows={[row("a"), row("b")]} />);
    const words = screen.getByRole("textbox", { name: "words" });
    words.focus();

    await userEvent.keyboard("{ArrowDown}");
    const named = words.getAttribute("aria-activedescendant") ?? "";
    expect(document.getElementById(named)?.textContent).toBe("row b");
  });

  it("picks on a click and leaves the focus where the words are", async () => {
    const picked = vi.fn();
    render(<Held rows={[row("a"), row("b", undefined, picked)]} />);
    const words = screen.getByRole("textbox", { name: "words" });
    words.focus();

    await userEvent.click(screen.getByRole("option", { name: "row b" }));

    expect(picked).toHaveBeenCalledOnce();
    expect(document.activeElement).toBe(words);
  });

  it("follows the pointer", () => {
    render(<Held rows={[row("a"), row("b")]} />);

    fireEvent.mouseMove(screen.getByRole("option", { name: "row b" }));

    expect(chosen()).toEqual(["row b"]);
  });

  it("heads each run of rows with its group once", () => {
    render(<Held rows={[row("a", "First"), row("b", "First"), row("c", "Second")]} />);

    expect(screen.getAllByText("First")).toHaveLength(1);
    expect(screen.getAllByText("Second")).toHaveLength(1);
  });

  it("leaves the keys it does not use to whoever holds the words", () => {
    render(<Held rows={[row("a")]} />);
    const words = screen.getByRole("textbox", { name: "words" });

    const passed = fireEvent.keyDown(words, { key: "ArrowLeft" });

    expect(passed).toBe(true);
  });
});
