import { cleanup, render } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { Filed } from "../core";
import { draftOf, keepDraft, keepDraftsOf, letDraftGo } from "../drafts";
import Docs from "../ui/Docs";

vi.mock("../desk", async (real) => ({
  ...(await real<typeof import("../desk")>()),
  useDesk: () => ({ papersChanged: () => {} }),
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  ask: () => Promise.resolve(true),
  save: () => Promise.resolve(null),
  open: () => Promise.resolve(null),
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: () => Promise.resolve(null),
}));

const KEY = "tisty.clashing";
const LISTED = "a3f1-0001";
const GONE = "a3f1-0002";
const draft = { words: "# Compras\n\nleche y pan", base: "# Compras\n\nleche" };

const filed = (file: string): Filed => ({
  id: `01${file}`,
  file,
  title: "Compras",
  folder: null,
  archived: false,
  away: false,
});

const stillKept = () => Object.keys(JSON.parse(localStorage.getItem(KEY) ?? "{}"));

beforeEach(() => {
  letDraftGo(LISTED);
  letDraftGo(GONE);
  keepDraft(LISTED, draft);
  keepDraft(GONE, draft);
});

afterEach(() => {
  cleanup();
  letDraftGo(LISTED);
  letDraftGo(GONE);
});

describe("keepDraftsOf", () => {
  it("lets go of the drafts of documents no longer listed and keeps the listed ones", () => {
    keepDraftsOf(new Set([LISTED]));

    expect(draftOf(LISTED)).toEqual(draft);
    expect(draftOf(GONE)).toBeUndefined();
    expect(stillKept()).toEqual([LISTED]);
  });

  it("touches nothing when every draft is of a listed document", () => {
    keepDraftsOf(new Set([LISTED, GONE]));

    expect(stillKept().sort()).toEqual([LISTED, GONE]);
  });
});

describe("the list of documents arriving", () => {
  it("removes the drafts of documents it does not list", () => {
    render(<Docs known={[filed(LISTED)]} onKept={vi.fn()} onError={vi.fn()} />);

    expect(draftOf(GONE)).toBeUndefined();
    expect(draftOf(LISTED)).toEqual(draft);
  });

  it("removes nothing while the list is still empty", () => {
    render(<Docs known={[]} onKept={vi.fn()} onError={vi.fn()} />);

    expect(draftOf(GONE)).toEqual(draft);
    expect(draftOf(LISTED)).toEqual(draft);
  });
});
