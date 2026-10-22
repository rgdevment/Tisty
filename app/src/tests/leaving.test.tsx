import { renderHook, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

const sent = vi.hoisted(() => ({
  kept: [] as { at: string; bytes: number }[],
  word: [] as { bodies: string[]; at: string }[],
  where: "D:/salida/Plan" as string | null,
  pages: [] as string[][],
  facts: null as null | Record<string, string | number>,
  pdf: [] as { html: string; paper: string }[],
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  save: ({ defaultPath }: { defaultPath: string }) =>
    Promise.resolve(
      sent.where && `${sent.where}${defaultPath.slice(defaultPath.lastIndexOf("."))}`,
    ),
}));

vi.mock("../core", async (original) => ({
  ...(await original<typeof import("../core")>()),
  keepFile: (at: string, bytes: number[]) => {
    sent.kept.push({ at, bytes: bytes.length });
    return Promise.resolve();
  },
  docWord: (bodies: string[], at: string) => {
    sent.word.push({ bodies, at });
    return Promise.resolve();
  },
  docFacts: () => (sent.facts ? Promise.resolve(sent.facts) : Promise.reject(new Error("gone"))),
  pagePdf: (html: string, paper: string) => {
    sent.pdf.push({ html, paper });
    return Promise.resolve([37, 80, 68, 70]);
  },
}));

vi.mock("../ui/webPage", () => ({
  webPage: (bodies: string[]) => {
    sent.pages.push(bodies);
    return Promise.resolve(`<html>${bodies.join("|")}</html>`);
  },
}));

import { factsCard, pdfOf } from "../ui/printing";
import { useLeaving } from "../ui/useLeaving";

const leave = (book: () => Promise<string[] | null>, onError = vi.fn()) =>
  renderHook(() => useLeaving({ book, called: "Plan", onError }));

describe("a document leaving as a web page or a Word file", () => {
  it("writes the web page where the person chose", async () => {
    leave(() => Promise.resolve(["# Plan", "# Anexo"]));
    window.dispatchEvent(new CustomEvent("tisty:to-html"));

    await waitFor(() => expect(sent.kept).toHaveLength(1));
    expect(sent.kept[0].at).toBe("D:/salida/Plan.html");
    expect(sent.pages[sent.pages.length - 1]).toEqual(["# Plan", "# Anexo"]);
  });

  it("hands the book to the core for Word", async () => {
    leave(() => Promise.resolve(["# Plan"]));
    window.dispatchEvent(new CustomEvent("tisty:to-word"));

    await waitFor(() => expect(sent.word).toHaveLength(1));
    expect(sent.word[0]).toEqual({ bodies: ["# Plan"], at: "D:/salida/Plan.docx" });
  });

  it("writes nothing when the person closes the dialog or nothing is open", async () => {
    sent.where = null;
    const before = sent.kept.length + sent.word.length;
    leave(() => Promise.resolve(["# Plan"]));
    window.dispatchEvent(new CustomEvent("tisty:to-html"));
    window.dispatchEvent(new CustomEvent("tisty:to-word"));
    leave(() => Promise.resolve(null));
    window.dispatchEvent(new CustomEvent("tisty:to-word"));
    await new Promise((done) => setTimeout(done, 20));

    expect(sent.kept.length + sent.word.length).toBe(before);
    sent.where = "D:/salida/Plan";
  });

  it("says what went wrong instead of failing in silence", async () => {
    const onError = vi.fn();
    leave(() => Promise.reject({ code: "cannotRead", name: "Plan" }), onError);
    window.dispatchEvent(new CustomEvent("tisty:to-html"));

    await waitFor(() => expect(onError).toHaveBeenCalled());
  });
});

describe("the PDF as the page on paper", () => {
  it("lays the page out on the chosen sheet", async () => {
    const made = await pdfOf(["# Plan"], "letter", { credited: true });

    expect(made.type).toBe("application/pdf");
    expect(sent.pdf[sent.pdf.length - 1]?.paper).toBe("letter");
  });

  it("cards who wrote it and when, and nothing when nothing is known", async () => {
    const said = async (body: string) => (await factsCard("a-0001", body))?.map((one) => one.said);
    expect(await factsCard("a-0001", "# Plan")).toBeUndefined();
    sent.facts = { author: "ana", signedAs: "Ana Pérez", made: 1_700_000_000 };
    expect(await said("# Plan")).toContain("Ana Pérez");
    expect(await said("---\nauthor: Equipo Norte\nversion: 1.2\n---\n# Plan")).toEqual(
      expect.arrayContaining(["Equipo Norte", "1.2"]),
    );
    sent.facts = {};
    expect(await factsCard("a-0001", "# Plan")).toBeUndefined();
  });
});
