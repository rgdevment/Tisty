import { fireEvent, render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import Notes from "../ui/Notes";

const BODY =
  "---\nview: notes\n---\n\n# Ideas\n\n## 2026-11-10 18:40\n\nprimera\n\n## 2026-11-11 09:12\n\nsegunda\n";

describe("a brainstorm shown as notes", () => {
  it("puts the newest idea first", () => {
    render(
      <Notes title="Ideas" body={BODY} reading={false} onWrite={() => {}} onText={() => {}} />,
    );

    const notes = within(screen.getByRole("list")).getAllByRole("listitem");
    expect(notes[0].textContent).toContain("segunda");
    expect(notes[1].textContent).toContain("primera");
  });

  it("writes a note back into its own section when it loses the focus", () => {
    const onWrite = vi.fn();
    render(<Notes title="Ideas" body={BODY} reading={false} onWrite={onWrite} onText={() => {}} />);

    const [newest] = screen.getAllByRole("textbox");
    fireEvent.change(newest, { target: { value: "segunda, mejorada" } });
    fireEvent.blur(newest);

    expect(onWrite).toHaveBeenCalledTimes(1);
    const written: string = onWrite.mock.calls[0][0];
    expect(written).toContain("## 2026-11-11 09:12\n\nsegunda, mejorada");
    expect(written).toContain("## 2026-11-10 18:40\n\nprimera");
  });

  it("does not write when nothing changed", () => {
    const onWrite = vi.fn();
    render(<Notes title="Ideas" body={BODY} reading={false} onWrite={onWrite} onText={() => {}} />);

    fireEvent.blur(screen.getAllByRole("textbox")[0]);

    expect(onWrite).not.toHaveBeenCalled();
  });

  it("removes one idea and keeps the other", async () => {
    const onWrite = vi.fn();
    render(<Notes title="Ideas" body={BODY} reading={false} onWrite={onWrite} onText={() => {}} />);

    await userEvent.click(screen.getAllByRole("button", { name: "Remove this idea" })[0]);

    const written: string = onWrite.mock.calls[0][0];
    expect(written).not.toContain("segunda");
    expect(written).toContain("primera");
  });

  it("only shows the ideas when the document is read-only", () => {
    render(<Notes title="Ideas" body={BODY} reading onWrite={() => {}} onText={() => {}} />);

    expect(screen.queryAllByRole("textbox")).toEqual([]);
    expect(screen.queryByRole("button", { name: "Remove this idea" })).toBeNull();
    expect(screen.queryByRole("button", { name: "New idea" })).toBeNull();
    expect(screen.getByText("primera")).toBeTruthy();
  });

  it("moves the focused idea one place with Alt and an arrow", async () => {
    const onWrite = vi.fn();
    render(<Notes title="Ideas" body={BODY} reading={false} onWrite={onWrite} onText={() => {}} />);

    fireEvent.keyDown(screen.getAllByRole("textbox")[0], { key: "ArrowRight", altKey: true });

    const written: string = onWrite.mock.calls[0][0];
    expect(written.indexOf("segunda")).toBeLessThan(written.indexOf("primera"));
  });
});
