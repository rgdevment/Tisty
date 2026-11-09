import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { Filed } from "../core";
import { fill } from "../locales";
import Tree from "../ui/Tree";

const doc = (one: Partial<Filed> & { id: string; file: string; title: string }): Filed => ({
  folder: null,
  archived: false,
  away: false,
  ...one,
});

describe("a document that came from another Tisty", () => {
  it("says whose it is, and that whoever edits it through a link sends to the same link", () => {
    render(
      <Tree
        papers={{
          folders: [],
          docs: [
            doc({ id: "D1", file: "d-1", title: "Read", guest: "Mario" }),
            doc({ id: "D2", file: "d-2", title: "Edited", guest: "Mario", editing: true }),
          ],
        }}
        onOpen={vi.fn()}
        onFile={vi.fn()}
      />,
    );

    for (let round = 0; round < 4; round += 1) {
      for (const one of screen.queryAllByRole("button", { name: /^Open / })) fireEvent.click(one);
    }

    expect(screen.getAllByTitle(fill("guestDoc", "Mario"))).toHaveLength(1);
    expect(screen.getAllByTitle(fill("guestEditing", "Mario"))).toHaveLength(1);
  });
});
