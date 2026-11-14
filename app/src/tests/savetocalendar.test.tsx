import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { offered } from "../calendaring";
import type { Task } from "../core";
import Fields from "../ui/Fields";

const ipc = vi.hoisted(() => ({
  sent: [] as { cmd: string; args?: Record<string, unknown> }[],
  chosen: null as string | null,
  fails: false,
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    ipc.sent.push({ cmd, args });
    if (cmd === "save_to_calendar") {
      if (ipc.fails) return Promise.reject({ code: "cannotOpen", name: "x" });
      return Promise.resolve(ipc.chosen !== null);
    }
    if (cmd === "keep_calendar") {
      ipc.chosen = (args?.calendar as string | undefined) ?? null;
      return Promise.resolve(ipc.chosen);
    }
    return Promise.resolve(null);
  },
}));

const day = { at: "2026-11-20T00:00:00", has_time: false, tz: "America/Santiago", floating: true };

function shown(over: Partial<Task> = {}, onError = vi.fn()) {
  const task = {
    id: "01KA7B3VJ9RZ0Q8M5N2X4Y6W1T",
    title: "Walk Rina",
    status: "open",
    priority: "unset",
    order: "a0",
    ...over,
  } as Task;
  render(<Fields task={task} lists={[]} known={[]} onPatch={vi.fn()} onError={onError} />);
  return onError;
}

const button = () => screen.getByRole("button", { name: /save to my calendar/i });
const saves = () => ipc.sent.filter((one) => one.cmd === "save_to_calendar");

beforeEach(() => {
  ipc.sent = [];
  ipc.chosen = null;
  ipc.fails = false;
});

describe("saving a task to the person's calendar", () => {
  it("is offered on a task with a day or a deadline, and on no other", () => {
    shown();
    expect(screen.queryByRole("button", { name: /save to my calendar/i })).toBeNull();
  });

  it("says what it does and that a later change stays behind", () => {
    shown({ deadline: day as Task["deadline"] });

    expect(button().getAttribute("title")).toMatch(/the event stays where it was/i);
  });

  it("opens the chosen calendar at once, asking nothing", async () => {
    ipc.chosen = "google";
    shown({ date: day as Task["date"] });

    await userEvent.click(button());

    await waitFor(() => expect(saves()).toHaveLength(1));
    expect(saves()[0]?.args).toEqual({ id: "01KA7B3VJ9RZ0Q8M5N2X4Y6W1T" });
    expect(screen.queryByText(/which calendar do you use/i)).toBeNull();
  });

  it("asks once which calendar, keeps the answer and then saves", async () => {
    shown({ date: day as Task["date"] });

    await userEvent.click(button());
    await userEvent.click(
      await screen.findByRole("button", { name: "Outlook for work or school" }),
    );

    await waitFor(() => expect(saves()).toHaveLength(2));
    expect(ipc.sent.find((one) => one.cmd === "keep_calendar")?.args).toEqual({
      calendar: "outlook-work",
    });
    expect(screen.queryByText(/which calendar do you use/i)).toBeNull();
  });

  it("offers Apple's calendar only on a Mac", async () => {
    shown({ date: day as Task["date"] });

    await userEvent.click(button());

    expect(await screen.findByRole("button", { name: "Google Calendar" })).toBeTruthy();
    expect(screen.queryByRole("button", { name: /apple/i })).toBeNull();
    expect(offered(true).find((one) => one.choice === "apple")?.away).toBeUndefined();
    expect(offered(false).find((one) => one.choice === "apple")?.away).toBe(true);
  });

  it("hands a failure to the window instead of swallowing it", async () => {
    ipc.chosen = "apple";
    ipc.fails = true;
    const onError = shown({ date: day as Task["date"] });

    await userEvent.click(button());

    await waitFor(() => expect(onError).toHaveBeenCalledWith({ code: "cannotOpen", name: "x" }));
  });
});
