import { render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { ONCE_A_DAY } from "../inUse";
import Composed from "../ui/Composed";
import { composed } from "./crated";

const { served } = vi.hoisted(() => ({
  served: vi.fn((reference: string) => Promise.resolve(`C:\\data\\${reference}`)),
}));

vi.mock("../core", async () => {
  const { crated } = await import("./crated");
  return {
    served,
    noteDrawn: (body: string) => Promise.resolve(crated(body)),
    mendsIn: () => Promise.resolve([]),
  };
});

vi.mock("@tauri-apps/api/core", () => ({
  convertFileSrc: (at: string) => `http://asset.localhost/${encodeURIComponent(at)}`,
}));

vi.mock("@tauri-apps/plugin-opener", () => ({
  openPath: vi.fn(),
  openUrl: vi.fn(),
}));

const TWO = "![one](<attachments/aa/1.png>)\n\n![two](<attachments/bb/2.png>)";
const resolved = () => document.querySelectorAll('img[src^="http"]').length;

describe("more than one image", () => {
  it("resolves every image on a plain mount", async () => {
    render(<Composed html={composed(TWO)} className="prose" />);
    await waitFor(() => expect(resolved()).toBe(2));
  });

  it("keeps them when the parent re-renders with the very same html", async () => {
    const { rerender } = render(<Composed html={composed(TWO)} className="prose" />);
    await waitFor(() => expect(resolved()).toBe(2));

    rerender(<Composed html={composed(TWO)} className="prose" />);
    expect(resolved()).toBe(2);
  });
});

describe("a picture left on screen", () => {
  it("is asked for again each day it stays, and no longer once it is gone", async () => {
    const one = "![tres](<attachments/cc/3.png>)";
    const asked = () => served.mock.calls.filter(([at]) => at === "attachments/cc/3.png").length;
    vi.useFakeTimers();
    try {
      const shown = render(<Composed html={composed(one)} className="prose" />);
      await vi.advanceTimersByTimeAsync(0);
      const before = asked();

      await vi.advanceTimersByTimeAsync(ONCE_A_DAY);
      expect(asked()).toBe(before + 1);

      shown.unmount();
      await vi.advanceTimersByTimeAsync(ONCE_A_DAY);
      expect(asked()).toBe(before + 1);
    } finally {
      vi.useRealTimers();
    }
  });
});

describe("a picture already showing", () => {
  it("stays when asking for it again fails for a while", async () => {
    const one = "![cuatro](<attachments/dd/4.png>)";
    const first = render(<Composed html={composed(one)} className="prose" />);
    await waitFor(() => expect(resolved()).toBe(1));
    first.unmount();
    served.mockImplementationOnce(() => Promise.reject({ code: "cloudNotNow" }));

    render(<Composed html={composed(one)} className="prose" />);
    await new Promise((settled) => setTimeout(settled, 0));

    expect(resolved()).toBe(1);
    expect(screen.queryByText(/cuatro/)).toBeNull();
  });
});

describe("an attachment that cannot be drawn, inside a task", () => {
  const chip = () => document.querySelector<HTMLAnchorElement>("a.chip");

  it("becomes a chip rather than a picture that will never load", async () => {
    render(
      <Composed html={composed("![el contrato](<attachments/ab/cd.pdf>)")} className="prose" />,
    );

    await waitFor(() => expect(chip()).toBeTruthy());
    expect(chip()?.textContent).toContain("el contrato");
    expect(document.querySelector("img")).toBeNull();
  });

  it("says what kind of thing it is", async () => {
    render(
      <Composed html={composed("![el contrato](<attachments/ab/cd.pdf>)")} className="prose" />,
    );

    await waitFor(() => expect(chip()).toBeTruthy());
    expect(chip()?.querySelector(".chip-badge")?.textContent).toBe("PDF");
  });

  it("falls back to the file name when nobody wrote one", async () => {
    render(<Composed html={composed("![](<attachments/ab/cd.pdf>)")} className="prose" />);

    await waitFor(() => expect(chip()).toBeTruthy());
    expect(chip()?.textContent).toContain("cd.pdf");
  });

  it("points a document chip at the document, not at a file to open", async () => {
    render(<Composed html={composed("![el informe](tisty:doc/mac0-0007)")} className="prose" />);

    await waitFor(() => expect(chip()).toBeTruthy());
    expect(chip()?.getAttribute("href")).toBe("tisty:doc/mac0-0007");
  });

  it("reads as a plain link, since a document needs no badge to say what it is", async () => {
    render(<Composed html={composed("![el informe](tisty:doc/mac0-0007)")} className="prose" />);

    await waitFor(() => expect(chip()).toBeTruthy());
    expect(chip()?.classList.contains("chip-paper")).toBe(true);
  });

  it("wears the shape a document wears everywhere else, not a word for it", async () => {
    render(<Composed html={composed("![el informe](tisty:doc/mac0-0007)")} className="prose" />);

    await waitFor(() => expect(chip()).toBeTruthy());
    const badge = chip()?.querySelector(".chip-badge");
    expect(badge?.querySelector("svg")).toBeTruthy();
    expect(badge?.textContent?.trim()).toBe("");
  });

  it("still names the kind of an attachment, which the shape cannot say", async () => {
    render(<Composed html={composed("![hoja](<attachments/aa/x.xlsx>)")} className="prose" />);

    await waitFor(() => expect(chip()).toBeTruthy());
    expect(chip()?.querySelector(".chip-badge")?.textContent).toBe("XLSX");
  });

  it("leaves a real picture alone", async () => {
    render(<Composed html={composed("![una foto](<attachments/aa/1.png>)")} className="prose" />);

    await waitFor(() => expect(resolved()).toBe(1));
    expect(chip()).toBeNull();
  });
});
