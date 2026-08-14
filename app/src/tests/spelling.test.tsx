import { readFileSync } from "node:fs";
import { render, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import Editor from "../ui/Editor";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: () => Promise.resolve(null),
  convertFileSrc: (at: string) => at,
}));

vi.mock("../core", async () => ({
  ...(await vi.importActual<typeof import("../core")>("../core")),
  served: () => Promise.reject({ code: "cannotRead" }),
  weighs: () => Promise.reject({ code: "cannotRead" }),
  noteTrouble: vi.fn(() => Promise.resolve()),
}));

describe("writing help the system provides", () => {
  it("asks the webview to check spelling as you type", async () => {
    render(<Editor value="" onWrite={vi.fn()} />);

    await waitFor(() => {
      const box = document.querySelector(".tisty-doc");
      expect(box?.getAttribute("spellcheck")).toBe("true");
    });
  });

  it("never rewrites the document with what the editor itself just wrote", async () => {
    const { stale } = await import("../ui/Editor");
    const shown = () => "hola";

    expect(stale("hola", "hola", shown)).toBe(false);
    expect(stale("hola\n", "hola\n", shown)).toBe(false);
  });

  it("rewrites only when the words came from somewhere else", async () => {
    const { stale } = await import("../ui/Editor");

    expect(stale("de fuera", "lo mio", () => "lo mio")).toBe(true);
    expect(stale("lo mio", "otra cosa", () => "lo mio")).toBe(false);
  });

  it("says in the log when an attachment cannot be reached, instead of swallowing it", async () => {
    const { noteTrouble } = await import("../core");
    render(
      <Editor
        value="[charla](<attachments/ab/ausente.mp4>)"
        papers={[]}
        onWrite={vi.fn()}
      />,
    );

    await waitFor(() => expect(noteTrouble).toHaveBeenCalledWith("cannotRead"));
  });

  it("tells it which language to check in, or it checks in the wrong one", () => {
    const start = readFileSync("src/main.tsx", "utf8");

    expect(start).toMatch(/documentElement\.lang\s*=\s*locale\(\)/);
  });

  it("leaves the native menu alone where you write, and only there", () => {
    const start = readFileSync("src/main.tsx", "utf8");

    expect(start).toContain("contenteditable");
    expect(start).toMatch(/if \(!writes\) e\.preventDefault\(\)/);
  });
});
