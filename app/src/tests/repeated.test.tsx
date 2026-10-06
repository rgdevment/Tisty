import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { fill, t } from "../locales";
import Repeated from "../ui/Repeated";

const ipc = vi.hoisted(() => ({
  asked: 0,
  groups: [] as { name: string; lists: number; tasks: number }[],
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) => {
    if (cmd === "repeated_lists") {
      ipc.asked += 1;
      return Promise.resolve(ipc.groups);
    }
    return Promise.resolve(null);
  },
}));

beforeEach(() => {
  ipc.asked = 0;
  ipc.groups = [];
});

describe("the repeated lists in Maintenance", () => {
  it("names each repeated list, how many there are and what they hold, before joining", async () => {
    ipc.groups = [
      { name: "Trabajo", lists: 2, tasks: 14 },
      { name: "Familia", lists: 3, tasks: 0 },
    ];
    render(<Repeated held={false} join={() => {}} className="" />);

    expect(await screen.findByText("Trabajo")).toBeTruthy();
    expect(screen.getByText(fill("repeatedRow", "2", "14"))).toBeTruthy();
    expect(screen.getByText(fill("repeatedRow", "3", "0"))).toBeTruthy();
    expect(screen.getByRole("button", { name: t("repeatedDo") })).toBeTruthy();
  });

  it("says nothing repeats and offers no button when every name is single", async () => {
    render(<Repeated held={false} join={() => {}} className="" />);

    expect(await screen.findByText(t("repeatedNone"))).toBeTruthy();
    expect(screen.queryByRole("button", { name: t("repeatedDo") })).toBeNull();
  });

  it("joins on a click and looks again once it is done", async () => {
    ipc.groups = [{ name: "Personal", lists: 2, tasks: 3 }];
    const join = vi.fn((then: () => void) => {
      ipc.groups = [];
      then();
    });
    render(<Repeated held={false} join={join} className="" />);

    await userEvent.click(await screen.findByRole("button", { name: t("repeatedDo") }));

    expect(join).toHaveBeenCalledTimes(1);
    expect(await screen.findByText(t("repeatedNone"))).toBeTruthy();
    expect(ipc.asked).toBe(2);
  });

  it("waits while another card is working", async () => {
    ipc.groups = [{ name: "Personal", lists: 2, tasks: 3 }];
    render(<Repeated held join={() => {}} className="" />);

    const button = await screen.findByRole("button", { name: t("repeatedDo") });
    expect((button as HTMLButtonElement).disabled).toBe(true);
  });
});
