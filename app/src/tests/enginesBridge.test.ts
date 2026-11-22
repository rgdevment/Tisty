import { describe, expect, it } from "vitest";

const SERVED = "../../scripts/engines/served.mjs";

const arriving = async function* (parts: readonly Uint8Array[]) {
  yield* parts;
};

describe("the bridge the editor harness serves", () => {
  it("reads a long body split inside a letter as the page sent it", async () => {
    const { questionOf } = await import(SERVED);
    const sent = JSON.stringify({
      cmd: "doc_placed",
      args: { body: "## Capítulo 79\n\nNadie recordaba quién había pintado la puerta." },
    });
    const bytes = Buffer.from(sent);
    const cut = bytes.indexOf("í") + 1;

    const question = await questionOf(arriving([bytes.subarray(0, cut), bytes.subarray(cut)]));

    expect(question).toBe(sent);
  });
});
