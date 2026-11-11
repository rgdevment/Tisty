import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { adopt, t } from "../locales";
import type { LanguageWord } from "../sharing";
import LanguageReview from "../ui/LanguageReview";

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
  prints: [] as string[],
  refuse: null as null | string,
  changes: null as null | { kind: "same" | "gone" | "came"; text: string }[],
}));

vi.mock("@tauri-apps/api/core", () => ({
  convertFileSrc: (at: string) => `asset://${at}`,
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    ipc.calls.push({ cmd, args: args ?? {} });
    if (cmd === "doc_language_shown")
      return Promise.resolve({
        document: "anaEngl1",
        lang: "en",
        title: "Walks",
        by: "Ana",
        body: "We walk by the river.",
        pictures: [{ at: 0, path: "/cache/reviewing/0.png" }],
        print: ipc.prints.shift() ?? "P",
        changes: ipc.changes,
      });
    if (cmd === "doc_language_reviewed" && ipc.refuse) {
      const code = ipc.refuse;
      ipc.refuse = null;
      return Promise.reject({ code });
    }
    return Promise.resolve(null);
  },
}));

const word: LanguageWord = {
  document: "anaEngl1",
  lang: "en",
  title: null,
  word: "waiting",
  mine: false,
};

const sent = (cmd: string) => ipc.calls.filter((one) => one.cmd === cmd);

beforeEach(() => {
  ipc.calls = [];
  ipc.prints = [];
  ipc.refuse = null;
  ipc.changes = null;
  adopt("en");
});

describe("Review a language", () => {
  it("shows the language with its pictures and keeps the very version shown", async () => {
    ipc.prints = ["P1"];
    const onDone = vi.fn();
    render(<LanguageReview doc="d1" word={word} onDone={onDone} onClose={vi.fn()} />);

    await screen.findByText("We walk by the river.");
    expect(screen.getByText("Added by Ana")).toBeTruthy();
    expect(
      screen.getByRole("img", { name: "Picture 1 of this language" }).getAttribute("src"),
    ).toBe("asset:///cache/reviewing/0.png");
    await userEvent.click(screen.getByRole("button", { name: t("reviewKeep") }));

    await waitFor(() => expect(onDone).toHaveBeenCalled());
    expect(sent("doc_language_reviewed")[0].args).toEqual({
      id: "d1",
      document: "anaEngl1",
      keep: true,
      print: "P1",
    });
  });

  it("leaves a language out with no print", async () => {
    const onDone = vi.fn();
    render(<LanguageReview doc="d1" word={word} onDone={onDone} onClose={vi.fn()} />);
    await screen.findByText("We walk by the river.");

    await userEvent.click(screen.getByRole("button", { name: t("reviewLeaveOut") }));

    await waitFor(() => expect(onDone).toHaveBeenCalled());
    expect(sent("doc_language_reviewed")[0].args).toMatchObject({ keep: false, print: null });
  });

  it("shows a language that changed again, and keeps nothing until it is looked at", async () => {
    ipc.prints = ["P1", "P2"];
    ipc.refuse = "folioLanguageChanged";
    const onDone = vi.fn();
    render(<LanguageReview doc="d1" word={word} onDone={onDone} onClose={vi.fn()} />);
    await screen.findByText("We walk by the river.");

    await userEvent.click(screen.getByRole("button", { name: t("reviewKeep") }));

    await screen.findByText(t("folioLanguageChanged"));
    await waitFor(() => expect(sent("doc_language_shown").length).toBe(2));
    expect(onDone).not.toHaveBeenCalled();
  });

  it("shows what changed since the owner kept it, struck out and marked", async () => {
    ipc.changes = [
      { kind: "same", text: "One." },
      { kind: "gone", text: "Two." },
      { kind: "came", text: "Three." },
    ];
    const changed = { ...word, word: "changed" as const };
    render(<LanguageReview doc="d1" word={changed} onDone={vi.fn()} onClose={vi.fn()} />);

    expect((await screen.findByText("Two.")).tagName).toBe("DEL");
    expect(screen.getByText("Three.").tagName).toBe("INS");
    expect(screen.getByText("Changes in English")).toBeTruthy();
    expect(screen.queryByText("We walk by the river.")).toBeNull();
    expect(screen.queryByText(t("reviewNoCopy"))).toBeNull();
  });

  it("shows a changed language whole when this computer holds no copy of what was kept", async () => {
    const changed = { ...word, word: "changed" as const };
    render(<LanguageReview doc="d1" word={changed} onDone={vi.fn()} onClose={vi.fn()} />);

    await screen.findByText("We walk by the river.");
    expect(screen.getByText(t("reviewNoCopy"))).toBeTruthy();
  });
});
