import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import Docs from "../ui/Docs";
import Tree from "../ui/Tree";
import type { Filed, Papers } from "../core";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) => {
    if (cmd === "doc_read") return Promise.resolve("");
    if (cmd === "icons") return Promise.resolve([["work", "💼"]]);
    return Promise.resolve(null);
  },
}));

const known: Filed[] = [{ id: "01A", file: "914kqe8z-0001", title: "", folder: null }];
const papers: Papers = { folders: [], docs: known };

describe("a document that was left with no title", () => {
  it("opens without taking the window down", async () => {
    render(<Docs open="914kqe8z-0001" known={known} onKept={vi.fn()} onError={vi.fn()} />);

    await waitFor(() => expect(document.querySelector(".tisty-doc")).toBeTruthy());
  });

  it("survives being swapped for another document", async () => {
    const second: Filed[] = [
      ...known,
      { id: "01B", file: "914kqe8z-0002", title: "Notas", folder: null },
    ];
    const { rerender } = render(
      <Docs open="914kqe8z-0001" known={second} onKept={vi.fn()} onError={vi.fn()} />,
    );
    await waitFor(() => expect(document.querySelector(".tisty-doc")).toBeTruthy());

    rerender(<Docs open="914kqe8z-0002" known={second} onKept={vi.fn()} onError={vi.fn()} />);
    rerender(<Docs open="914kqe8z-0001" known={second} onKept={vi.fn()} onError={vi.fn()} />);

    await waitFor(() => expect(document.querySelector(".tisty-doc")).toBeTruthy());
  });

  it("can be picked from the tree", async () => {
    const onOpen = vi.fn();
    render(<Tree papers={papers} onOpen={onOpen} onFile={vi.fn()} />);

    await userEvent.click(screen.getByRole("button", { name: "Untitled" }));

    expect(onOpen.mock.calls[0][0].file).toBe("914kqe8z-0001");
  });
});
