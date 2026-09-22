import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import Star from "../ui/Star";

const opened = vi.hoisted(() => ({ urls: [] as string[], refuse: false }));

vi.mock("@tauri-apps/plugin-opener", () => ({
  openUrl: (url: string) => {
    if (opened.refuse) return Promise.reject(new Error("no browser answered"));
    opened.urls.push(url);
    return Promise.resolve();
  },
}));

const ipc = vi.hoisted(() => ({ sent: [] as string[] }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) => {
    ipc.sent.push(cmd);
    return Promise.resolve(null);
  },
}));

function shown() {
  const said = { settled: 0, problems: [] as unknown[] };
  render(
    <Star
      apart="right-3"
      onSettled={() => (said.settled += 1)}
      onError={(problem) => said.problems.push(problem)}
    />,
  );
  return said;
}

beforeEach(() => {
  opened.urls = [];
  opened.refuse = false;
  ipc.sent = [];
});

describe("the card that asks for a star", () => {
  it("announces itself and asks nothing until it is answered", () => {
    shown();

    expect(screen.getByRole("status")).toBeTruthy();
    expect(ipc.sent).toEqual([]);
    expect(opened.urls).toEqual([]);
  });

  it("opens the repository and never comes back", async () => {
    const said = shown();

    await userEvent.click(screen.getByRole("button", { name: /star on github/i }));

    expect(opened.urls).toEqual(["https://github.com/rgdevment/Tisty"]);
    expect(ipc.sent).toEqual(["star_done"]);
    expect(said.settled).toBe(1);
  });

  it("goes for good on a refusal too, without reaching the network", async () => {
    const said = shown();

    await userEvent.click(screen.getByRole("button", { name: /don't show again/i }));

    expect(opened.urls).toEqual([]);
    expect(ipc.sent).toEqual(["star_done"]);
    expect(said.settled).toBe(1);
  });

  it("is only silenced by the cross, which is not an answer", async () => {
    const said = shown();

    await userEvent.click(screen.getByRole("button", { name: /not now/i }));

    expect(said.settled).toBe(1);
    expect(ipc.sent).toEqual([]);
    expect(opened.urls).toEqual([]);
  });

  it("is silenced by Escape from anywhere, not only from inside it", async () => {
    const said = shown();

    document.body.focus();
    await userEvent.keyboard("{Escape}");

    expect(said.settled).toBe(1);
    expect(ipc.sent).toEqual([]);
  });

  it("says so when the link cannot be opened, instead of swallowing the click", async () => {
    opened.refuse = true;
    const said = shown();

    await userEvent.click(screen.getByRole("button", { name: /star on github/i }));

    expect(said.problems.length).toBe(1);
    expect(String(said.problems[0])).toMatch(/no browser answered/);
  });
});
