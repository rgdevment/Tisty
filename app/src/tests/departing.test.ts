import { afterEach, describe, expect, it, vi } from "vitest";
import { departing } from "../departing";
import { t } from "../locales";
import { forgotten, kept, PATIENCE } from "../saving";
import { asksSure } from "../sure";

const ipc = vi.hoisted(() => ({ calls: [] as string[] }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) => {
    ipc.calls.push(cmd);
    return Promise.resolve(null);
  },
}));

const never = <T>() => new Promise<T>(() => {});

const answering = (yes: boolean) => {
  const asked: string[] = [];
  asksSure((said) => {
    asked.push(said);
    return Promise.resolve(yes);
  });
  return asked;
};

afterEach(() => {
  asksSure(null);
  ipc.calls.length = 0;
});

describe("closing Tisty", () => {
  it("leaves at once when every save answered", async () => {
    await kept("cerrado", "uno", async () => "uno");

    await departing();

    expect(ipc.calls).toEqual(["parted"]);
  });

  it("keeps the app from leaving while a save has not answered, and stays when the person says so", async () => {
    void kept("cuelga", "leche y pan", never).catch(() => {});
    const asked = answering(false);

    const going = departing();
    await Promise.resolve();
    expect(ipc.calls).toEqual(["parting_waits"]);
    await vi.advanceTimersByTimeAsync(PATIENCE);
    await going;

    expect(asked).toEqual([t("partingUnsaved")]);
    expect(ipc.calls).toEqual(["parting_waits", "parting_waits", "parting_waits", "parting_stays"]);
    forgotten("cuelga");
  });

  it("shows the window within a second while it waits, never a quiet wait from the tray", async () => {
    void kept("bandeja", "sin respuesta", never).catch(() => {});
    answering(false);

    const going = departing();
    await vi.advanceTimersByTimeAsync(1000);
    expect(ipc.calls).toEqual(["parting_waits", "parting_waits"]);
    await vi.advanceTimersByTimeAsync(PATIENCE);
    await going;
    forgotten("bandeja");
  });

  it("can always be closed again when asking the person breaks", async () => {
    void kept("rompe", "palabras", never).catch(() => {});
    asksSure(() => Promise.reject(new Error("the question could not be shown")));

    const going = departing();
    await vi.advanceTimersByTimeAsync(PATIENCE);
    await going;

    expect(ipc.calls.slice(-1)).toEqual(["parting_stays"]);
    forgotten("rompe");
  });

  it("leaves without the words only once the person chose to, and writes that down", async () => {
    void kept("cuelga otra vez", "huevos", never).catch(() => {});
    const asked = answering(true);

    const going = departing();
    await vi.advanceTimersByTimeAsync(PATIENCE);
    await going;

    expect(asked).toEqual([t("partingUnsaved")]);
    expect(ipc.calls.slice(-2)).toEqual(["note_trouble", "parted"]);
    expect(ipc.calls).not.toContain("parting_stays");
    forgotten("cuelga otra vez");
  });
});
