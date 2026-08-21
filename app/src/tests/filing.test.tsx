import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { List, Task } from "../core";
import Fields from "../ui/Fields";

function put(lists: List[]) {
  const onPatch = vi.fn();
  const one = {
    id: "t1",
    title: "revisar el deploy",
    status: "open",
    priority: "unset",
    order: "a0",
  } as Task;
  render(<Fields task={one} lists={lists} known={[]} onPatch={onPatch} />);
  fireEvent.click(screen.getByText(/@ list/i));
  return onPatch;
}

describe("filing a task with no lists yet", () => {
  it("offers to name one", () => {
    const onPatch = put([]);

    const box = screen.getByLabelText("New list…");
    fireEvent.change(box, { target: { value: "Trabajo" } });
    fireEvent.submit(box);

    expect(onPatch).toHaveBeenCalledWith({ listNamed: "Trabajo" });
  });

  it("keeps the case it was typed in, because a list is a name", () => {
    const onPatch = put([]);

    const box = screen.getByLabelText("New list…");
    fireEvent.change(box, { target: { value: "  Casa Nueva  " } });
    fireEvent.submit(box);

    expect(onPatch).toHaveBeenCalledWith({ listNamed: "Casa Nueva" });
  });

  it("still lists the ones already made", () => {
    put([{ id: "l1", name: "Trabajo" } as List]);

    expect(screen.getByText("Trabajo")).toBeTruthy();
    expect(screen.getByLabelText("New list…")).toBeTruthy();
  });
});
