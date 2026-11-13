import { afterEach, describe, expect, it, vi } from "vitest";
import { edited } from "../ui/hosting";

afterEach(() => {
  document.body.innerHTML = "";
});

const opened = (doc: string) => {
  const page = document.createElement("article");
  page.className = "page";
  const block = document.createElement("p");
  page.append(block);
  document.body.append(page);
  const change = vi.fn();
  const leave = vi.fn();
  const editing = edited(block, doc, change, leave, undefined, false, "live");
  return { editing, change, leave };
};

describe("the whole-document editor hands its words back", () => {
  it("once they rest, not on every key, and at once when asked", () => {
    const { editing, change } = opened("one");
    editing.view.dispatch({ changes: { from: 3, insert: " two" }, userEvent: "input.type" });
    editing.view.dispatch({ changes: { from: 7, insert: "!" }, userEvent: "input.type" });
    expect(change).not.toHaveBeenCalled();
    editing.flush?.();
    expect(change).toHaveBeenCalledTimes(1);
    expect(change).toHaveBeenLastCalledWith("one two!");
    editing.leave();
  });

  it("after a short rest by itself", async () => {
    const { editing, change } = opened("one");
    editing.view.dispatch({ changes: { from: 3, insert: "s" }, userEvent: "input.type" });
    await vi.advanceTimersByTimeAsync(200);
    expect(change).toHaveBeenLastCalledWith("ones");
    editing.leave();
  });

  it("leaves a document only looked at as it came, empty steps and all", () => {
    const { editing, leave } = opened("- [ ] \n- [ ] keep");
    editing.leave();
    expect(leave.mock.calls[0][0]).toBe("- [ ] \n- [ ] keep");
  });

  it("tidies a document written in, as a block always was, and hands back the last words", () => {
    const { editing, change, leave } = opened("- [ ] \n- [ ] keep");
    editing.view.dispatch({ changes: { from: 17, insert: "s" }, userEvent: "input.type" });
    editing.leave();
    expect(leave.mock.calls[0][0]).toBe("- [ ] keeps");
    expect(change).toHaveBeenLastCalledWith("- [ ] keeps");
  });
});
