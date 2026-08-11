import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, describe, expect, it, vi } from "vitest";
import CaptureField from "../ui/CaptureField";
import type { List } from "../core";

const read = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (name: string, args: Record<string, unknown>) => {
    if (name === "read") return read(args);
    return Promise.resolve(null);
  },
}));

const seen = (title: string, dated: boolean) => ({
  title,
  tags: [],
  spans: dated ? [{ from: 12, to: 18, mark: "date", certainty: "sure" }] : [],
  offers: [],
  date: dated
    ? { at: "2026-08-12T10:00:00", tz: "America/Santiago", floating: true, has_time: false }
    : undefined,
});

const show = () =>
  render(
    <CaptureField
      invite="Add a task"
      lists={[] as List[]}
      tags={[]}
      onCapture={() => Promise.resolve(undefined as never)}
      onError={() => {}}
    />,
  );

describe("what the capture field teaches", () => {
  afterEach(() => vi.useRealTimers());

  /// Writing «every tuesday» is the thing this product does that the clones do
  /// not, and nothing in the window said it was possible.
  it("names the cadence among the things you can write", () => {
    show();

    expect(screen.getByText("every tuesday")).toBeTruthy();
  });

  it("still names the marks it always did", () => {
    show();

    expect(screen.getByText("tag")).toBeTruthy();
    expect(screen.getByText("list")).toBeTruthy();
    expect(screen.getByText("priority")).toBeTruthy();
  });

  /// It used to be replaced by the chips on the first thing that parsed, which
  /// is exactly when someone is still learning what else they could write.
  it("does not vanish the moment something is understood", async () => {
    vi.setSystemTime(new Date("2026-08-11T09:00:00"));
    read.mockResolvedValue(seen("meeting", true));
    show();

    await userEvent.type(screen.getByRole("textbox"), "meeting tomorrow");
    // The chip is what used to push the hint out; wait for it, not for the hint.
    await screen.findAllByRole("button", { name: /tomorrow/i }, { timeout: 2000 });

    expect(screen.getByText("every tuesday")).toBeTruthy();
  });
});
