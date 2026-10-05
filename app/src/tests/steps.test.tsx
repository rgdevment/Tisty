import { readFileSync } from "node:fs";
import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { STEP_AT_MOST, type Step } from "../core";
import { t } from "../locales";
import Steps from "../ui/Steps";
import { inTheRepo } from "./repo";

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => Promise.resolve(null) }));

const step = (text: string): Step => ({ id: "s1", text, done: false, order: "a0" });

const shown = (steps: Step[] = [], onWrite = vi.fn()) => {
  render(<Steps steps={steps} onWrite={onWrite} onMark={vi.fn()} onDrop={vi.fn()} />);
  return onWrite;
};

const adding = () => screen.getByLabelText(t("addStep")) as HTMLTextAreaElement;

describe("how long a step is allowed to be", () => {
  it("agrees with the core, which is the one that refuses the step", () => {
    const said = readFileSync(inTheRepo("crates/tisty-core/src/model/task.rs"), "utf8").match(
      /pub const STEP_AT_MOST: usize = (\d+);/,
    );

    expect(said).not.toBeNull();
    expect(Number(said?.[1])).toBe(STEP_AT_MOST);
  });

  it("stops a new step at the limit instead of letting it be refused afterwards", () => {
    shown();
    fireEvent.change(adding(), { target: { value: "x".repeat(STEP_AT_MOST + 10) } });

    expect(adding().value).toBe("x".repeat(STEP_AT_MOST));
  });

  it("counts a character the way the core does, an emoji as one", () => {
    shown();
    fireEvent.change(adding(), { target: { value: "🌱".repeat(STEP_AT_MOST) } });

    expect(Array.from(adding().value)).toHaveLength(STEP_AT_MOST);
    expect(screen.getByRole("status").textContent).toBe("0");
  });

  it("keeps a longer step written before the limit whole while it is read", () => {
    const long = "x".repeat(STEP_AT_MOST + 30);
    shown([step(long)]);

    const field = screen.getByDisplayValue(long) as HTMLTextAreaElement;
    expect(field.value).toBe(long);
    fireEvent.change(field, { target: { value: `${long}y` } });
    expect(field.value).toBe(long);
  });
});

describe("what is left to write", () => {
  it("says nothing while there is plenty of room", () => {
    shown();
    fireEvent.change(adding(), { target: { value: "comprar pan" } });

    expect(screen.queryByRole("status")).toBeNull();
  });

  it("counts down near the limit", () => {
    shown();
    fireEvent.change(adding(), { target: { value: "x".repeat(STEP_AT_MOST - 5) } });

    expect(screen.getByRole("status").textContent).toBe("5");
  });
});

describe("a step is one line", () => {
  it("is added with Enter, without a line break inside it", () => {
    const wrote = shown();
    fireEvent.change(adding(), { target: { value: "llamar al banco" } });
    fireEvent.keyDown(adding(), { key: "Enter" });

    expect(wrote).toHaveBeenCalledWith("llamar al banco");
    expect(adding().value).toBe("");
  });

  it("folds a pasted line break into a space", () => {
    shown();
    fireEvent.change(adding(), { target: { value: "una cosa\notra" } });

    expect(adding().value).toBe("una cosa otra");
  });
});

describe("writing through an input method", () => {
  it("leaves Enter to the input method while a word is still being composed", () => {
    const wrote = shown();
    fireEvent.change(adding(), { target: { value: "にほん" } });
    fireEvent.keyDown(adding(), { key: "Enter", isComposing: true });

    expect(wrote).not.toHaveBeenCalled();
    expect(adding().value).toBe("にほん");
  });
});
