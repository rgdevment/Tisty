import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import { composed, docLink, docOf } from "../markdown";
import Composed from "../ui/Composed";
import Insert from "../ui/Insert";

const shelf = vi.hoisted(() => ({
  docs: [
    { id: "01A", file: "mac0-0001", title: "Informe técnico", folder: null, archived: false },
    { id: "01B", file: "mac0-0002", title: "Recetas", folder: null, archived: false },
    { id: "01C", file: "mac0-0003", title: "Lo viejo", folder: null, archived: true },
  ],
}));

const opener = vi.hoisted(() => ({
  url: vi.fn(() => Promise.resolve()),
  reveal: vi.fn(() => Promise.resolve()),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: () => Promise.resolve(null),
  convertFileSrc: (at: string) => at,
}));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: () => Promise.resolve(null) }));
vi.mock("@tauri-apps/plugin-opener", () => ({ openUrl: opener.url }));
vi.mock("../core", async () => ({
  ...(await vi.importActual<typeof import("../core")>("../core")),
  docs: () => Promise.resolve({ folders: [], docs: shelf.docs }),
  opened: () => Promise.resolve(),
  revealed: opener.reveal,
  served: () => Promise.resolve(""),
  attach: () => Promise.resolve(null),
}));

describe("a reference to a document of this Tisty", () => {
  it("says which document without saying where it sits", () => {
    expect(docLink("mac0-0001", "Informe técnico")).toBe("[Informe técnico](tisty:doc/mac0-0001)");
    expect(docOf("tisty:doc/mac0-0001")).toBe("mac0-0001");
    expect(docOf("https://ejemplo.org")).toBeNull();
    expect(docOf("attachments/foto.png")).toBeNull();
  });

  it("survives a title with a loose bracket, which would end the link early", () => {
    const written = docLink("mac0-0001", "Informe] borrador");

    expect(composed(written)).toContain('href="tisty:doc/mac0-0001"');
    expect(composed(written)).not.toContain("tisty:doc/mac0-0001)");
  });

  it("is dressed apart from an address that leaves the app", () => {
    expect(composed(docLink("mac0-0001", "Informe"))).toContain('class="paper"');
    expect(composed("[Fuera](https://ejemplo.org)")).not.toContain("paper");
    expect(composed("[Un archivo](attachments/foto.png)")).not.toContain("paper");
  });

  it("survives the document editor, which drops schemes it was not told about", async () => {
    const { Editor } = await import("@tiptap/core");
    const { written, asMarkdown } = await import("../ui/writing");
    const written_ = docLink("mac0-0007", "Informe");
    const editor = new Editor({ extensions: written(), content: written_ });

    expect(asMarkdown(editor)).toBe(written_);
    expect(editor.getHTML()).toContain('href="tisty:doc/mac0-0007"');

    editor.destroy();
  });

  it("opens the document instead of asking the system to open a file", async () => {
    const onDoc = vi.fn();
    render(
      <Composed
        html={composed("Repasar [Informe](tisty:doc/mac0-0001)")}
        onDoc={onDoc}
        className=""
      />,
    );

    await userEvent.click(screen.getByText("Informe"));

    expect(onDoc).toHaveBeenCalledWith("mac0-0001");
    expect(opener.reveal).not.toHaveBeenCalled();
    expect(opener.url).not.toHaveBeenCalled();
  });

  it("still hands an ordinary address to the system", async () => {
    const onDoc = vi.fn();
    render(
      <Composed html={composed("[Fuera](https://ejemplo.org)")} onDoc={onDoc} className="" />,
    );

    await userEvent.click(screen.getByText("Fuera"));

    expect(onDoc).not.toHaveBeenCalled();
    expect(opener.url).toHaveBeenCalledWith("https://ejemplo.org");
  });
});

describe("picking a document to reference", () => {
  const menu = (onPut = vi.fn()) => {
    render(<Insert onPut={onPut} onClose={vi.fn()} />);
    return onPut;
  };

  it("is offered where the other things to insert are", async () => {
    menu();

    expect(screen.getByRole("button", { name: /A document/ })).toBeTruthy();
  });

  it("writes the reference by identifier, not by title", async () => {
    const onPut = menu();
    await userEvent.click(screen.getByRole("button", { name: /A document/ }));
    await waitFor(() => screen.getByRole("button", { name: /Informe técnico/ }));

    await userEvent.click(screen.getByRole("button", { name: /Informe técnico/ }));

    expect(onPut).toHaveBeenCalledWith("[Informe técnico](tisty:doc/mac0-0001)");
  });

  it("narrows to what is being typed", async () => {
    menu();
    await userEvent.click(screen.getByRole("button", { name: /A document/ }));
    await waitFor(() => screen.getByRole("button", { name: /Informe técnico/ }));

    await userEvent.type(screen.getByLabelText(/Which document/), "rece");

    expect(screen.queryByRole("button", { name: /Informe técnico/ })).toBeNull();
    expect(screen.getByRole("button", { name: /Recetas/ })).toBeTruthy();
  });

  it("leaves the archived ones out of the choosing", async () => {
    menu();
    await userEvent.click(screen.getByRole("button", { name: /A document/ }));
    await waitFor(() => screen.getByRole("button", { name: /Informe técnico/ }));

    expect(screen.queryByRole("button", { name: /Lo viejo/ })).toBeNull();
  });
});
