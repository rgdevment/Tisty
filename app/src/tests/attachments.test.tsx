import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { composed } from "../markdown";
import Composed from "../ui/Composed";
import Prose from "../ui/Prose";

vi.mock("../core", () => ({
  served: (reference: string) => Promise.resolve(`C:\\data\\${reference}`),
}));

vi.mock("@tauri-apps/api/core", () => ({
  convertFileSrc: (at: string) => `http://asset.localhost/${encodeURIComponent(at)}`,
}));

vi.mock("@tauri-apps/plugin-opener", () => ({ openPath: vi.fn(), openUrl: vi.fn() }));

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

describe("the description field, which is the one that takes files", () => {
  it("shows its images when it does not take files", async () => {
    render(<Prose value={TWO} hint="h" label="Journal" onWrite={vi.fn()} />);
    await waitFor(() => expect(resolved()).toBe(2));
  });

  it("shows its images when it does take files", async () => {
    render(<Prose value={TWO} hint="h" label="Description" catches onWrite={vi.fn()} />);
    await waitFor(() => expect(resolved()).toBe(2));
  });

  it("keeps them after entering and leaving the source", async () => {
    const user = userEvent.setup();
    render(<Prose value={TWO} hint="h" label="Description" catches onWrite={vi.fn()} />);
    await waitFor(() => expect(resolved()).toBe(2));

    await user.click(screen.getByLabelText("Description"));
    await user.tab();
    await waitFor(() => expect(resolved()).toBe(2));
  });

  it("shows them in the second column of the full screen view", async () => {
    const user = userEvent.setup();
    render(
      <Prose value={TWO} hint="h" label="Description" catches beside onWrite={vi.fn()} />,
    );
    await user.click(screen.getByLabelText("Description"));
    expect(screen.getByLabelText("Composed")).toBeTruthy();
    await waitFor(() => expect(resolved()).toBe(2));
  });
});

describe("an attachment that cannot be drawn, inside a task", () => {
  const chip = () => document.querySelector<HTMLAnchorElement>("a.chip");

  it("becomes a chip rather than a picture that will never load", async () => {
    render(<Composed html={composed("![el contrato](<attachments/ab/cd.pdf>)")} className="prose" />);

    await waitFor(() => expect(chip()).toBeTruthy());
    expect(chip()?.textContent).toContain("el contrato");
    expect(document.querySelector("img")).toBeNull();
  });

  it("says what kind of thing it is", async () => {
    render(<Composed html={composed("![el contrato](<attachments/ab/cd.pdf>)")} className="prose" />);

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
    expect(chip()?.querySelector(".chip-badge")?.textContent).toBe("DOC");
  });

  it("leaves a real picture alone", async () => {
    render(<Composed html={composed("![una foto](<attachments/aa/1.png>)")} className="prose" />);

    await waitFor(() => expect(resolved()).toBe(1));
    expect(chip()).toBeNull();
  });
});
