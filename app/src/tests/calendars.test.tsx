import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { IcsBrought } from "../core";
import { fill, t } from "../locales";
import { Importing } from "../ui/Calendars";

const calls: { cmd: string; args: unknown }[] = [];
let brought: IcsBrought;
let refused: { code: string } | null = null;

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args: unknown) => {
    calls.push({ cmd, args });
    if (cmd === "ics_read") return refused ? Promise.reject(refused) : Promise.resolve(brought);
    if (cmd === "ics_take") return Promise.resolve({ made: 1, missed: 0 });
    return Promise.resolve([]);
  },
}));

const found = (at: number, title: string, more: Partial<IcsBrought["found"][number]> = {}) => ({
  at,
  kind: "event" as const,
  title,
  place: "",
  odd: [],
  done: false,
  already: false,
  takeable: true,
  ...more,
});

describe("bringing in a calendar", () => {
  beforeEach(() => {
    calls.length = 0;
    refused = null;
    brought = {
      invitation: true,
      stamp: "s1",
      found: [
        found(0, "Dentist", { place: "Av. 1" }),
        found(1, "Old meeting", { odd: ["past"] }),
        found(2, "Gym", { already: true, takeable: false }),
      ],
    };
  });

  it("picks what is ahead, leaves what passed unpicked and what came in before shut", async () => {
    const done = vi.fn();
    render(
      <Importing
        path="C:/in/invite.ics"
        lists={[{ id: "l1", name: "Health", order: "a0" }]}
        onDone={done}
        fail={() => {}}
      />,
    );

    await screen.findByText("Dentist");
    expect(screen.getByText(t("icsInvitation"))).toBeTruthy();
    const boxes = screen.getAllByRole("checkbox") as HTMLInputElement[];
    expect(boxes.map((one) => [one.checked, one.disabled])).toEqual([
      [true, false],
      [false, false],
      [false, true],
    ]);
    expect(screen.getByText(t("icsAlready"))).toBeTruthy();

    fireEvent.change(screen.getByRole("combobox"), { target: { value: "l1" } });
    fireEvent.click(screen.getByRole("button", { name: fill("icsTake", "1") }));

    await waitFor(() => expect(done).toHaveBeenCalledWith(1, 0));
    expect(calls.find((one) => one.cmd === "ics_take")?.args).toEqual({
      path: "C:/in/invite.ics",
      stamp: "s1",
      picked: [0],
      list: "l1",
    });
  });

  it("says plainly when the file is no calendar", async () => {
    refused = { code: "icsNotACalendar" };
    render(<Importing path="C:/in/notes.ics" lists={[]} onDone={() => {}} fail={() => {}} />);

    expect(await screen.findByText(t("icsNotACalendar"))).toBeTruthy();
  });
});
