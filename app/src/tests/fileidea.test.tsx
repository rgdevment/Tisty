import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import type { Filed } from "../core";
import FileIdea from "../ui/FileIdea";

const doc = (file: string, title: string, lang?: string, tongue?: [string, number]): Filed => ({
  id: file,
  file,
  title,
  folder: null,
  archived: false,
  away: false,
  lang: lang ?? null,
  tongue: tongue ?? null,
});

const known = [
  doc("a1-0001", "Rina", "es", ["a1-0001", 0]),
  doc("a1-0002", "Rina", "en", ["a1-0001", 1]),
  doc("a1-0003", "Recetas", "es"),
];

describe("choosing the document an idea goes into", () => {
  it("names the language of each version of a document that has several", async () => {
    const onPick = vi.fn();
    render(<FileIdea idea="una idea" known={known} own="" onPick={onPick} onClose={() => {}} />);

    const list = screen.getByRole("list");
    expect(within(list).getByText("Español")).toBeTruthy();
    expect(within(list).getByText("English")).toBeTruthy();
    expect(within(list).getByRole("button", { name: /Recetas/ }).textContent).toBe("Recetas");
    await userEvent.click(within(list).getByRole("button", { name: /English/ }));
    expect(onPick).toHaveBeenCalledWith("a1-0002");
  });

  it("finds a version by the name of its language", async () => {
    render(<FileIdea idea="una idea" known={known} own="" onPick={() => {}} onClose={() => {}} />);

    await userEvent.type(screen.getByRole("textbox"), "english");

    expect(within(screen.getByRole("list")).getAllByRole("button")).toHaveLength(1);
  });
});
