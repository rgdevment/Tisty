import {
  act,
  cleanup,
  fireEvent,
  render,
  renderHook,
  screen,
  waitFor,
} from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { t } from "../locales";
import About from "../ui/About";
import Alone from "../ui/Alone";
import { useAloneOffer } from "../watching";

const ipc = vi.hoisted(() => ({
  calls: [] as [string, Record<string, unknown> | undefined][],
  answer: (_cmd: string, _args?: Record<string, unknown>): Promise<unknown> =>
    Promise.resolve(null),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    ipc.calls.push([cmd, args]);
    return ipc.answer(cmd, args);
  },
}));

vi.mock("@tauri-apps/plugin-opener", () => ({ openUrl: () => Promise.resolve() }));

const build = {
  version: "0.2.0",
  license: "AGPL-3.0",
  repository: "https://example.invalid/tisty",
  store: "C:/store",
  candidates: false,
  candidatesApply: true,
  autoUpdates: false,
  autoUpdatesApply: true,
};

const automatic = () => ipc.calls.filter(([cmd]) => cmd === "update_automatic");

beforeEach(() => {
  ipc.calls = [];
  ipc.answer = () => Promise.resolve(null);
});

afterEach(cleanup);

describe("the automatic updates box in About", () => {
  const box = () => screen.queryByRole("checkbox", { name: /Install updates automatically/ });

  it("is not shown when this copy cannot install on its own", async () => {
    ipc.answer = () => Promise.resolve({ ...build, autoUpdatesApply: false });
    render(<About ready={null} onError={vi.fn()} />);

    await screen.findByText("0.2.0");
    expect(box()).toBeNull();
  });

  it("is shown when this copy can install on its own", async () => {
    ipc.answer = () => Promise.resolve(build);
    render(<About ready={null} onError={vi.fn()} />);

    expect(
      await screen.findByRole("checkbox", { name: /Install updates automatically/ }),
    ).toBeTruthy();
  });

  it("asks to turn it on when ticked", async () => {
    ipc.answer = () => Promise.resolve(build);
    render(<About ready={null} onError={vi.fn()} />);

    await userEvent.click(
      await screen.findByRole("checkbox", { name: /Install updates automatically/ }),
    );

    await waitFor(() => expect(automatic()).toEqual([["update_automatic", { wants: true }]]));
    expect((box() as HTMLInputElement).checked).toBe(true);
  });

  it("goes back to unticked when turning it on fails", async () => {
    ipc.answer = (cmd) =>
      cmd === "update_automatic"
        ? Promise.reject({ code: "updateNoAnswer" })
        : Promise.resolve(build);
    render(<About ready={null} onError={vi.fn()} />);

    await userEvent.click(
      await screen.findByRole("checkbox", { name: /Install updates automatically/ }),
    );

    await waitFor(() => expect((box() as HTMLInputElement).checked).toBe(false));
    expect(automatic()).toHaveLength(1);
  });
});

describe("the corner that offers automatic updates", () => {
  const corner = () => {
    const onSettled = vi.fn();
    const onError = vi.fn();
    render(<Alone apart="right-3" onSettled={onSettled} onError={onError} />);
    return { onSettled, onError };
  };

  it("turns them on and settles when accepted", async () => {
    const { onSettled } = corner();

    await userEvent.click(screen.getByRole("button", { name: t("aloneGo") }));

    expect(automatic()).toEqual([["update_automatic", { wants: true }]]);
    expect(onSettled).toHaveBeenCalledTimes(1);
  });

  it("turns them off for good and settles when told not to ask again", async () => {
    const { onSettled } = corner();

    await userEvent.click(screen.getByRole("button", { name: t("aloneNo") }));

    expect(automatic()).toEqual([["update_automatic", { wants: false }]]);
    expect(onSettled).toHaveBeenCalledTimes(1);
  });

  it("only settles when put off", async () => {
    const { onSettled } = corner();

    await userEvent.click(screen.getAllByRole("button", { name: t("aloneLater") })[0]);

    expect(automatic()).toEqual([]);
    expect(onSettled).toHaveBeenCalledTimes(1);
  });

  it("is put off by Escape like by the button", () => {
    const { onSettled } = corner();

    fireEvent.keyDown(window, { key: "Escape" });

    expect(automatic()).toEqual([]);
    expect(onSettled).toHaveBeenCalledTimes(1);
  });

  it("says what went wrong when the choice could not be kept", async () => {
    ipc.answer = () => Promise.reject(new Error("nope"));
    const { onError } = corner();

    await userEvent.click(screen.getByRole("button", { name: t("aloneGo") }));

    await waitFor(() => expect(onError).toHaveBeenCalledTimes(1));
  });
});

describe("the offer of automatic updates", () => {
  const due = (value: boolean) => {
    ipc.answer = (cmd) => Promise.resolve(cmd === "update_alone_due" ? value : null);
  };

  it("shows when the copy says it is due", async () => {
    due(true);
    const { result } = renderHook(() => useAloneOffer(true));

    await waitFor(() => expect(result.current.alone).toBe(true));
  });

  it("stays away when the copy says it is not due", async () => {
    due(false);
    const { result } = renderHook(() => useAloneOffer(true));

    await waitFor(() => expect(ipc.calls.map(([cmd]) => cmd)).toContain("update_alone_due"));
    expect(result.current.alone).toBe(false);
  });

  it("does not ask before the first look has happened", () => {
    due(true);
    const { result } = renderHook(() => useAloneOffer(undefined));

    expect(ipc.calls).toEqual([]);
    expect(result.current.alone).toBe(false);
  });

  it("does not come back once settled, even when the look changes", async () => {
    due(true);
    const { result, rerender } = renderHook(({ looked }) => useAloneOffer(looked), {
      initialProps: { looked: 1 },
    });
    await waitFor(() => expect(result.current.alone).toBe(true));

    act(() => result.current.settleAlone());
    expect(result.current.alone).toBe(false);
    const asked = ipc.calls.length;
    rerender({ looked: 2 });

    await act(async () => {});
    expect(result.current.alone).toBe(false);
    expect(ipc.calls).toHaveLength(asked);
  });
});
