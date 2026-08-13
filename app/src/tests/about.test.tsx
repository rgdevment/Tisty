import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import About from "../ui/About";

const opened = vi.hoisted(() => ({ urls: [] as string[] }));

vi.mock("@tauri-apps/plugin-opener", () => ({
  openUrl: (url: string) => {
    opened.urls.push(url);
    return Promise.resolve();
  },
}));

const ipc = vi.hoisted(() => ({
  tries: 0,
  answer: (_cmd: string): Promise<unknown> => Promise.resolve(null),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) => {
    if (cmd === "about") ipc.tries += 1;
    return ipc.answer(cmd);
  },
}));

const build = {
  version: "0.1.0",
  license: "AGPL-3.0",
  repository: "https://github.com/rgdevment/tisty",
  store: "C:/Users/x/AppData/Roaming/tisty",
};

beforeEach(() => {
  opened.urls = [];
  ipc.tries = 0;
  ipc.answer = () => Promise.resolve(build);
});

describe("the about screen", () => {
  it("says what this build is", async () => {
    render(<About ready={null} onError={() => {}} />);

    expect(await screen.findByText("0.1.0")).toBeTruthy();
  });

  it("recovers from its own failure", async () => {
    ipc.answer = () => Promise.reject(new Error("the store is not readable"));
    render(<About ready={null} onError={() => {}} />);

    expect(await screen.findByRole("alert")).toBeTruthy();
    expect(screen.getByText(/not readable/)).toBeTruthy();

    ipc.answer = () => Promise.resolve(build);
    await userEvent.click(screen.getByRole("button", { name: /try again/i }));

    expect(await screen.findByText("0.1.0")).toBeTruthy();
    await waitFor(() => expect(ipc.tries).toBe(2));
    expect(screen.queryByRole("alert")).toBeNull();
  });
});

describe("the other tools", () => {
  it("points each one at its own repository", async () => {
    render(<About ready={null} onError={() => {}} />);
    await screen.findByText("0.1.0");

    await userEvent.click(screen.getByRole("button", { name: /CopyPaste/ }));
    await userEvent.click(screen.getByRole("button", { name: /LinkUnbound/ }));

    expect(opened.urls).toEqual([
      "https://github.com/rgdevment/CopyPaste",
      "https://github.com/rgdevment/LinkUnbound",
    ]);
  });

  it("draws an icon for each, not a bullet", async () => {
    const { container } = render(<About ready={null} onError={() => {}} />);
    await screen.findByText("0.1.0");

    expect(container.querySelectorAll("img").length).toBe(2);
  });

  it("opens Tisty's own repository as a url", async () => {
    render(<About ready={null} onError={() => {}} />);
    await screen.findByText("0.1.0");

    await userEvent.click(screen.getByRole("button", { name: /repository/i }));

    expect(opened.urls).toContain("https://github.com/rgdevment/tisty");
  });
});
