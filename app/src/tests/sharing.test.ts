import { beforeEach, describe, expect, it, vi } from "vitest";
import { finishWhatIsAsked } from "../finishing";
import { adopt } from "../locales";
import { plainly } from "../refusal";
import { docSealed, type Told, toldPlainly } from "../sharing";

const invoked = vi.hoisted(() => vi.fn());
const finished = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/core", () => ({ invoke: invoked }));
vi.mock("tisty-editor/read", () => ({ finished }));

beforeEach(() => {
  invoked.mockReset();
  finished.mockReset();
  adopt("en");
});

describe("a document sealed where it is kept", () => {
  it("asks for what the person chose and nothing else", async () => {
    invoked.mockResolvedValue({ envelopes: 2, bytes: 10, told: [] });
    const sealed = await docSealed("d1", { public: true, cited: ["d2"], signedAs: "Rina" });
    expect(invoked).toHaveBeenCalledWith("doc_sealed", {
      id: "d1",
      public: true,
      cited: ["d2"],
      signedAs: "Rina",
    });
    expect(sealed.envelopes).toBe(2);
  });
});

describe("what did not travel as it was written", () => {
  const each: [Told, string[]][] = [
    [{ what: "pictureAsFile", document: "Paseos", name: "a.svg" }, ["a.svg", "Paseos", "file"]],
    [{ what: "pictureAsWords", document: "Paseos", name: "a.png" }, ["a.png", "Paseos", "words"]],
    [
      { what: "pictureAsLink", document: "Paseos", name: "https://a.example" },
      ["https://a.example", "Paseos", "link"],
    ],
    [
      { what: "fileAsWords", document: "Paseos", name: "a.pdf" },
      ["a.pdf", "Paseos", "file", "words"],
    ],
    [{ what: "notDrawn", document: "Paseos", many: 2 }, ["2", "Paseos", "code"]],
    [{ what: "pageLeftBehind", document: "Paseos" }, ["Paseos", "page"]],
    [{ what: "languageLeftBehind", language: "en" }, ["en", "language"]],
    [{ what: "languageOverTheTop", language: "en" }, ["en", "20"]],
    [{ what: "takenBackNotKept" }, ["retired", "identity"]],
    [{ what: "citedLeftBehind", document: "Otro" }, ["Otro", "words"]],
    [{ what: "pictureNotKept", document: "Paseos", name: "a.png" }, ["a.png", "Paseos", "kept"]],
    [{ what: "pictureNotKept", document: "Paseos", name: "" }, ["a picture", "Paseos", "kept"]],
    [{ what: "notDrawn", document: "Paseos", many: 1 }, ["One diagram", "Paseos", "code"]],
    [{ what: "nameLeftOut" }, ["name"]],
  ];

  it.each(each)("is told plainly: %j", (told, says) => {
    const sentence = toldPlainly(told);
    for (const one of says) expect(sentence).toContain(one);
    expect(sentence).not.toMatch(/\{(name|other)\}/);
  });

  it.each(each)("is told in Spanish too: %j", (told) => {
    adopt("es");
    const sentence = toldPlainly(told);
    expect(sentence).not.toMatch(/\{(name|other)\}/);
    adopt("en");
    expect(sentence).not.toBe(toldPlainly(told));
  });
});

describe("why a document could not be sealed", () => {
  it.each([
    "notOneOwnDoc",
    "onlyADocumentShared",
    "noIdentityToSign",
    "cannotSeal",
    "cannotFinish",
    "awayNotShared",
  ])("is said in words the window knows: %s", (code) => {
    const said = plainly({ code });
    expect(said).not.toContain(code);
    adopt("es");
    expect(plainly({ code })).not.toBe(said);
  });
});

describe("a window nobody sees", () => {
  it("finishes what it is asked and says so", async () => {
    invoked.mockImplementation((command: string) =>
      Promise.resolve(
        command === "finishing_asked"
          ? [
              { name: "a", html: "<p>a</p>" },
              { name: "a/1", html: "<p>b</p>" },
            ]
          : undefined,
      ),
    );
    finished.mockImplementation((html: string) =>
      Promise.resolve({ html: `${html}!`, notDrawn: 1 }),
    );
    await finishWhatIsAsked();
    expect(invoked).toHaveBeenLastCalledWith("finishing_answered", {
      answers: [
        { name: "a", html: "<p>a</p>!", notDrawn: 1 },
        { name: "a/1", html: "<p>b</p>!", notDrawn: 1 },
      ],
    });
  });

  it("says when it could not, so nobody is left waiting", async () => {
    invoked.mockImplementation((command: string) =>
      Promise.resolve(command === "finishing_asked" ? [{ name: "a", html: "x" }] : undefined),
    );
    finished.mockRejectedValue(new Error("no"));
    await finishWhatIsAsked();
    expect(invoked).toHaveBeenLastCalledWith("finishing_answered", { answers: null });
  });
});
