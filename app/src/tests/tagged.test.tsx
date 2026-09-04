import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import type { Counted, Filed } from "../core";
import Tagged from "../ui/Tagged";
import Tags from "../ui/Tags";

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => Promise.resolve(null) }));

const doc = (id: string, title: string, tags: string[]): Filed => ({
  id,
  file: `${id}.md`,
  title,
  folder: null,
  archived: false,
  tags,
  wrote: "2026-09-02T09:00:00Z",
});

describe("the documents a tag holds", () => {
  it("says nothing at all where no document carries it", () => {
    const { container } = render(<Tagged docs={[]} onOpen={vi.fn()} />);

    expect(container.textContent).toBe("");
  });

  it("names them under a heading of their own, apart from the tasks", () => {
    render(
      <Tagged
        docs={[doc("01A", "Alquiler del local", ["legal", "contrato"]), doc("01B", "", ["legal"])]}
        onOpen={vi.fn()}
      />,
    );

    expect(screen.getByText("Documents")).toBeTruthy();
    expect(screen.getByText("2")).toBeTruthy();
    expect(screen.getByText("#legal #contrato")).toBeTruthy();
    expect(screen.getByText("Untitled")).toBeTruthy();
  });

  it("opens the one that was clicked, by the file the tree knows it as", async () => {
    const opened = vi.fn();
    render(<Tagged docs={[doc("01A", "Alquiler del local", ["legal"])]} onOpen={opened} />);

    await userEvent.click(screen.getByText("Alquiler del local"));

    expect(opened).toHaveBeenCalledWith("01A.md");
  });
});

describe("what a tag chip counts", () => {
  const shown = (tags: Counted[]) => render(<Tags tags={tags} chosen={[]} onToggle={vi.fn()} />);

  it("keeps a bare number where the tag lives only in tasks", () => {
    shown([{ tag: "casa", tasks: 3, docs: 0 }]);

    expect(screen.getByRole("button", { name: "#casa3" })).toBeTruthy();
  });

  it("says both where documents carry it too, tasks first", () => {
    shown([{ tag: "legal", tasks: 0, docs: 2 }]);

    expect(screen.getByRole("button", { name: "#legal0 · 2" })).toBeTruthy();
  });
});
