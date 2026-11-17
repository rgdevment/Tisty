import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { ahead, every, repeating, turnOf, unlisted } from "../examples";
import { adopt } from "../locales";
import Search from "../ui/Search";
import TaskList from "../ui/TaskList";
import { type Chosen, nothing, offers } from "../views";

const quoted = (said: string): string => /«([^»]+)»/.exec(said)?.[1] ?? "";

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
    expect(every()).toContain(quoted(nothing({ named: "tasks" }, false)));
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

describe("the example an empty screen offers", () => {
  const example = quoted(nothing({ named: "tasks" }, false));
  const today = (onTry?: (example: string) => void) =>
    render(
      <TaskList
        tasks={[]}
        lists={[]}
        title="Today"
        empty={nothing({ named: "tasks" }, false)}
        onTry={onTry}
        onSelect={() => {}}
      />,
    );

  it("hands over the sentence it quotes at a click", async () => {
    const tried = vi.fn();
    today(tried);

    await userEvent.click(screen.getByRole("button", { name: example }));

    expect(tried).toHaveBeenCalledWith(example);
  });

  it("keeps the words around the sentence as they read", () => {
    const { container } = today(vi.fn());

    expect(container.textContent).toContain(nothing({ named: "tasks" }, false));
  });

  it("keeps its quote marks and the sign after it on the example's own line", () => {
    const said = nothing({ list: "01L" }, false);
    const inList = quoted(said);
    render(
      <TaskList
        tasks={[]}
        lists={[]}
        title="Home"
        empty={said}
        onTry={vi.fn()}
        onSelect={() => {}}
      />,
    );

    const button = screen.getByRole("button", { name: inList });

    expect(button.textContent).toBe(`«${inList}»`);
    expect(button.parentElement?.className).toContain("whitespace-nowrap");
    expect(button.parentElement?.textContent).toBe(`«${inList}».`);
  });

  it("stays plain words where nothing can take it", () => {
    today();

    expect(screen.queryByRole("button", { name: example })).toBeNull();
    expect(screen.getByText(nothing({ named: "tasks" }, false))).toBeTruthy();
  });

  it("leaves an empty screen that quotes nothing as it reads", () => {
    const said = nothing({ named: "tasks", slice: "all" }, false);
    render(
      <TaskList
        tasks={[]}
        lists={[]}
        title="Home"
        empty={said}
        onTry={vi.fn()}
        onSelect={() => {}}
      />,
    );

    expect(screen.getByText(said).querySelector("button")).toBeNull();
  });

  it("is offered beside a capture field only, and never over a search", () => {
    expect(offers({ named: "tasks" }, false)).toBe(true);
    expect(offers({ named: "tasks", slice: "repeating" }, false)).toBe(true);
    expect(offers({ list: "01L" }, false)).toBe(true);
    expect(offers({ named: "tasks" }, true)).toBe(false);
    expect(offers({ named: "archive", layer: "routine" }, false)).toBe(false);
    expect(offers({ named: "search" }, false)).toBe(false);
  });
});

describe("the examples that take turns on an empty screen", () => {
  const turns = (chosen: Chosen, many: number) =>
    Array.from({ length: many }, (_, n) => quoted(nothing(chosen, false, 0, n / many)));

  it("gives a view another one as its turn moves, each of them once", () => {
    const all = unlisted(every());

    expect(turns({ named: "tasks" }, all.length)).toEqual(all);
  });

  it("offers Upcoming only what is dated ahead, and Repeating only what comes back", () => {
    expect(turns({ named: "tasks", slice: "upcoming" }, ahead().length)).toEqual(ahead());
    expect(turns({ named: "tasks", slice: "repeating" }, repeating().length)).toEqual(repeating());
  });

  it("offers a list one too, and never one that names a list of its own", () => {
    const offered = turns({ list: "01L" }, unlisted(every()).length);

    expect(new Set(offered).size).toBeGreaterThan(1);
    expect(offered.filter((one) => one.includes("@"))).toEqual([]);
    expect(every().some((one) => one.includes("@"))).toBe(true);
  });

  it("holds the last one at the very end of the turn, never past it", () => {
    const all = unlisted(every());

    expect(turnOf(all, 0.999999)).toBe(all[all.length - 1]);
    expect(turnOf([], 0.5)).toBe("");
  });

  it("carries as many in Spanish as in English, kind by kind", () => {
    const sizes = () => [ahead().length, repeating().length, every().length];
    const english = sizes();
    adopt("es");
    const spanish = sizes();
    adopt("en");

    expect(spanish).toEqual(english);
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
