import { readdirSync, readFileSync } from "node:fs";
import { fireEvent, render, screen } from "@testing-library/react";
import { afterEach, beforeAll, beforeEach, describe, expect, it, vi } from "vitest";
import type { Habit, Series, Task, Turn } from "../core";
import { t } from "../locales";
import Ahead, { weekday } from "../ui/Ahead";
import Detail from "../ui/Detail";
import Spread from "../ui/Spread";

const scrolledTo: Element[] = [];

beforeAll(() => {
  if (!Element.prototype.setPointerCapture) {
    Element.prototype.setPointerCapture = vi.fn();
    Element.prototype.releasePointerCapture = vi.fn();
  }
  Element.prototype.scrollIntoView = function (this: Element) {
    scrolledTo.push(this);
  };
});

beforeEach(() => {
  scrolledTo.length = 0;
});

const iso = (at: Date): string =>
  `${at.getFullYear()}-${String(at.getMonth() + 1).padStart(2, "0")}-${String(at.getDate()).padStart(2, "0")}`;

const weekDay = (n: number): string => {
  const now = new Date();
  const first = new Date(
    now.getFullYear(),
    now.getMonth(),
    now.getDate() - ((now.getDay() + 6) % 7),
  );
  return iso(new Date(first.getFullYear(), first.getMonth(), first.getDate() + n));
};

const made = (id: string, title: string, on?: string, clock = ""): Task =>
  ({
    id,
    title,
    status: "open",
    priority: "unset",
    order: "a0",
    steps: [],
    log: [],
    volume: {},
    ...(on === undefined
      ? {}
      : {
          date: {
            at: `${on}T${clock || "00:00:00"}`,
            tz: "America/Santiago",
            floating: true,
            has_time: clock !== "",
          },
        }),
  }) as unknown as Task;

const dragged = (from: HTMLElement, onto: Element | null) => {
  const was = document.elementFromPoint;
  document.elementFromPoint = () => onto as Element;
  fireEvent.pointerDown(from, { button: 0, clientX: 0, clientY: 0 });
  fireEvent.pointerMove(from, { clientX: 80, clientY: 80 });
  fireEvent.pointerUp(from, { clientX: 80, clientY: 80 });
  document.elementFromPoint = was;
};

const dayAt = (n: number) => document.querySelector(`[data-day="${weekDay(n)}"]`);

const sources = (where: string[], ext: RegExp = /\.tsx$/): { name: string; body: string }[] =>
  where.flatMap((one) =>
    readdirSync(one, { withFileTypes: true })
      .filter((entry) => entry.isFile() && ext.test(entry.name))
      .map((entry) => ({
        name: `${one}/${entry.name}`,
        body: readFileSync(`${one}/${entry.name}`, "utf8"),
      })),
  );

describe("the crowded-day nudge never sends a task to a day that has already gone", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date(2026, 8, 11, 9, 0, 0));
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("offers a free day no earlier than today, not just later than where the task landed", () => {
    const placed = vi.fn();
    render(
      <Spread
        tasks={[
          made("01A", "Uno", weekDay(2)),
          made("01B", "Dos", weekDay(2)),
          made("01C", "Sin fecha"),
        ]}
        onPlace={placed}
        onOpen={vi.fn()}
      />,
    );

    dragged(screen.getByText("Sin fecha"), dayAt(2));
    fireEvent.click(
      screen.getByRole("button", { name: new RegExp(t("spreadMoveIt").split("{")[0].trim()) }),
    );

    const offered = placed.mock.calls[placed.mock.calls.length - 1][1] as string;
    expect(offered >= weekDay(4)).toBe(true);
  });
});

describe("pressing Hoy while the month is open", () => {
  it("asks to reach today's row in the river, not just the top of it", () => {
    const today = iso(new Date());
    render(<Spread tasks={[]} onPlace={vi.fn()} onOpen={vi.fn()} />);

    fireEvent.click(screen.getByRole("button", { name: t("spreadMoon") }));
    fireEvent.click(screen.getByRole("button", { name: t("spreadNow") }));

    const row = document.querySelector(`[data-day="${today}"]`);
    expect(scrolledTo).toContain(row);
  });
});

describe("what a cell in the month grid tells you about itself", () => {
  it("a cell holding a task announces that task in its label", () => {
    render(
      <Spread tasks={[made("01A", "Kermés", weekDay(3))]} onPlace={vi.fn()} onOpen={vi.fn()} />,
    );

    fireEvent.click(screen.getByRole("button", { name: t("spreadMoon") }));

    const cell = document.querySelector(`[data-day="${weekDay(3)}"]`);
    expect(cell?.getAttribute("aria-label")).toContain("Kermés");
  });

  it("a cell outside the shown month names its month, not just a bare weekday and number", () => {
    render(<Spread tasks={[]} onPlace={vi.fn()} onOpen={vi.fn()} />);

    fireEvent.click(screen.getByRole("button", { name: t("spreadMoon") }));

    const now = new Date();
    const cells = [...document.querySelectorAll("[data-day]")];
    const away = cells.find((one) => {
      const key = one.getAttribute("data-day") ?? "";
      const month = Number(key.split("-")[1]) - 1;
      return month !== now.getMonth();
    });
    const [year, month, date] = (away?.getAttribute("data-day") ?? "").split("-").map(Number);
    const at = new Date(year, month - 1, date);
    const bare = `${weekday().format(at)} ${at.getDate()}`;

    expect(away?.getAttribute("aria-label")).not.toBe(bare);
  });
});

describe("the house keeps one scale, even where manners.test.ts cannot see", () => {
  it("never leaves a bare rounded class lying around, one that a bracketed-size guard would miss", () => {
    const bare = sources(["src/ui"]).flatMap(({ name, body }) =>
      [...body.matchAll(/\brounded\b(?!-)/g)].map(() => name),
    );

    expect(bare).toEqual([]);
  });

  it("never sizes type with a named Tailwind scale, one a bracketed-px guard would miss", () => {
    const named = sources(["src/ui", "src"]).flatMap(({ name, body }) =>
      [...body.matchAll(/\btext-(xs|sm|base|lg|[0-9]?xl)\b/g)].map((hit) => `${name}: ${hit[0]}`),
    );

    expect(named).toEqual([]);
  });
});

describe("a translation only stays in the catalogue if something reads it", () => {
  it("every spread* key in the English catalogue is used somewhere outside locales.ts", () => {
    const catalogue = readFileSync("src/locales.ts", "utf8");
    const start = catalogue.indexOf("const en = {");
    const end = catalogue.indexOf("\n};", start);
    const enBlock = catalogue.slice(start, end);
    const keys = [...enBlock.matchAll(/^\s*(spread\w*):/gm)].map((hit) => hit[1]);

    const body = sources(["src/ui", "src"], /\.tsx?$/)
      .filter(({ name }) => !name.endsWith("locales.ts"))
      .map(({ body: text }) => text)
      .join("\n");

    const unread = keys.filter((key) => !body.includes(key));

    expect(unread).toEqual([]);
  });
});

describe("the title's size does not actually depend on whether the task is expanded", () => {
  it("renders the very same class whether expanded is true or false", () => {
    const task = made("01A", "Kermés");
    const props = {
      task,
      lists: [],
      known: [],
      onExpand: vi.fn(),
      onCollapse: vi.fn(),
      onPatch: vi.fn(),
      onStep: vi.fn(),
      onMark: vi.fn(),
      onDropStep: vi.fn(),
      onLog: vi.fn(),
      onComplete: vi.fn(),
      onDiscard: vi.fn(),
      onReopen: vi.fn(),
      onErase: vi.fn(),
      onClose: vi.fn(),
    };

    const closed = render(<Detail {...props} expanded={false} />);
    const narrow = screen.getByRole("textbox", { name: t("fieldTitle") }).className;
    closed.unmount();

    render(<Detail {...props} expanded />);
    const wide = screen.getByRole("textbox", { name: t("fieldTitle") }).className;

    expect(wide).toBe(narrow);
  });
});

describe("the ahead strip only ever draws a handful of beads", () => {
  it("draws no more than five beads for a routine with hundreds of turns behind it", () => {
    const turns: Turn[] = Array.from({ length: 400 }, (_, i) => ({
      id: `t${i}`,
      status: "done",
    }));
    const series: Series = {
      last: "2026-01-01",
      title: "Take pills",
      turns,
      kept: 400,
      owed: 400,
      dropped: 0,
      open: 0,
      skipped: 0,
      streak: 400,
      longest: 400,
      measurable: true,
    };
    const routine: Habit = {
      task: {
        ...made("01R", "Take pills", weekDay(0)),
        repeat: { from: "due", each: { every: 1, unit: "day" } },
      } as unknown as Task,
      series,
    };

    render(<Ahead coming={[]} routines={[routine]} days={7} onOpen={vi.fn()} />);

    const beads = document.querySelectorAll('[class*="h-1.5"][class*="w-1.5"]');
    expect(beads.length).toBe(5);
  });
});
