import { describe, expect, it } from "vitest";
import { ending, named, previewOf, weighed } from "../previews";

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
});
