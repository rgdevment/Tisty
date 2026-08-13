import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import Search from "../ui/Search";
import TaskList from "../ui/TaskList";
import { nothing } from "../views";

const searched = vi.fn((_args: Record<string, unknown>) => Promise.resolve([]));
vi.mock("@tauri-apps/api/core", () => ({
  invoke: (name: string, args: Record<string, unknown>) => {
    if (name === "search") return searched(args);
    return Promise.resolve(null);
  },
}));

describe("what an empty screen says", () => {
  it("says something different on each of them", () => {
    const said = [
      nothing({ named: "tasks" }, false),
      nothing({ named: "tasks", slice: "upcoming" }, false),
      nothing({ named: "archive" }, false),
      nothing({ named: "search" }, false),
      nothing({ named: "search" }, true),
      nothing({ list: "01L" }, false),
    ];

    expect(new Set(said).size).toBe(said.length);
    expect(said.every((one) => one.length > 0)).toBe(true);
  });

  it("says the archive is a destination, not a failure", () => {
    expect(nothing({ named: "archive" }, false)).toMatch(/close/i);
  });

  it("talks about the search when a search came back empty", () => {
    expect(nothing({ named: "search" }, true)).toMatch(/match/i);
  });

  it("teaches the syntax where a new reader lands", () => {
    expect(nothing({ named: "tasks" }, false)).toMatch(/tomorrow/i);
  });

  it("reaches the list", () => {
    render(
      <TaskList
        tasks={[]}
        lists={[]}
        title="Today"
        empty={nothing({ named: "tasks" }, false)}
        onSelect={() => {}}
      />,
    );

    expect(screen.getByText(/Nothing for today/)).toBeTruthy();
  });
});

describe("the search scope", () => {
  it("asks on the first letter, whatever alphabet it is", async () => {
    searched.mockClear();
    render(<Search onFound={() => {}} onError={() => {}} />);

    const field = screen.getByRole("textbox");
    field.focus();
    await import("@testing-library/user-event").then(({ default: user }) => user.type(field, "会"));
    await new Promise((rest) => setTimeout(rest, 250));

    expect(searched).toHaveBeenCalledWith(expect.objectContaining({ query: "会" }));
  });

  it("looks in the archive by default, as the field says it does", async () => {
    searched.mockClear();
    render(<Search onFound={() => {}} onError={() => {}} />);

    const field = screen.getByRole("textbox");
    field.focus();
    await import("@testing-library/user-event").then(({ default: user }) =>
      user.type(field, "ticket"),
    );
    await new Promise((rest) => setTimeout(rest, 250));

    expect(searched).toHaveBeenCalledWith(expect.objectContaining({ scope: "either" }));
  });
});
