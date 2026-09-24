import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useEffect } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Filed } from "../core";
import Docs from "../ui/Docs";

const store = vi.hoisted(() => ({ heads: true }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) => {
    switch (cmd) {
      case "doc_read":
        return Promise.resolve("# Compras\n\nleche");
      case "doc_facts":
        return Promise.resolve({
          made: 1772668800,
          wrote: 1772755200,
          bytes: 8400,
          author: null,
          editor: null,
          born: null,
        });
      default:
        return Promise.resolve(null);
    }
  },
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  save: () => Promise.resolve(null),
  open: () => Promise.resolve(null),
  ask: () => Promise.resolve(true),
}));

vi.mock("../ui/Editor", () => ({
  default: ({
    value,
    onWrite,
    onOutline,
  }: {
    value: string;
    onWrite: (text: string) => void;
    onOutline?: (heads: unknown[]) => void;
  }) => {
    useEffect(() => {
      onOutline?.(store.heads ? [{ key: "1", level: 1, text: "Compras", go: () => {} }] : []);
    }, [onOutline]);
    return <textarea aria-label="editor" value={value} onChange={(e) => onWrite(e.target.value)} />;
  },
}));

const book: Filed = {
  id: "01F",
  file: "a3f1-0001",
  title: "Compras",
  folder: null,
  archived: false,
  away: false,
};

const page = (one: Partial<Filed> & { file: string; id: string }): Filed => ({
  title: "Fruta",
  folder: null,
  pageOf: "01F",
  archived: false,
  away: false,
  ...one,
});

const open = (known: Filed[], at = "a3f1-0001", onDoc = vi.fn()) => {
  render(<Docs open={at} known={known} onDoc={onDoc} onKept={vi.fn()} onError={vi.fn()} />);
  return onDoc;
};

const index = async () =>
  within(await screen.findByRole("complementary", { name: "About this document" }));

describe("the pages a document holds, seen from the column", () => {
  beforeEach(() => {
    store.heads = true;
    Object.defineProperty(window, "innerWidth", {
      value: 1500,
      configurable: true,
      writable: true,
    });
  });

  it("counts the pages it holds and names the one being read", async () => {
    open([
      book,
      page({ id: "02F", file: "a3f1-0002" }),
      page({ id: "03F", file: "a3f1-0003", title: "Verdura" }),
    ]);

    const aside = await index();
    expect(await aside.findByText("2 pages")).toBeTruthy();
    expect(aside.getByRole("button", { name: "Fruta" }).getAttribute("aria-current")).toBeNull();
  });

  it("says out loud which page is in the archive, and which an agent gave up for old", async () => {
    open([
      book,
      page({ id: "02F", file: "a3f1-0002", archived: true, away: true }),
      page({
        id: "03F",
        file: "a3f1-0003",
        title: "Verdura",
        flagged: { at: "2026-09-20T10:00:00Z", said: "it has had its day" },
      }),
    ]);

    const aside = await index();
    expect(await aside.findByRole("button", { name: "Fruta — in the archive" })).toBeTruthy();
    expect(
      aside.getByRole("button", { name: "Verdura — An agent says this has had its day" }),
    ).toBeTruthy();
  });

  it("keeps the mark out of sight while the archive holds the page", async () => {
    open([
      book,
      page({
        id: "02F",
        file: "a3f1-0002",
        title: "",
        archived: true,
        away: true,
        flagged: { at: "2026-09-20T10:00:00Z", said: "it has had its day" },
      }),
    ]);

    const aside = await index();
    expect(await aside.findByRole("button", { name: "Untitled — in the archive" })).toBeTruthy();
    expect(aside.queryByText("\u25c6"), "the archive already answered the mark").toBeNull();
  });

  it("holds a page nobody named under a name anyone can read", async () => {
    open([book, page({ id: "02F", file: "a3f1-0002", title: "" })]);

    const aside = await index();
    expect(await aside.findByText("1 page")).toBeTruthy();
    expect(aside.getByRole("button", { name: "Untitled" })).toBeTruthy();
  });

  it("marks the page being read among the ones its document holds", async () => {
    open(
      [
        book,
        page({ id: "02F", file: "a3f1-0002" }),
        page({ id: "03F", file: "a3f1-0003", title: "Verdura" }),
      ],
      "a3f1-0003",
    );

    const aside = await index();
    expect(
      (await aside.findByRole("button", { name: "Verdura" })).getAttribute("aria-current"),
    ).toBe("true");
    expect(aside.getByRole("button", { name: "Fruta" }).getAttribute("aria-current")).toBeNull();
  });

  it("opens the page it was asked for", async () => {
    const onDoc = open([book, page({ id: "02F", file: "a3f1-0002" })]);

    const aside = await index();
    await userEvent.click(await aside.findByRole("button", { name: "Fruta" }));

    expect(onDoc).toHaveBeenCalledWith("a3f1-0002");
  });

  it("keeps the outline up for a document with pages and no headings of its own", async () => {
    store.heads = false;
    open([book, page({ id: "02F", file: "a3f1-0002" })]);

    const aside = await index();
    expect(await aside.findByText("1 page")).toBeTruthy();
    expect(aside.queryByText("No headings yet")).toBeNull();
  });

  it("says there is no outline when the document holds neither headings nor pages", async () => {
    store.heads = false;
    open([book]);

    const aside = await index();
    await aside.findByText(/kB/);
    expect(aside.queryByText("1 page")).toBeNull();
  });
});
