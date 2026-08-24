import { readFileSync } from "node:fs";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import Editor from "../ui/Editor";
import Shot from "../ui/Shot";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: () => Promise.resolve(null),
  convertFileSrc: (at: string) => `http://asset.localhost/${encodeURIComponent(at)}`,
}));

vi.mock("../core", async () => ({
  ...(await vi.importActual<typeof import("../core")>("../core")),
  served: (at: string) => Promise.resolve(`C:/data/${at}`),
  weighs: () => Promise.resolve(370089),
  noteTrouble: vi.fn(() => Promise.resolve()),
}));

const BOTH = [
  "![foto.png](<attachments/aa/foto-1c0a.png>)",
  "![contrato.pdf](<attachments/19/contrato-1c0a.pdf>)",
].join("\n\n");

describe("an attachment the editor shows as a card", () => {
  it("never hands the file to a picture that could only draw it broken", async () => {
    render(<Editor value={BOTH} onWrite={vi.fn()} />);

    await waitFor(() => expect(document.querySelector('img[src^="http"]')).toBeTruthy());
    const carded = document.querySelector("img.card-source");

    expect(carded?.getAttribute("src")).toBe("attachments/19/contrato-1c0a.pdf");
  });

  it("hides that picture against the rule that lays every picture out", () => {
    const css = readFileSync("src/index.css", "utf8");
    const at = css.indexOf(".card-source");

    expect(at).toBeGreaterThan(-1);
    expect(css.slice(at, css.indexOf("}", at))).toContain(".tisty-doc img.card-source");
  });

  it("offers to keep a copy of the file wherever the person wants it", async () => {
    const kept = vi.fn();
    render(<Editor value={BOTH} onWrite={vi.fn()} onKeep={kept} />);

    await waitFor(() => expect(document.querySelector(".card-name")).toBeTruthy());
    await userEvent.click(screen.getByRole("button", { name: "More" }));
    await userEvent.click(await screen.findByRole("menuitem", { name: /save a copy/i }));

    expect(kept).toHaveBeenCalledWith("attachments/19/contrato-1c0a.pdf", "contrato.pdf");
  });

  it("says nothing of keeping a copy when there is nowhere to keep it to", async () => {
    render(<Editor value={BOTH} onWrite={vi.fn()} />);

    await waitFor(() => expect(document.querySelector(".card-name")).toBeTruthy());
    await userEvent.click(screen.getByRole("button", { name: "More" }));

    expect(screen.queryByRole("menuitem", { name: /save a copy/i })).toBeNull();
  });
});

describe("the handles a picked photo offers", () => {
  it("keeps a copy of the photo where the person wants it", async () => {
    const kept = vi.fn();
    render(<Shot at={{ x: 10, y: 40 }} onOpen={vi.fn()} onKeep={kept} onDrop={vi.fn()} />);

    await userEvent.click(screen.getByRole("button", { name: /save a copy/i }));

    expect(kept).toHaveBeenCalled();
  });

  it("offers nothing of the sort when there is nowhere to keep it to", () => {
    render(<Shot at={{ x: 10, y: 40 }} onOpen={vi.fn()} onDrop={vi.fn()} />);

    expect(screen.queryByRole("button", { name: /save a copy/i })).toBeNull();
  });
});
