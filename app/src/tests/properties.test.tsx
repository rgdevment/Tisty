import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { t } from "../locales";
import Properties from "../ui/Properties";

const front = "---\ntitle: Plan\ntags: [rina]\nfuente: libreta\n---\n\n# Plan";

describe("the properties beside a note", () => {
  it("shows nothing for a note that brings none", () => {
    const { container } = render(<Properties body="# Hola" onEdit={vi.fn()} />);
    expect(container.innerHTML).toBe("");
  });

  it("lists what the note says of itself, leaving out what the panel already shows", () => {
    render(<Properties body={front} skip={["title", "tags", "tag"]} />);
    expect(screen.getByText(t("frontNamed"))).toBeTruthy();
    expect(screen.getByText("fuente")).toBeTruthy();
    expect(screen.getByText("libreta")).toBeTruthy();
    expect(screen.queryByText("title")).toBeNull();
  });

  it("opens the block to edit, even when every property is shown elsewhere", () => {
    const edit = vi.fn();
    render(<Properties body={"---\ntitle: Plan\n---"} onEdit={edit} skip={["title"]} />);
    fireEvent.click(screen.getByText(t("frontEdit")));
    expect(edit).toHaveBeenCalled();
  });

  it("stays away when everything is shown elsewhere and nothing can be edited", () => {
    const { container } = render(<Properties body={"---\ntitle: Plan\n---"} skip={["title"]} />);
    expect(container.innerHTML).toBe("");
  });

  it("says so when the properties are too long to read", () => {
    render(<Properties body={`---\nnota: ${"b ".repeat(2100)}\n---`} />);
    expect(screen.getByText(t("frontOvergrown"))).toBeTruthy();
  });
});
