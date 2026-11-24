import { describe, expect, it, vi } from "vitest";
import { designing } from "../ui/designing";

const keys = (all: readonly { key: string; on: boolean }[]) =>
  all.filter((one) => one.on).map((one) => one.key);

describe("what the panel offers to dress a document", () => {
  it("is the five designs and one of its own, whose parts show only while it is worn", () => {
    const said = designing("coral", null, () => {});
    expect(said.all).toHaveLength(6);
    expect(keys(said.all)).toEqual(["coral"]);
    expect(said.all[5].key).toBe("own/papel/grafito/tinta/sans");
    expect(said.own).toBeUndefined();
  });

  it("offers each part as the whole design it would make", () => {
    const said = designing("own/arena/vino/cafe/lora", null, () => {});
    expect(said.own?.colors.map((one) => keys(one.all))).toEqual(
      Array(3).fill(["own/arena/vino/cafe/lora"]),
    );
    const [sheets] = said.own?.colors ?? [];
    expect(sheets.all.map((one) => one.key)).toContain("own/menta/vino/cafe/lora");
    const fonts = said.own?.font.all ?? [];
    expect(fonts).toHaveLength(9);
    expect(keys(fonts)).toEqual(["own/arena/vino/cafe/lora"]);
    expect(fonts.find((one) => one.label === "Caveat")?.key).toBe("own/arena/vino/cafe/caveat");
  });

  it("gives back the last one made, and tells what was picked", () => {
    const picked = vi.fn();
    const said = designing("bosque", "own/arena/vino/cafe/lora", picked);
    expect(said.all[5].key).toBe("own/arena/vino/cafe/lora");
    said.onPick("own/arena/vino/cafe/lora");
    expect(picked).toHaveBeenCalledWith("own/arena/vino/cafe/lora");
  });
});
