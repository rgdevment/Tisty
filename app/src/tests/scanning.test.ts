import { describe, expect, it, vi } from "vitest";
import type { Filed } from "../core";
import { scanned } from "../scanning";

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => Promise.resolve(null) }));

const filed = (file: string, title: string): Filed => ({
  id: file,
  file,
  title,
  folder: null,
  archived: false,
  away: false,
});

describe("looking through every document for what the editor cannot keep", () => {
  it("names the ones the core finds, and what each brings", async () => {
    const found = await scanned(
      () => Promise.resolve(["a-0003"]),
      () =>
        Promise.resolve({
          docs: [filed("a-0001", "Limpio"), filed("a-0002", "Cabecera"), filed("a-0003", "Html")],
        }),
    );

    expect(found.map((one) => one.file)).toEqual(["a-0003"]);
    expect(found[0].brings).toEqual(["frailHtml"]);
  });

  it("says nothing at all when every document survives being saved", async () => {
    const found = await scanned(
      () => Promise.resolve([]),
      () => Promise.resolve({ docs: [filed("a-0001", "Limpio")] }),
    );

    expect(found).toEqual([]);
  });

  it("keeps the title, because a file name is not what a person looks for", async () => {
    const found = await scanned(
      () => Promise.resolve(["a-0009"]),
      () => Promise.resolve({ docs: [filed("a-0009", "Minuta del lunes")] }),
    );

    expect(found[0].title).toBe("Minuta del lunes");
  });

  it("leaves out a file the list no longer has", async () => {
    expect(
      await scanned(
        () => Promise.resolve(["a-0404"]),
        () => Promise.resolve({ docs: [] }),
      ),
    ).toEqual([]);
  });
});
