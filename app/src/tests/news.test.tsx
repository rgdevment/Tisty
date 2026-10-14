import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { adopt, t } from "../locales";
import { headline, toldFor } from "../news";
import news from "../news.json";
import News from "../ui/News";

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
  answers: {} as Record<string, () => Promise<unknown>>,
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    ipc.calls.push({ cmd, args: args ?? {} });
    return ipc.answers[cmd]?.() ?? Promise.resolve(null);
  },
}));

const sent = (cmd: string) => ipc.calls.filter((one) => one.cmd === cmd);

beforeEach(() => {
  ipc.calls = [];
  ipc.answers = {};
  adopt("es");
});

describe("what a new version tells", () => {
  it("says the same in Spanish and in English for every version", () => {
    for (const one of news) {
      expect(one.es.length, one.version).toBe(one.en.length);
      for (const said of [...one.es, ...one.en]) {
        expect(said.title.trim().length, one.version).toBeGreaterThan(0);
        expect(said.said.trim().length, one.version).toBeGreaterThan(0);
      }
    }
  });

  it("tells only the versions asked for, in the window's language", () => {
    const [first] = news;
    expect(toldFor([first.version])[0].told).toEqual(first.es);
    expect(toldFor([first.version], true)[0].told).toEqual(first.en);
    expect(toldFor(["0.0.1"])).toEqual([]);
  });

  it("names a version by its own headline, or by the plain title when it has none", () => {
    const [first] = news;
    expect(headline(first.version)).toBe(first.headline?.es);
    expect(headline("0.0.1")).toBe(t("newsTitle"));
  });
});

describe("the news window", () => {
  it("shows nothing when there is nothing new", async () => {
    ipc.answers.news = () => Promise.resolve([]);
    render(<News />);

    await waitFor(() => expect(sent("news")).toHaveLength(1));
    expect(screen.queryByRole("dialog")).toBeNull();
  });

  it("tells what a version brings and remembers it was told once closed", async () => {
    const [first] = news;
    ipc.answers.news = () => Promise.resolve([first.version]);
    render(<News />);

    expect(await screen.findByRole("dialog")).toBeTruthy();
    expect(screen.getByText(headline(first.version))).toBeTruthy();
    expect(screen.getByText(first.es[0].title)).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: t("newsOk") }));

    expect(screen.queryByRole("dialog")).toBeNull();
    expect(sent("news_seen")).toHaveLength(1);
  });
});
