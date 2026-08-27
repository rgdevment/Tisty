import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import type { Edits, Parsed } from "../core";
import Chips from "../ui/Chips";

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => Promise.resolve(null) }));

const read = (over: Partial<Parsed> = {}): Parsed => ({
  title: "pagar la luz",
  date: { at: "2026-08-11T10:00:00", tz: "America/Santiago", floating: true, has_time: true },
  tags: [],
  spans: [],
  offers: [],
  ...over,
});

const show = (over: Partial<Parsed> = {}, edits: Edits = {}) => {
  const onEdit = vi.fn();
  render(<Chips seen={read(over)} edits={edits} onEdit={onEdit} empty={<span>nothing</span>} />);
  return onEdit;
};

describe("what the chips show of a captured line", () => {
  it("shows nothing at all when the line said nothing", () => {
    show({ date: undefined });

    expect(screen.getByText("nothing")).toBeTruthy();
  });

  it("shows the list it was filed into", () => {
    show({ list: "Casa" });

    expect(screen.getByText("Casa")).toBeTruthy();
  });

  it("shows every tag it carried", () => {
    show({ tags: ["dinero", "hogar"] });

    expect(screen.getByText("dinero")).toBeTruthy();
    expect(screen.getByText("hogar")).toBeTruthy();
  });

  it("leaves out what was already waved away", () => {
    show({ list: "Casa", tags: ["dinero"] }, { noList: true, noTags: ["dinero"] });

    expect(screen.queryByText("Casa")).toBeNull();
    expect(screen.queryByText("dinero")).toBeNull();
  });

  it("shows a deadline beside the date, and drops it when told to", () => {
    show({
      deadline: {
        at: "2026-08-20T10:00:00",
        tz: "America/Santiago",
        floating: true,
        has_time: false,
      },
    });
    expect(screen.getAllByRole("button").length).toBeGreaterThan(1);
  });
});

describe("taking a chip back out", () => {
  const drop = async (label: string) => {
    const onEdit = show({ list: "Casa", tags: ["dinero"] });
    await userEvent.click(screen.getByRole("button", { name: `Remove ${label}` }));
    return onEdit;
  };

  it("says the list is not wanted", async () => {
    const onEdit = await drop("Casa");

    expect(onEdit).toHaveBeenCalledWith(expect.objectContaining({ noList: true }));
  });

  it("says that tag is not wanted, leaving the others alone", async () => {
    const onEdit = await drop("dinero");

    expect(onEdit).toHaveBeenCalledWith(expect.objectContaining({ noTags: ["dinero"] }));
  });
});
