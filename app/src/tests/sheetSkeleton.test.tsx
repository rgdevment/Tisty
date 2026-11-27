import { render, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import Sheet from "../ui/Sheet";

const ipc = vi.hoisted(() => ({ holding: false }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) => {
    if (cmd === "doc_drawn" || cmd === "doc_placed") {
      if (ipc.holding) return new Promise(() => {});
      return Promise.resolve(new TextEncoder().encode('<p data-at="0-4">uno</p>').buffer);
    }
    return Promise.resolve(null);
  },
  convertFileSrc: (path: string) => path,
}));

const loaded = async (frame: HTMLIFrameElement) => {
  await waitFor(() => expect(frame.srcdoc).toContain("<article"));
  const page = frame.contentDocument as Document;
  page.open();
  page.write(frame.srcdoc);
  page.close();
  frame.dispatchEvent(new Event("load"));
  return page;
};

const pulsing = (container: HTMLElement) =>
  container.querySelector('[aria-hidden="true"][class*="animate-pulse"]');

beforeEach(() => {
  ipc.holding = false;
});

describe("the loading skeleton of a sheet", () => {
  it("shows while the very first page is still being drawn", async () => {
    ipc.holding = true;
    const view = render(<Sheet value="uno" label="Uno" file="uno.md" onWrite={() => {}} />);

    await waitFor(() => expect(pulsing(view.container)).toBeTruthy());
  });

  it("stays away when the open document is switched, until the new page is drawn", async () => {
    const view = render(<Sheet value="uno" label="Uno" file="uno.md" onWrite={() => {}} />);
    const frame = await waitFor(() => {
      const found = view.container.querySelector("iframe");
      expect(found).toBeTruthy();
      return found as HTMLIFrameElement;
    });
    await loaded(frame);
    await waitFor(() => expect(pulsing(view.container)).toBeNull());

    ipc.holding = true;
    view.rerender(<Sheet value="dos" label="Uno" file="uno.en.md" onWrite={() => {}} />);
    await new Promise((settled) => setTimeout(settled, 50));

    expect(pulsing(view.container)).toBeNull();
  });
});
