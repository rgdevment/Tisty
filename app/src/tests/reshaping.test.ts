import { beforeEach, describe, expect, it, vi } from "vitest";
import { adopt } from "../locales";

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
  chosen: "",
  asked: 0,
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    ipc.calls.push({ cmd, args: args ?? {} });
    return Promise.resolve(`![${String(args?.path)}](x)`);
  },
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  message: () => {
    ipc.asked += 1;
    return Promise.resolve(ipc.chosen);
  },
}));

// Only Windows and macOS turn a photo, and the module reads which one it runs on when it loads.
const onWindows = async () => {
  Object.defineProperty(window.navigator, "userAgent", {
    value: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
    configurable: true,
  });
  vi.resetModules();
  return import("../reshaping");
};

beforeEach(() => {
  adopt("en");
  ipc.calls = [];
  ipc.asked = 0;
  ipc.chosen = "";
});

describe("a photo not every computer shows", () => {
  it("is asked about once for all dropped together, and the choice reaches each one", async () => {
    const { attachedAll } = await onWindows();
    ipc.chosen = "Lighter";

    await Promise.all(await attachedAll(["C:/a/uno.HEIC", "C:/a/dos.tiff", "C:/a/tres.png"]));

    expect(ipc.asked).toBe(1);
    const shapes = ipc.calls.map((one) => [one.args.path, one.args.shape]);
    expect(shapes).toEqual([
      ["C:/a/uno.HEIC", "light"],
      ["C:/a/dos.tiff", "light"],
      ["C:/a/tres.png", undefined],
    ]);
  });

  it("is kept as it came when the person says so or closes the question", async () => {
    const { attachedAll } = await onWindows();
    ipc.chosen = "Cancel";

    await Promise.all(await attachedAll(["C:/a/uno.heic"]));

    expect(ipc.calls[0].args.shape).toBeUndefined();
  });

  it("asks nothing when every file shows everywhere", async () => {
    const { attachedAll } = await onWindows();

    await Promise.all(await attachedAll(["C:/a/foto.jpg", "C:/a/acta.pdf"]));

    expect(ipc.asked).toBe(0);
    expect(ipc.calls).toHaveLength(2);
  });
});
