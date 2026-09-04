import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { Counted, List } from "../core";
import { t } from "../locales";
import SlashMenu from "../ui/SlashMenu";

const lists: List[] = [
  { id: "l1", name: "Trabajo", order: "a", archived: false },
  { id: "l2", name: "Casa", order: "b", archived: false },
];
const tags: Counted[] = [
  { tag: "dinero", tasks: 4, docs: 0 },
  { tag: "casa", tasks: 2, docs: 0 },
];

const show = (query = "") => {
  const onDate = vi.fn();
  const onInsert = vi.fn();
  const onClose = vi.fn();
  render(
    <SlashMenu
      from={null}
      query={query}
      lists={lists}
      tags={tags}
      onDate={onDate}
      onInsert={onInsert}
      onClose={onClose}
    />,
  );
  return { onDate, onInsert, onClose };
};

const press = (key: string) => fireEvent.keyDown(document, { key });

describe("walking the field menu with the keys", () => {
  it("wraps around the bottom and back off the top", () => {
    show();
    const rows = () => screen.getAllByRole("option");
    const on = () => rows().findIndex((one) => one.getAttribute("aria-selected") === "true");

    expect(on()).toBe(0);
    press("ArrowDown");
    expect(on()).toBe(1);
    press("ArrowUp");
    press("ArrowUp");
    expect(on()).toBe(rows().length - 1);
  });

  it("takes the one it is standing on", () => {
    const { onDate } = show();

    press("Enter");

    expect(onDate).toHaveBeenCalledWith("date");
  });

  it("closes on escape, and ignores a key that means nothing here", () => {
    const { onClose, onDate } = show();

    press("a");
    expect(onClose).not.toHaveBeenCalled();

    press("Escape");
    expect(onClose).toHaveBeenCalled();
    expect(onDate).not.toHaveBeenCalled();
  });
});

describe("the second step, once a field is chosen", () => {
  const into = (label: string) => {
    const said = show();
    fireEvent.click(screen.getByText(label));
    return said;
  };

  it("offers the lists, and writes the one you take", () => {
    const { onInsert } = into(t("fieldList"));

    expect(screen.getByText("Trabajo")).toBeTruthy();
    fireEvent.click(screen.getByText("Casa"));

    expect(onInsert).toHaveBeenCalledWith("@casa");
  });

  it("offers the tags with how many wear them", () => {
    const { onInsert } = into(t("fieldTag"));

    expect(screen.getByText("dinero")).toBeTruthy();
    fireEvent.click(screen.getByText("casa"));

    expect(onInsert).toHaveBeenCalledWith("#casa");
  });

  it("offers the cadences in words", () => {
    const { onInsert } = into(t("fieldRepeat"));

    const rows = screen.getAllByRole("option");
    expect(rows.length).toBeGreaterThan(1);
    fireEvent.click(rows[0]);

    expect(onInsert).toHaveBeenCalledWith(expect.any(String));
  });

  it("offers the priorities, written the way you would type them", () => {
    const { onInsert } = into(t("fieldPriority"));

    const rows = screen.getAllByRole("option");
    fireEvent.click(rows[0]);

    expect(onInsert).toHaveBeenCalledWith(expect.stringMatching(/^!/));
  });

  it("asks for the deadline, which is a date of its own", () => {
    const { onDate } = into(t("fieldDeadline"));

    expect(onDate).toHaveBeenCalledWith("deadline");
  });
});
