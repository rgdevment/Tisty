import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import Star from "../ui/Star";

const opened = vi.hoisted(() => ({ urls: [] as string[] }));

vi.mock("@tauri-apps/plugin-opener", () => ({
  openUrl: (url: string) => {
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

beforeEach(() => {
  opened.urls = [];
  ipc.sent = [];
});

describe("the card that asks for a star", () => {
  it("opens the repository and never asks again", async () => {
    let settled = 0;
    render(<Star apart="right-3" onSettled={() => (settled += 1)} />);

    await userEvent.click(screen.getByRole("button", { name: /star on github/i }));

    expect(opened.urls).toEqual(["https://github.com/rgdevment/Tisty"]);
    expect(ipc.sent).toEqual(["star_done"]);
    expect(settled).toBe(1);
  });

  it("goes for good on a refusal too, without reaching the network", async () => {
    let settled = 0;
    render(<Star apart="right-3" onSettled={() => (settled += 1)} />);

    await userEvent.click(screen.getByRole("button", { name: /no thanks/i }));

    expect(opened.urls).toEqual([]);
    expect(ipc.sent).toEqual(["star_done"]);
    expect(settled).toBe(1);
  });
});
