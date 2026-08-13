import { readFileSync } from "node:fs";
import { render, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import Editor from "../ui/Editor";

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => Promise.resolve(null) }));

describe("writing help the system provides", () => {
  it("asks the webview to check spelling as you type", async () => {
    render(<Editor value="" onWrite={vi.fn()} />);

    await waitFor(() => {
      const box = document.querySelector(".tisty-doc");
      expect(box?.getAttribute("spellcheck")).toBe("true");
    });
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
