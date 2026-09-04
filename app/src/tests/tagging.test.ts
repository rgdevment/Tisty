import { getSchema } from "@tiptap/core";
import { Document } from "@tiptap/extension-document";
import { Paragraph } from "@tiptap/extension-paragraph";
import { Text } from "@tiptap/extension-text";
import { describe, expect, it } from "vitest";
import { named, spots } from "../ui/tagging";

const schema = getSchema([Document, Paragraph, Text]);

const lit = (said: string): string[] => {
  const doc = schema.node("doc", null, [
    schema.node("paragraph", null, said ? [schema.text(said)] : []),
  ]);
  return spots(doc).map((one) => said.slice(one.from - 1, one.to - 1));
};

describe("what the editor paints as a tag", () => {
  it("lights a hash pinned to a word", () => {
    expect(lit("esto es #legal antes que nada")).toEqual(["#legal"]);
  });

  it("leaves a heading alone, because it carries a space", () => {
    expect(lit("# Alquiler del local")).toEqual([]);
  });

  it("leaves the fragment of an address to the address", () => {
    expect(lit("mira https://ejemplo.com/pagina#seccion")).toEqual([]);
  });

  it("lights every one on the line", () => {
    expect(lit("#uno #dos #tres")).toEqual(["#uno", "#dos", "#tres"]);
  });

  it("holds nothing where the hash stands alone", () => {
    expect(lit("un # suelto")).toEqual([]);
  });

  it("reads the same as the core does, hyphens and all", () => {
    expect(lit("#pago-mensual y #Contrato")).toEqual(["#pago-mensual", "#Contrato"]);
  });

  it("hands the pressed word over in the shape the core keeps it", () => {
    expect(named("Contrato")).toBe("contrato");
    expect(named("pago_mensual")).toBe("pago-mensual");
    expect(named("legal")).toBe("legal");
  });
});
