import { describe, expect, it } from "vitest";
import { fetched } from "../src/fetched";

const LIMITS = '{"text":524288}';

const server = (docs: () => Response) =>
  (async (asked: RequestInfo | URL) =>
    String(asked) === "/api/v1/limits" ? new Response(LIMITS) : docs()) as typeof fetch;

describe("what the server answers for a link", () => {
  it("is the sealed bytes, with the limits to read them under", async () => {
    const said = await fetched(
      "k7Qm2xPa",
      server(() => new Response(new Uint8Array([1, 2, 3]))),
    );
    expect(said).toEqual({ sealed: new Uint8Array([1, 2, 3]), limits: LIMITS });
  });

  it("asks for the document the link names, and nothing of its key", async () => {
    const asked: string[] = [];
    await fetched("k7Qm2xPa", (async (url: RequestInfo | URL) => {
      asked.push(String(url));
      return new Response("");
    }) as typeof fetch);
    expect(asked.sort()).toEqual(["/api/v1/docs/k7Qm2xPa", "/api/v1/limits"]);
  });

  it("is the word of its refusal", async () => {
    const missing = () => new Response('{"refused":"missing"}', { status: 404 });
    expect(await fetched("k7Qm2xPa", server(missing))).toEqual({ refused: "missing" });
  });

  it("says down when the server fails without a word", async () => {
    const broken = () => new Response("<html>", { status: 502 });
    expect(await fetched("k7Qm2xPa", server(broken))).toEqual({ refused: "down" });
  });

  it("says offline when nothing answers", async () => {
    const nowhere = (async () => {
      throw new TypeError("Failed to fetch");
    }) as typeof fetch;
    expect(await fetched("k7Qm2xPa", nowhere)).toEqual({ refused: "offline" });
  });
});
