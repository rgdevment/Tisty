import { fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import Modal from "../ui/Modal";
import Sure from "../ui/Sure";

const ask = () => {
  const onAnswer = vi.fn();
  render(<Sure said="Seguro?" buttons={{ yes: "Si", no: "No" }} onAnswer={onAnswer} />);
  return { onAnswer, yes: screen.getByRole("button", { name: "Si" }) };
};

describe("the question that asks if you are sure", () => {
  it("takes the focus on the confirm button when it opens", () => {
    const { yes } = ask();

    expect(document.activeElement).toBe(yes);
  });

  it("answers yes on Enter", async () => {
    const { onAnswer } = ask();

    await userEvent.keyboard("{Enter}");

    expect(onAnswer).toHaveBeenCalledWith(true);
  });

  it("answers no on Escape", async () => {
    const { onAnswer } = ask();

    await userEvent.keyboard("{Escape}");

    expect(onAnswer).toHaveBeenCalledWith(false);
  });

  it("answers nothing to an Enter that is still held down", () => {
    const { onAnswer, yes } = ask();

    const passed = fireEvent.keyDown(yes, { key: "Enter", repeat: true });

    expect(passed).toBe(false);
    expect(onAnswer).not.toHaveBeenCalled();
  });

  it("lets a first Enter through", () => {
    const { yes } = ask();

    expect(fireEvent.keyDown(yes, { key: "Enter" })).toBe(true);
  });
});

describe("other dialogs", () => {
  it("still focus their first button", () => {
    render(
      <Modal title="Largo">
        <button type="button">Uno</button>
        <button type="button">Dos</button>
      </Modal>,
    );

    expect(document.activeElement).toBe(screen.getByRole("button", { name: "Uno" }));
  });
});
