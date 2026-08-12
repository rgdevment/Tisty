import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import Recall from "../ui/Recall";

/// Choosing a day whose hour has already gone by used to close the sheet and
/// do nothing at all — no reminder, no refusal, nothing in the log. The core
/// refuses it by name; the window has to let it speak.
describe("a reminder picked in the past", () => {
  it("is sent on, so the refusal can be shown", () => {
    const onAdd = vi.fn();
    const onClose = vi.fn();
    render(<Recall on={undefined} taken={[]} onAdd={onAdd} onClose={onClose} />);

    fireEvent.click(screen.getByText("Pick a day and time…"));
    const clock = screen.getByLabelText(/time/i) as HTMLInputElement;
    fireEvent.change(clock, { target: { value: "00:01" } });
    // The first of this month, which has gone by — the case that was swallowed.
    fireEvent.click(screen.getAllByText("1")[0]);
    fireEvent.click(screen.getByText("Remind me then"));

    expect(onAdd).toHaveBeenCalledTimes(1);
    expect(onAdd.mock.calls[0][0]).toMatch(/T00:01:00$/);
  });
});
