import { describe, expect, it } from "vitest";
import { ending, family, KINDS, named, previewOf, weighed } from "../previews";

describe("what a link is worth showing as", () => {
  it("plays what the webview can play", () => {
    expect(previewOf("attachments/charla-a3f9.mp4")).toEqual({
      as: "video",
      at: "attachments/charla-a3f9.mp4",
    });
    expect(previewOf("attachments/nota-11bc.m4a")).toEqual({
      as: "audio",
      at: "attachments/nota-11bc.m4a",
    });
  });

  it("makes a card of anything else that was copied in", () => {
    expect(previewOf("attachments/contrato-91f2.pdf")).toEqual({
      as: "file",
      at: "attachments/contrato-91f2.pdf",
      kind: "pdf",
    });
  });

  it("knows a document of this Tisty by its own scheme", () => {
    expect(previewOf("tisty:doc/mac0-0007")).toEqual({ as: "doc", id: "mac0-0007" });
  });

  it("leaves alone anything that points out of the machine", () => {
    expect(previewOf("https://ejemplo.org/charla.mp4")).toBeNull();
    expect(previewOf("mailto:a@b.test")).toBeNull();
    expect(previewOf("/Users/alguien/charla.mp4")).toBeNull();
    expect(previewOf("\\\\servidor\\charla.mp4")).toBeNull();
  });

  it("says nothing about a link with no file at the end of it", () => {
    expect(previewOf("attachments/sinextension")).toBeNull();
    expect(previewOf("")).toBeNull();
    expect(previewOf("   ")).toBeNull();
  });

  it("reads the ending past a query, and not from a dotted folder", () => {
    expect(ending("attachments/charla-a3f9.mp4?t=10")).toBe("mp4");
    expect(ending("attachments/v1.2/notas")).toBe("");
    expect(ending("attachments/CHARLA.MP4")).toBe("mp4");
  });

  it("gives back the name a person would recognise", () => {
    expect(named("attachments/informe%20final-91f2.pdf")).toBe("informe final-91f2.pdf");
    expect(named("attachments/charla.mp4")).toBe("charla.mp4");
  });

  it("says the weight the way the rest of Tisty says it", () => {
    expect(weighed(940)).toBe("940 B");
    expect(weighed(2_400_000)).toBe("2.4 MB");
    expect(weighed(1000)).toBe("1.0 kB");
  });

  it("plays what the webview can play whatever case the extension is written in", () => {
    expect(previewOf("attachments/charla-a3f9.MP4")).toEqual({
      as: "video",
      at: "attachments/charla-a3f9.MP4",
    });
    expect(ending("attachments/CHARLA.Mp4")).toBe("mp4");
  });

  it("reads only the last of a name with more than one extension", () => {
    expect(ending("attachments/copia-a3f9.tar.gz")).toBe("gz");
    expect(previewOf("attachments/copia-a3f9.tar.gz")).toEqual({
      as: "file",
      at: "attachments/copia-a3f9.tar.gz",
      kind: "gz",
    });
  });

  it("finds nothing to end a name that never had a dot", () => {
    expect(ending("attachments/README")).toBe("");
  });

  it("gives back the name without what follows a query or a fragment", () => {
    expect(named("attachments/informe-a3f9.pdf?download=1")).toBe("informe-a3f9.pdf");
    expect(named("attachments/charla-a3f9.mp4#t=5")).toBe("charla-a3f9.mp4");
  });

  it("reads the ending past a fragment too", () => {
    expect(ending("attachments/charla-a3f9.mp4#t=5")).toBe("mp4");
  });

  it("keeps the raw name when a stray percent sign breaks decoding", () => {
    expect(named("attachments/100%off-a3f9.pdf")).toBe("100%off-a3f9.pdf");
  });

  it("does not cut a name however long it is", () => {
    const long = `${"a".repeat(300)}-a3f9.txt`;
    expect(named(`attachments/${long}`)).toBe(long);
  });

  it("takes only the last segment of a path, climbing dots and all", () => {
    expect(named("attachments/../secret-a3f9.txt")).toBe("secret-a3f9.txt");
    expect(previewOf("..")).toBeNull();
  });

  it.fails("never shows a unit one step behind what the rounded number reads", () => {
    expect(weighed(999_960)).not.toBe("1000.0 kB");
  });
});

describe("what family a file belongs to", () => {
  it("groups the ones a person thinks of together", () => {
    expect(family("docx")).toBe("word");
    expect(family("doc")).toBe("word");
    expect(family("xlsx")).toBe("sheet");
    expect(family("csv")).toBe("sheet");
    expect(family("pptx")).toBe("slides");
    expect(family("7z")).toBe("archive");
  });

  it("falls back to plain rather than guessing", () => {
    expect(family("wat")).toBe("plain");
    expect(family("")).toBe("plain");
  });

  it("names every kind through the catalogue, never in one language", () => {
    const said = Object.values(KINDS);

    expect(said.every((one) => one.startsWith("kind"))).toBe(true);
    expect(said.some((one) => /[áéíóúñ¿]/i.test(one))).toBe(false);
  });
});
