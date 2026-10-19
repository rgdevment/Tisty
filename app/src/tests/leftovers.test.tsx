import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { Reviewed } from "../core";
import { fill, t } from "../locales";
import Leftovers from "../ui/Leftovers";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: () => Promise.resolve(null),
}));

const audit: Reviewed = {
  tasks: 7,
  lists: 2,
  agrees: true,
  loose: 0,
  looseBytes: 0,
  astray: [],
  unread: null,
  stranded: [],
  missing: [],
  events: 42,
  machines: [],
  logBytes: 0,
  docsBytes: 0,
  heldBytes: 0,
  heldFiles: 0,
};

const shown = (reviewed: Reviewed) =>
  render(
    <Leftovers
      audit={reviewed}
      build={null}
      busy={null}
      run={() => {}}
      fail={() => {}}
      letGo={() => {}}
      letGoOfAll={() => {}}
      takeIn={() => {}}
      takeInAll={() => {}}
      letGoOfPaper={() => {}}
      forgetMissing={() => {}}
    />,
  );

describe("what is loose in Maintenance", () => {
  it("says nothing is loose when every document was read", () => {
    shown(audit);

    expect(screen.getByText(t("looseNone"))).toBeTruthy();
  });

  it("names the document it could not read instead of saying nothing is loose", () => {
    shown({ ...audit, unread: "Recetas" });

    expect(screen.getByText(fill("looseUnread", "Recetas"))).toBeTruthy();
    expect(screen.queryByText(t("looseNone"))).toBeNull();
    expect(screen.queryByRole("button", { name: t("upkeepSafeAll") })).toBeNull();
  });

  it("calls an untitled document by what the window calls it", () => {
    shown({ ...audit, unread: "" });

    expect(screen.getByText(fill("looseUnread", t("untitledDoc")))).toBeTruthy();
  });
});
