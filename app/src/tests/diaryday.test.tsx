import { act, render, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { t } from "../locales";
import DiaryView from "../ui/DiaryView";
import { type Reaching, wired, worn } from "../ui/dressing";
import { framed } from "../ui/framed";

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
  stale: false,
}));

const asking = vi.hoisted(() => ({ yes: true }));

const FILE = "a3f1-0011";
const TODAY = { book: "a3f1-0010", page: FILE };

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    ipc.calls.push({ cmd, args: args ?? {} });
    if (cmd === "diary_day_words") return Promise.resolve("Lluvia.");
    if (cmd === "diary_rewrite" && ipc.stale) return Promise.reject({ code: "diaryDayChanged" });
    if (cmd === "diary_rewrite" || cmd === "diary_unstick") return Promise.resolve(TODAY);
    if (cmd === "diary_laid") return Promise.resolve(new ArrayBuffer(0));
    if (cmd === "diary_closed") return Promise.resolve("");
    return Promise.resolve(null);
  },
}));

vi.mock("../sure", () => ({ sure: () => Promise.resolve(asking.yes) }));

const held = vi.hoisted(() => ({ page: null as null | Record<string, unknown> }));

vi.mock("../ui/Page", () => ({
  default: (props: Record<string, unknown>) => {
    held.page = props;
    return <div data-testid="days" />;
  },
}));

beforeEach(() => {
  ipc.calls = [];
  ipc.stale = false;
  asking.yes = true;
  held.page = null;
});

const asked = (cmd: string) => ipc.calls.filter((one) => one.cmd === cmd);

const page = (html: string): Document => new DOMParser().parseFromString(framed(html), "text/html");

const NOTE = (said: string) =>
  `<blockquote class="callout" data-callout="idea"><p>${said}</p></blockquote>`;

const MONTH =
  '<div class="diary-month">' +
  '<section class="diary-entry diary-is-today" data-day="16" data-nth="0">' +
  '<div class="diary-date"><span class="diary-number">16</span><span class="diary-weekday">lun</span></div>' +
  '<div class="diary-words"><p data-at="0-20">Paseo con <a href="tisty:task/01T">Rina</a> y <strong>Roger</strong>.</p>' +
  NOTE("Comprar pan") +
  `<ul data-at="40-60"><li>${NOTE("dentro de una lista")}</li></ul>` +
  NOTE("Llamar a Jaime") +
  '<details class="diary-closed"><summary>1 tarea cerrada</summary><ul><li><a href="tisty:task/01X">Pagar la luz</a></li></ul></details>' +
  "</div></section>" +
  '<section class="diary-entry" data-day="16" data-nth="1">' +
  '<div class="diary-date"><span class="diary-number">16</span></div>' +
  '<div class="diary-words"><div class="diary-box editing"><textarea class="diary-box-words">Lluvia.</textarea></div>' +
  '<div class="diary-box-bar"><button type="button" data-asked="dayCancel">Cancelar</button><button type="button" class="diary-box-keep" data-asked="dayKeep">Guardar</button></div>' +
  NOTE("Junto a la caja") +
  "</div></section></div>";

const reaching = () => {
  const heard = {
    days: [] as unknown[],
    notes: [] as unknown[],
    tasks: [] as string[],
    keeps: 0,
    cancels: 0,
    aways: 0,
  };
  const reach: Reaching = {
    body: "",
    onDoc: () => {},
    onTask: (id) => heard.tasks.push(id),
    onAway: () => {
      heard.aways += 1;
    },
    onDay: (at) => heard.days.push(at),
    onDayKeep: () => {
      heard.keeps += 1;
    },
    onDayCancel: () => {
      heard.cancels += 1;
    },
    onUnstick: (at, note) => heard.notes.push([at, note]),
  };
  return { heard, reach };
};

const click = (on: Element | null | undefined) =>
  on?.dispatchEvent(
    new MouseEvent("click", { bubbles: true, cancelable: true, button: 0, detail: 1 }),
  );

const key = (on: Element | null | undefined, said: string, ctrlKey = false) => {
  const event = new KeyboardEvent("keydown", {
    key: said,
    ctrlKey,
    bubbles: true,
    cancelable: true,
  });
  on?.dispatchEvent(event);
  return event;
};

describe("a day of the diary on the page", () => {
  it("opens from its words, by its number and its place among days of that number", () => {
    const own = page(MONTH);
    const { heard, reach } = reaching();
    wired(own, () => reach);

    click(own.querySelector(".diary-words strong"));
    click(own.querySelector(".diary-words a"));
    click(own.querySelector(".diary-number"));
    click(own.querySelector(".diary-closed li"));
    click(own.querySelector('blockquote[data-callout="idea"] p'));

    expect(heard.days).toEqual([{ day: 16, nth: 0 }]);
    expect(heard.tasks).toEqual(["01T"]);
  });

  it("does not open while words are being selected, nor from inside the box already open", () => {
    const frame = document.createElement("iframe");
    document.body.append(frame);
    const own = frame.contentDocument as Document;
    own.open();
    own.write(framed(MONTH));
    own.close();
    const { heard, reach } = reaching();
    wired(own, () => reach);
    const range = own.createRange();
    range.selectNodeContents(own.querySelector(".diary-words strong") as Element);
    own.getSelection()?.removeAllRanges();
    own.getSelection()?.addRange(range);

    click(own.querySelector(".diary-words strong"));
    own.getSelection()?.removeAllRanges();
    heard.aways = 0;
    const box = own.querySelector("textarea");
    box?.dispatchEvent(new MouseEvent("mousedown", { bubbles: true, button: 0 }));
    click(box);

    frame.remove();
    expect(heard.days).toEqual([]);
    expect(heard.aways).toBe(0);
  });

  it("keeps or cancels from the bar and from the keys, and the window never hears the box's keys", () => {
    const own = page(MONTH);
    const { heard, reach } = reaching();
    wired(own, () => reach);
    const window_ = vi.fn();
    window.addEventListener("keydown", window_);
    const box = own.querySelector("textarea");

    click(own.querySelector('[data-asked="dayKeep"]'));
    click(own.querySelector('[data-asked="dayCancel"]'));
    const keeping = key(box, "Enter", true);
    key(box, "Escape");
    key(box, "a");
    key(box, "z", true);
    key(box, "Enter");
    window.removeEventListener("keydown", window_);

    expect(heard.keeps).toBe(2);
    expect(heard.cancels).toBe(2);
    expect(keeping.defaultPrevented).toBe(true);
    expect(window_).not.toHaveBeenCalled();
  });

  it("gives each of a day's own notes a way off, numbered as the core numbers them", () => {
    const own = page(MONTH);
    const { heard, reach } = reaching();
    worn(own, () => true, reach);
    worn(own, () => true, reach);
    wired(own, () => reach);

    const offs = Array.from(own.querySelectorAll<HTMLElement>(".note-off"));
    expect(offs.map((one) => one.getAttribute("aria-label"))).toEqual([
      t("diaryUnstick"),
      t("diaryUnstick"),
      t("diaryUnstick"),
    ]);
    expect(own.querySelector("li .note-off"), "a note inside a list is words").toBeNull();
    for (const one of offs) click(one);

    expect(heard.notes).toEqual([
      [{ day: 16, nth: 0 }, 0],
      [{ day: 16, nth: 0 }, 1],
      [{ day: 16, nth: 1 }, 0],
    ]);
    expect(heard.days).toEqual([]);
  });

  it("offers no way off a note where the diary cannot be written", () => {
    const own = page(MONTH);
    worn(own, () => true, { body: "", onDoc: () => {} });

    expect(own.querySelector(".note-off")).toBeNull();
  });
});

const view = (writable = true) => {
  const onWritten = vi.fn();
  const onError = vi.fn();
  const shown = (file: string) => (
    <DiaryView
      file={file}
      body={"# Noviembre\n\n## Lunes 16\n\nLluvia.\n"}
      title="Noviembre"
      book="Diario 2026"
      fresh={0}
      writable={writable}
      onDoc={vi.fn()}
      onTask={vi.fn()}
      onBefore={() => Promise.resolve()}
      onWritten={onWritten}
      onEdit={vi.fn()}
      onError={onError}
    />
  );
  const { rerender } = render(shown(FILE));
  return { onWritten, onError, toMonth: (file: string) => rerender(shown(file)) };
};

type Held = {
  draw: (body: string) => Promise<string>;
  onDay?: (at: { day: number; nth: number }) => void;
  onDayKeep?: () => void;
  onDayCancel?: () => void;
  onAway?: () => void;
  onUnstick?: (at: { day: number; nth: number }, note: number) => void;
  onShown: (page: Window | null, body?: string) => void;
};

const now = () => held.page as unknown as Held;

const BOX =
  '<div class="diary-box editing"><textarea class="diary-box-words">Lluvia.</textarea></div>';

const frames: HTMLIFrameElement[] = [];

const framedBox = () => {
  const frame = document.createElement("iframe");
  document.body.append(frame);
  frames.push(frame);
  const own = frame.contentDocument as Document;
  own.body.innerHTML = BOX;
  return { frame, own };
};

const drawnOpen = async () => {
  ipc.calls = [];
  await now().draw("x");
  return asked("diary_laid")[0]?.args.open;
};

const opened = async () => {
  await act(async () => now().onDay?.({ day: 16, nth: 0 }));
  await waitFor(() => expect(asked("diary_day_words")).toHaveLength(1));
  const { frame, own } = framedBox();
  act(() => now().onShown(frame.contentWindow, "# Noviembre\n\n## Lunes 16\n\nLluvia.\n"));
  return own;
};

const type = (own: Document, said: string) => {
  const box = own.querySelector("textarea") as HTMLTextAreaElement;
  box.value = said;
  box.dispatchEvent(new Event("input", { bubbles: true }));
};

describe("a day of the diary written in place", () => {
  afterEach(() => {
    for (const one of frames.splice(0)) one.remove();
  });

  it("opens the day touched as a box holding its words, focused at their end", async () => {
    view();
    const own = await opened();

    expect(asked("diary_day_words")[0]?.args).toEqual({
      body: "# Noviembre\n\n## Lunes 16\n\nLluvia.\n",
      at: { day: 16, nth: 0 },
    });
    expect(await drawnOpen()).toEqual({ day: 16, nth: 0 });
    const box = own.querySelector("textarea") as HTMLTextAreaElement;
    expect(own.activeElement).toBe(box);
    expect(box.selectionStart).toBe("Lluvia.".length);
  });

  it("keeps what is typed when the month is drawn again under it, and writes it in place", async () => {
    const { onWritten } = view();
    const own = await opened();
    type(own, "Sol por la tarde.");

    own.body.innerHTML = BOX;
    act(() => now().onShown(own.defaultView, "# Noviembre\n"));
    expect((own.querySelector("textarea") as HTMLTextAreaElement).value).toBe("Sol por la tarde.");

    await act(async () => now().onDayKeep?.());

    await waitFor(() => expect(onWritten).toHaveBeenCalledWith(TODAY));
    expect(asked("diary_rewrite")[0]?.args).toEqual({
      file: FILE,
      at: { day: 16, nth: 0 },
      was: "Lluvia.",
      words: "Sol por la tarde.",
    });
    expect(await drawnOpen()).toBeNull();
  });

  it("closes an untouched box on a press away and keeps a changed one", async () => {
    view();
    await opened();
    act(() => now().onAway?.());
    expect(await drawnOpen()).toBeNull();

    const own = await opened();
    type(own, "Otra cosa.");
    act(() => now().onAway?.());
    expect(await drawnOpen()).toEqual({ day: 16, nth: 0 });
    act(() => now().onDay?.({ day: 15, nth: 0 }));
    expect(asked("diary_day_words")).toHaveLength(0);
  });

  it("drops what was typed on cancel and writes nothing when nothing changed", async () => {
    view();
    const own = await opened();
    type(own, "Borrador.");
    act(() => now().onDayCancel?.());
    expect(await drawnOpen()).toBeNull();

    await opened();
    await act(async () => now().onDayKeep?.());
    expect(await drawnOpen()).toBeNull();
    expect(asked("diary_rewrite")).toHaveLength(0);
  });

  it("stays open with its words when the day changed meanwhile", async () => {
    const { onError, onWritten } = view();
    ipc.stale = true;
    const own = await opened();
    type(own, "Sol.");

    await act(async () => now().onDayKeep?.());

    await waitFor(() => expect(onError).toHaveBeenCalled());
    expect(onWritten).not.toHaveBeenCalled();
    expect(await drawnOpen()).toEqual({ day: 16, nth: 0 });
  });

  it("takes a note off only once the person says so, naming the page that was shown", async () => {
    const { onWritten } = view();
    const { frame } = framedBox();
    act(() => now().onShown(frame.contentWindow, "lo que se vio"));

    asking.yes = false;
    await act(async () => now().onUnstick?.({ day: 16, nth: 0 }, 1));
    expect(asked("diary_unstick")).toHaveLength(0);

    asking.yes = true;
    await act(async () => now().onUnstick?.({ day: 16, nth: 0 }, 1));
    await waitFor(() => expect(onWritten).toHaveBeenCalledWith(TODAY));
    expect(asked("diary_unstick")[0]?.args).toEqual({
      file: FILE,
      at: { day: 16, nth: 0 },
      note: 1,
      shown: "lo que se vio",
    });
  });

  it("leaves an open box behind when another month is shown", async () => {
    const { toMonth } = view();
    const own = await opened();
    type(own, "Sol.");

    act(() => toMonth("a3f1-0012"));

    expect(await drawnOpen()).toBeNull();
  });

  it("offers neither a box nor a way off a note in a diary that cannot be written", () => {
    view(false);

    expect(now().onDay).toBeUndefined();
    expect(now().onUnstick).toBeUndefined();
    expect(now().onDayKeep).toBeUndefined();
  });
});
