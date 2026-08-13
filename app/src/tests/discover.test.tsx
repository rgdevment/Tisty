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

  it("does not vanish the moment something is understood", async () => {
    vi.setSystemTime(new Date("2026-08-11T09:00:00"));
    read.mockResolvedValue(seen("meeting", true));
    show();

    await userEvent.type(screen.getByRole("textbox"), "meeting tomorrow");
    await screen.findAllByRole("button", { name: /tomorrow/i }, { timeout: 2000 });

    expect(screen.getByText("every tuesday")).toBeTruthy();
  });
});

describe("the / menu on a fresh install", () => {
  it("says how a list is made when there are none", async () => {
    const { default: SlashMenu } = await import("../ui/SlashMenu");
    render(
      <SlashMenu
        from="list"
        query=""
        lists={[]}
        tags={[]}
        onDate={() => {}}
        onInsert={() => {}}
        onClose={() => {}}
      />,
    );

    expect(screen.getByRole("status").textContent).toMatch(/No lists yet/i);
  });

  it("says the same for tags", async () => {
    const { default: SlashMenu } = await import("../ui/SlashMenu");
    render(
      <SlashMenu
        from="tag"
        query=""
        lists={[]}
        tags={[]}
        onDate={() => {}}
        onInsert={() => {}}
        onClose={() => {}}
      />,
    );

    expect(screen.getByRole("status").textContent).toMatch(/No tags yet/i);
  });

  it("stays out of the way when the query matches no field", async () => {
    const { default: SlashMenu } = await import("../ui/SlashMenu");
    const { container } = render(
      <SlashMenu
        from={null}
        query="zzzz"
        lists={[]}
        tags={[]}
        onDate={() => {}}
        onInsert={() => {}}
        onClose={() => {}}
      />,
    );

    expect(container.firstChild).toBeNull();
  });
});
