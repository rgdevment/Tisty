import { fireEvent, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import TableEditor from "../ui/TableEditor";

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => Promise.resolve(null) }));
vi.mock("../core", async () => ({
  ...(await vi.importActual<typeof import("../core")>("../core")),
  docs: () =>
    Promise.resolve({
      folders: [],
      docs: [
        {
          id: "01A",
          file: "mac0-0001",
          title: "Contrato",
          folder: null,
          archived: false,
          away: false,
        },
      ],
    }),
}));

const opened = (kind: "date" | "select" | "tags" | "link", was = "", options: string[] = []) => {
  const onWrite = vi.fn();
  const onClose = vi.fn();
  render(
    <TableEditor
      kind={kind}
      at={{ x: 10, y: 10 }}
      was={was}
      options={options}
      onWrite={onWrite}
      onClose={onClose}
    />,
  );
  return { onWrite, onClose };
};

describe("a cell written by picking", () => {
  it("takes a date said in words, or one picked from the calendar", async () => {
    const { onWrite } = opened("date", "2026-11-14");
    const field = screen.getByRole("textbox", { name: /Or type it/ });
    expect(document.activeElement).toBe(field);
    await userEvent.type(field, "next friday{Enter}");
    expect(onWrite).toHaveBeenLastCalledWith("next friday");
    const day = screen.getAllByRole("button").find((one) => one.textContent === "20");
    if (day) fireEvent.click(day);
    expect(onWrite).toHaveBeenLastCalledWith("2026-11-20");
  });

  it("offers the column's options, a new one typed in, and a way to empty it", async () => {
    const { onWrite } = opened("select", "Música", ["Música", "Video"]);
    expect(screen.getByRole("button", { name: "Música" }).getAttribute("aria-pressed")).toBe(
      "true",
    );
    fireEvent.click(screen.getByRole("button", { name: "Video" }));
    expect(onWrite).toHaveBeenLastCalledWith("Video");
    await userEvent.type(screen.getByRole("textbox", { name: "New option" }), "Nube{Enter}");
    expect(onWrite).toHaveBeenLastCalledWith("Nube");
    fireEvent.click(screen.getByRole("button", { name: "Clear" }));
    expect(onWrite).toHaveBeenLastCalledWith("");
  });

  it("chooses tags a few at a time and writes them once, when it is left", async () => {
    const { onWrite, onClose } = opened("tags", "web, docs", ["web", "docs", "diseño"]);
    fireEvent.click(screen.getByRole("button", { name: "docs" }));
    fireEvent.click(screen.getByRole("button", { name: "diseño" }));
    await userEvent.type(screen.getByRole("textbox", { name: "New tag" }), "urgente{Enter}");
    expect(onWrite).not.toHaveBeenCalled();
    fireEvent.keyDown(screen.getByRole("dialog"), { key: "Escape" });
    expect(onWrite).toHaveBeenCalledWith("web, diseño, urgente");
    expect(onClose).not.toHaveBeenCalled();
  });

  it("closes without writing when the tags were left as they were", () => {
    const { onWrite, onClose } = opened("tags", "web", ["web"]);
    fireEvent.keyDown(screen.getByRole("dialog"), { key: "Escape" });
    expect(onWrite).not.toHaveBeenCalled();
    expect(onClose).toHaveBeenCalled();
  });

  it("links a document found by its name, as the slash menu writes it", async () => {
    const { onWrite } = opened("link");
    fireEvent.click(await screen.findByText("Contrato"));
    expect(onWrite).toHaveBeenCalledWith("![Contrato](tisty:doc/mac0-0001)");
  });
});
