import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import Insert from "../ui/Insert";

const store = vi.hoisted(() => ({
  made: [] as string[],
  wrote: [] as { id: string; body: string }[],
  fails: false,
}));

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => Promise.resolve(null) }));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: () => Promise.resolve(null) }));
vi.mock("../core", async () => ({
  ...(await vi.importActual<typeof import("../core")>("../core")),
  docs: () => Promise.resolve({ folders: [], docs: [] }),
  attach: () => Promise.resolve(null),
  docNew: () => {
    if (store.fails) return Promise.reject({ code: "cannotWrite" });
    const id = `mac0-000${store.made.length + 1}`;
    store.made.push(id);
    return Promise.resolve({ id, title: "" });
  },
  docWrite: (id: string, body: string) => {
    store.wrote.push({ id, body });
    return Promise.resolve({ id, title: "" });
  },
}));

beforeEach(() => {
  store.made = [];
  store.wrote = [];
  store.fails = false;
});

describe("making a document from a task", () => {
  const open = async (onPut = vi.fn(), onError = vi.fn()) => {
    render(<Insert onPut={onPut} onClose={vi.fn()} onError={onError} />);
    await userEvent.click(screen.getByRole("button", { name: /A new document/ }));
    return { onPut, onError };
  };

  it("writes the name as the title, so the document is not born untitled", async () => {
    await open();

    await userEvent.type(screen.getByLabelText(/Name of the document/), "Minuta del lunes{Enter}");

    await waitFor(() => expect(store.wrote).toHaveLength(1));
    expect(store.wrote[0].body).toBe("# Minuta del lunes\n\n");
  });

  it("leaves the reference behind in the task that asked for it", async () => {
    const { onPut } = await open();

    await userEvent.type(screen.getByLabelText(/Name of the document/), "Minuta del lunes{Enter}");

    await waitFor(() => expect(onPut).toHaveBeenCalled());
    expect(onPut).toHaveBeenCalledWith("[Minuta del lunes](tisty:doc/mac0-0001)");
  });

  it("makes nothing at all when the name is only spaces", async () => {
    const { onPut } = await open();

    await userEvent.type(screen.getByLabelText(/Name of the document/), "   {Enter}");

    expect(store.made).toHaveLength(0);
    expect(onPut).not.toHaveBeenCalled();
  });

  it("says what went wrong instead of leaving a half made document", async () => {
    store.fails = true;
    const { onPut, onError } = await open();

    await userEvent.type(screen.getByLabelText(/Name of the document/), "Minuta{Enter}");

    await waitFor(() => expect(onError).toHaveBeenCalledWith({ code: "cannotWrite" }));
    expect(onPut).not.toHaveBeenCalled();
    expect(store.wrote).toHaveLength(0);
  });
});
