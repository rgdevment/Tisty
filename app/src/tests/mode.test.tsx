import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Carrying } from "../core";
import { fill, t } from "../locales";
import Mode, { modeSaid } from "../ui/Mode";

const ipc = vi.hoisted(() => ({ state: null as unknown }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: () => Promise.resolve(ipc.state),
}));
vi.mock("@tauri-apps/api/event", () => ({ listen: () => Promise.resolve(() => {}) }));

const state = (over: Partial<Carrying>): Carrying =>
  ({ asked: true, mode: "alone", freesSpace: false, ...over }) as Carrying;

beforeEach(() => {
  ipc.state = null;
});

describe("the way Tisty syncs, said in a word", () => {
  it("names the folder by its last part, wherever the separators lean", () => {
    expect(modeSaid(state({ mode: "folder", chosen: "C:\\Users\\Ana\\Dropbox\\Tisty" }))).toBe(
      fill("modeFolder", "Tisty"),
    );
    expect(modeSaid(state({ mode: "folder", chosen: "/Volumes/nas/tisty/" }))).toBe(
      fill("modeFolder", "tisty"),
    );
  });

  it("says when it keeps to this machine, or syncs a way a newer Tisty chose", () => {
    expect(modeSaid(state({ mode: "alone" }))).toBe(t("modeAlone"));
    expect(modeSaid(state({ mode: "later" }))).toBe(t("modeLater"));
  });

  it("opens the settings that change it", async () => {
    ipc.state = state({ mode: "folder", chosen: "/nas/tisty" });
    const open = vi.fn();
    render(<Mode onOpen={open} />);

    await userEvent.click(await screen.findByText(fill("modeFolder", "tisty")));

    expect(open).toHaveBeenCalled();
  });

  it("says nothing before the person has chosen", async () => {
    ipc.state = state({ asked: false });
    const { container } = render(<Mode onOpen={() => {}} />);

    await new Promise((done) => setTimeout(done, 0));
    expect(container.textContent).toBe("");
  });
});
