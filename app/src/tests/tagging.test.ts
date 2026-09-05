import { getSchema } from "@tiptap/core";
import { Document } from "@tiptap/extension-document";
import { Paragraph } from "@tiptap/extension-paragraph";
import { Text } from "@tiptap/extension-text";
import { describe, expect, it } from "vitest";
import { drawn, named, pressed, spots } from "../ui/tagging";

const schema = getSchema([Document, Paragraph, Text]);

const written = (said: string) =>
  schema.node("doc", null, [schema.node("paragraph", null, said ? [schema.text(said)] : [])]);

const lit = (said: string): string[] =>
  spots(written(said)).map((one) => said.slice(one.from - 1, one.to - 1));

const reading = (said: string) => ({ doc: written(said), schema: { nodes: schema.nodes } });

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
    expect(lit("#_borrador y #-legal")).toEqual([]);
  });

  it("reads a run of code the way the core reads it, and a lone backtick as prose", () => {
    expect(lit("el operador ` marca código y esto es #legal")).toEqual(["#legal"]);
    expect(lit("`#rojo` no, pero #legal sí")).toEqual(["#legal"]);
  });

  it("reads the same as the core does, hyphens and all", () => {
    expect(lit("#pago-mensual y #Contrato")).toEqual(["#pago-mensual", "#Contrato"]);
  });

  it("hands over the tag under the press, and nothing under plain words", () => {
    const said = "esto es #legal hoy";
    const at = said.indexOf("#legal") + 3;

    expect(pressed(reading(said), at)).toBe("legal");
    expect(pressed(reading(said), 2)).toBeNull();
  });

  it("draws what it would light, so a press and a paint never disagree", () => {
    const shown = drawn(reading("#uno y #dos"));

    expect(shown).toHaveLength(2);
  });

  it("hands the pressed word over in the shape the core keeps it", () => {
    expect(named("Contrato")).toBe("contrato");
    expect(named("pago_mensual")).toBe("pago-mensual");
    expect(named("legal")).toBe("legal");
  });

  it("shapes the odd ones the way the core does, or the screen opens empty", () => {
    expect(named("legal_")).toBe("legal");
    expect(named("legal__mensual")).toBe("legal-mensual");
    expect(named("-legal-")).toBe("legal");
    expect(named("diseño")).toBe("diseno");
    expect(named("camión")).toBe(named("camion"));
    expect(named("CAMIÓN")).toBe("camion");
  });
});
