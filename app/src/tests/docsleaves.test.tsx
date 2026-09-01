import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Filed } from "../core";
import Docs from "../ui/Docs";

const store = vi.hoisted(() => ({
  bodies: {} as Record<string, string>,
  put: [] as { file: string; title: string }[],
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) =>
    cmd === "doc_read"
      ? Promise.resolve(store.bodies[String(args?.id)] ?? "")
      : Promise.resolve(null),
}));

vi.mock("../ui/Editor", () => ({
  default: ({
    value,
    above,
    below,
    onInsert,
  }: {
    value: string;
    above?: React.ReactNode;
    below?: React.ReactNode;
    onInsert?: (put: (file: string, title: string) => void) => void;
  }) => {
    onInsert?.((file, title) => store.put.push({ file, title }));
    return (
      <div>
        {above}
        <textarea aria-label="editor" readOnly value={value} />
        {below}
      </div>
    );
  },
}));

const known: Filed[] = [
  { id: "01A", file: "a3f1-0001", title: "Bases de datos", folder: null, archived: false },
  {
    id: "01B",
    file: "a3f1-0002",
    title: "El pod",
    folder: null,
    archived: false,
    pageOf: "01A",
  },
  {
    id: "01C",
    file: "a3f1-0003",
    title: "El túnel",
    folder: null,
    archived: false,
    pageOf: "01A",
  },
  { id: "01D", file: "a3f1-0004", title: "Solo", folder: null, archived: false },
];

describe("a document that holds pages, open", () => {
  beforeEach(() => {
    store.bodies = {
      "a3f1-0001": "# Bases de datos\n\n![El pod](tisty:doc/a3f1-0002)",
      "a3f1-0002": "# El pod",
      "a3f1-0003": "# El túnel",
      "a3f1-0004": "# Solo",
    };
    store.put = [];
  });

  const show = (open: string, onDoc = vi.fn()) => {
    render(<Docs open={open} known={known} onKept={vi.fn()} onError={vi.fn()} onDoc={onDoc} />);
    return { onDoc };
  };

  it("lists every page after the text, the named one numbered and the rest loose", async () => {
    show("a3f1-0001");

    await waitFor(() => expect(screen.getByText("El pod")).toBeTruthy());
    const rows = screen.getAllByRole("listitem");
    expect(rows.map((one) => one.textContent)).toEqual(["01El pod", "—El túnelPut it in the text"]);
  });

  it("opens a page from the index", async () => {
    const { onDoc } = show("a3f1-0001");
    await waitFor(() => expect(screen.getByText("El pod")).toBeTruthy());

    await userEvent.click(screen.getByText("El pod"));

    expect(onDoc).toHaveBeenCalledWith("a3f1-0002");
  });

  it("hands a loose page to the text when asked to put it there", async () => {
    show("a3f1-0001");
    await waitFor(() => expect(screen.getByText("El túnel")).toBeTruthy());

    await userEvent.click(screen.getByRole("button", { name: "Put it in the text" }));

    expect(store.put).toEqual([{ file: "a3f1-0003", title: "El túnel" }]);
  });

  it("says nothing after a document that holds no pages", async () => {
    show("a3f1-0004");
    await waitFor(() => expect(screen.getByLabelText("editor")).toBeTruthy());

    expect(screen.queryByRole("listitem")).toBeNull();
  });
});

describe("a page, open", () => {
  beforeEach(() => {
    store.bodies = {
      "a3f1-0001":
        "# Bases de datos\n\n![El pod](tisty:doc/a3f1-0002)\n\n![El túnel](tisty:doc/a3f1-0003)",
      "a3f1-0002": "# El pod",
      "a3f1-0003": "# El túnel",
    };
    store.put = [];
  });

  it("says which document it belongs to and where it sits", async () => {
    render(<Docs open="a3f1-0002" known={known} onKept={vi.fn()} onError={vi.fn()} />);

    await waitFor(() => expect(screen.getByText("Bases de datos")).toBeTruthy());
    expect(screen.getByText("Page 1 of 2")).toBeTruthy();
  });

  it("offers the step to the page that follows, and none on the last", async () => {
    const { unmount } = render(
      <Docs open="a3f1-0002" known={known} onKept={vi.fn()} onError={vi.fn()} />,
    );
    await waitFor(() => expect(screen.getByText("El túnel")).toBeTruthy());
    unmount();

    render(<Docs open="a3f1-0003" known={known} onKept={vi.fn()} onError={vi.fn()} />);
    await waitFor(() => expect(screen.getByText("Page 2 of 2")).toBeTruthy());
    expect(screen.queryByText("Next")).toBeNull();
  });

  it("shows no index of its own, because a page holds no pages", async () => {
    render(<Docs open="a3f1-0002" known={known} onKept={vi.fn()} onError={vi.fn()} />);

    await waitFor(() => expect(screen.getByLabelText("editor")).toBeTruthy());
    expect(screen.queryByRole("listitem")).toBeNull();
  });
});
