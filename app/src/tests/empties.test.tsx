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
  /// One line for six situations told the reader nothing at the only moment
  /// they were going to read.
  it("says something different on each of them", () => {
    const said = [
      nothing({ named: "today" }, false),
      nothing({ named: "inbox" }, false),
      nothing({ named: "upcoming" }, false),
      nothing({ named: "archive" }, false),
      nothing({ named: "search" }, false),
      nothing({ named: "search" }, true),
      nothing({ list: "01L" }, false),
    ];

    expect(new Set(said).size).toBe(said.length);
    expect(said.every((one) => one.length > 0)).toBe(true);
  });

  /// In the archive «Nothing here» reads as a fault; it is where things arrive.
  it("says the archive is a destination, not a failure", () => {
    expect(nothing({ named: "archive" }, false)).toMatch(/close/i);
  });

  /// A search that found nothing is about the search, not about the place.
  it("talks about the search when a search came back empty", () => {
    expect(nothing({ named: "search" }, true)).toMatch(/match/i);
  });

  /// The first screen after installing is an empty «Today».
  it("teaches the syntax where a new reader lands", () => {
    expect(nothing({ named: "today" }, false)).toMatch(/tomorrow/i);
  });

  it("reaches the list", () => {
    render(
      <TaskList
        tasks={[]}
        lists={[]}
        title="Today"
        centred
        empty={nothing({ named: "today" }, false)}
        onSelect={() => {}}
      />,
    );

    expect(screen.getByText(/Nothing for today/)).toBeTruthy();
  });
});

describe("the search scope", () => {
  /// The placeholder promised the archive and the default reached only what
  /// was open: a ticket from six months ago read as lost.
  /// One letter matches most of a long archive, and that cost lands on every
  /// keystroke: the whole thing crosses IPC and gets drawn.
  it("waits for a second letter before asking at all", async () => {
    searched.mockClear();
    render(<Search onFound={() => {}} onError={() => {}} />);

    const field = screen.getByRole("textbox");
    field.focus();
    await import("@testing-library/user-event").then(({ default: user }) => user.type(field, "e"));
    await new Promise((rest) => setTimeout(rest, 250));

    expect(searched).not.toHaveBeenCalled();
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
