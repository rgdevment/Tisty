import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import type { List } from "../core";
import Pulse from "../ui/Pulse";

const lists: List[] = [
  { id: "01A", name: "Casa", order: "a0" },
  { id: "01B", name: "Trabajo", order: "a1" },
];

const picked = () => ({
  list: vi.fn(),
  tags: vi.fn(),
  quadrants: vi.fn(),
});

const show = (
  counts: Record<string, number>,
  tags: { tag: string; tasks: number; docs: number }[] = [],
  hands = picked(),
) => {
  render(
    <Pulse
      counts={counts}
      lists={lists}
      tags={tags}
      papers={3}
      onList={hands.list}
      onTags={hands.tags}
      onQuadrants={hands.quadrants}
    />,
  );
  return hands;
};

describe("the day beside the list", () => {
  it("counts the whole store, not the slice on screen", () => {
    show({ overdue: 2, dueToday: 5, upcoming: 9 });

    const said = (label: string) =>
      screen.getByText(label).parentElement?.querySelector("dt")?.textContent;
    expect(said("overdue")).toBe("2");
    expect(said("for today")).toBe("5");
    expect(said("ahead")).toBe("9");
  });

  it("takes each quadrant from the tally, and opens Priorities from any of them", async () => {
    const hands = show({ do: 4, decide: 1, delegate: 0, minor: 2, quadrants: 7 });

    await userEvent.click(screen.getByRole("button", { name: "Do4" }));

    expect(hands.quadrants).toHaveBeenCalled();
    expect(screen.getByRole("button", { name: "Unclassified7" })).toBeTruthy();
  });

  it("keeps «unclassified» out of the way where every task has a priority", () => {
    show({ do: 1, quadrants: 0 });

    expect(screen.queryByText("Unclassified")).toBeNull();
  });

  it("names only the lists holding something, busiest first", () => {
    const hands = show({ "01A": 1, "01B": 6 });

    const named = screen
      .getAllByRole("button", { name: /Casa|Trabajo/ })
      .map((one) => one.textContent);
    expect(named).toEqual(["Trabajo6", "Casa1"]);
    expect(hands.list).not.toHaveBeenCalled();
  });

  it("says nothing of tags where none is in use", () => {
    show({});

    expect(screen.queryByText("Tags")).toBeNull();
  });

  it("puts the most used tag first and carries its count", async () => {
    const hands = show({}, [
      { tag: "banco", tasks: 1, docs: 0 },
      { tag: "casa", tasks: 4, docs: 0 },
    ]);

    const first = screen.getAllByRole("button", { name: /^#/ })[0];
    expect(first.textContent).toBe("#casa 4");

    await userEvent.click(first);
    expect(hands.tags).toHaveBeenCalled();
  });

  it("gathers what is left over as figures, not as a second way in", () => {
    show({ all: 12, inbox: 3, undated: 4, routines: 2, archive: 30 });

    expect(screen.getByText("With no date").parentElement?.textContent).toBe("With no date4");
    expect(screen.getByText("Archived").parentElement?.textContent).toBe("Archived30");
    expect(screen.queryByRole("button", { name: /Archived/ })).toBeNull();
  });
});
