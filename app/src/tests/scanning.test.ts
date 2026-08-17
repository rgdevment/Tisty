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
});

describe("looking through every document for what the editor cannot keep", () => {
  it("names the ones that would open read only, and what each brings", async () => {
    const bodies: Record<string, string> = {
      "a-0001": "# Limpio\n\nun parrafo",
      "a-0002": "---\ntitle: algo\n---\n\n# Con cabecera",
      "a-0003": "# Con html\n\n<details>\n<summary>ver</summary>\nx\n</details>",
    };

    const found = await scanned(
      (file) => Promise.resolve(bodies[file]),
      () =>
        Promise.resolve({
          docs: [filed("a-0001", "Limpio"), filed("a-0002", "Cabecera"), filed("a-0003", "Html")],
        }),
    );

    expect(found.map((one) => one.file)).toEqual(["a-0002", "a-0003"]);
    expect(found[0].brings).toEqual(["frailFront"]);
    expect(found[1].brings).toEqual(["frailHtml"]);
  });

  it("says nothing at all when every document survives being saved", async () => {
    const found = await scanned(
      () => Promise.resolve("# Limpio\n\n- una lista\n- y otra"),
      () => Promise.resolve({ docs: [filed("a-0001", "Limpio")] }),
    );

    expect(found).toEqual([]);
  });

  it("keeps the title, because a file name is not what a person looks for", async () => {
    const found = await scanned(
      () => Promise.resolve("una nota[^1]\n\n[^1]: el pie"),
      () => Promise.resolve({ docs: [filed("a-0009", "Minuta del lunes")] }),
    );

    expect(found[0].title).toBe("Minuta del lunes");
  });

  it("holds one body at a time, so a big store does not have to fit in memory", async () => {
    let open = 0;
    let most = 0;

    await scanned(
      () => {
        open += 1;
        most = Math.max(most, open);
        return Promise.resolve("# Limpio").finally(() => {
          open -= 1;
        });
      },
      () =>
        Promise.resolve({
          docs: Array.from({ length: 20 }, (_, i) => filed(`a-${i}`, `Uno ${i}`)),
        }),
    );

    expect(most).toBe(1);
  });

  it("does not choke on a store with no documents at all", async () => {
    expect(
      await scanned(
        () => Promise.resolve(""),
        () => Promise.resolve({ docs: [] }),
      ),
    ).toEqual([]);
  });
});
