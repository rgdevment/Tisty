import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import type { Filed } from "../core";
import Editor from "../ui/Editor";
import type { Block } from "../ui/Slash";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: () => Promise.resolve(null),
  convertFileSrc: (at: string) => at,
}));

const known: Filed[] = [
  { id: "01A", file: "mac0-0001", title: "Aquí", folder: null, archived: false },
  { id: "01B", file: "mac0-0002", title: "Recetas", folder: null, archived: false },
];

const write = async () => {
  const onWrite = vi.fn();
  const blocks: Block[] = [];
  render(
    <Editor
      value="hola"
      taking
      paper="mac0-0001"
      papers={known}
      label="Paper"
      onWrite={onWrite}
      onBlocks={(all) => blocks.push(...all)}
    />,
  );

  await waitFor(() => expect(blocks.length).toBeGreaterThan(0));
  const put = blocks.find((one) => one.key === "paper");
  if (!put) throw new Error("no block for a document");
  put.run();

  return onWrite;
};

describe("putting one document inside another", () => {
  it("drops it in as a card, with no question in between", async () => {
    const onWrite = await write();

    await userEvent.click(await screen.findByText("Recetas"));

    await waitFor(() => {
      const calls = onWrite.mock.calls;
      expect(calls[calls.length - 1]?.[0]).toContain("![Recetas](tisty:doc/mac0-0002)");
    });
  });

  it("leaves the document it is already in out of the list", async () => {
    await write();

    expect(await screen.findByText("Recetas")).toBeTruthy();
    expect(screen.queryByText("Aquí")).toBeNull();
  });
});
