import { describe, expect, it } from "vitest";
import { named, often } from "../ui/tagging";
import cases from "./tagcases.json";

describe("the shape the core gives a tag", () => {
  it("shapes the odd ones the way the core does, or the screen opens empty", () => {
    expect(named("legal_")).toBe("legal");
    expect(named("legal__mensual")).toBe("legal-mensual");
    expect(named("-legal-")).toBe("legal");
    expect(named("diseño")).toBe("diseno");
    expect(named("camión")).toBe(named("camion"));
    expect(named("CAMIÓN")).toBe("camion");
  });
});

describe("the words the core reads too, from one file so the two cannot drift apart", () => {
  it("makes the tag each word says", () => {
    for (const [said, tag] of cases.named) expect(named(said), said).toBe(tag);
  });

  it("finds the tags each sentence says", () => {
    for (const [line, tags] of cases.sentences) {
      expect([...often(line as string).keys()], line as string).toEqual(tags);
    }
  });
});

describe("how often the writing leans on each tag", () => {
  const counted = (body: string) => Object.fromEntries(often(body));

  it("counts nothing where the writing was numbering, not labelling", () => {
    expect(counted("el punto #1, el #2 y el ticket #1234")).toEqual({});
  });

  it("counts every mention, not the tags the document ends up with", () => {
    expect(counted("#dinero arriba, #legal y otra vez #dinero")).toEqual({ dinero: 2, legal: 1 });
  });

  it("counts a written form and a normalised one as the same tag", () => {
    expect(counted("#Camión, #camion y #CAMION")).toEqual({ camion: 3 });
  });

  it("passes over what the core passes over, so no number outruns the chip", () => {
    const body = [
      "#legal una vez",
      "`#legal` entre comillas no",
      "```",
      "#legal dentro del bloque tampoco",
      "```",
      "y https://ejemplo.com/pagina#legal es una direccion",
    ].join(String.fromCharCode(10));

    expect(counted(body)).toEqual({ legal: 1 });
  });
});
