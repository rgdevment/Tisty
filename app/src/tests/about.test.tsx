import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import About from "../ui/About";

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
  ipc.tries = 0;
  ipc.answer = () => Promise.resolve(build);
});

describe("the about screen", () => {
  it("says what this build is", async () => {
    render(<About onError={() => {}} />);

    expect(await screen.findByText("0.1.0")).toBeTruthy();
  });

  /// It used to hand the failure to the window banner and draw nothing: the one
  /// screen you go to when something is wrong had no way to ask again. Its
  /// sister screen has had an error card with a retry all along.
  it("recovers from its own failure", async () => {
    ipc.answer = () => Promise.reject(new Error("the store is not readable"));
    render(<About onError={() => {}} />);

    expect(await screen.findByRole("alert")).toBeTruthy();
    expect(screen.getByText(/not readable/)).toBeTruthy();

    ipc.answer = () => Promise.resolve(build);
    await userEvent.click(screen.getByRole("button", { name: /try again/i }));

    expect(await screen.findByText("0.1.0")).toBeTruthy();
    await waitFor(() => expect(ipc.tries).toBe(2));
    expect(screen.queryByRole("alert")).toBeNull();
  });
});
