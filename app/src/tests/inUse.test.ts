import { afterEach, describe, expect, it, vi } from "vitest";

const { served } = vi.hoisted(() => ({
  served: vi.fn((reference: string) => Promise.resolve(`C:\\data\\${reference}`)),
}));

vi.mock("../core", () => ({ served }));

import { serve } from "../inUse";

afterEach(() => {
  served.mockReset();
  vi.useRealTimers();
});

describe("asking for an attachment", () => {
  it("asks once while an ask is on its way, whoever else wants it", async () => {
    served.mockImplementation((reference: string) => Promise.resolve(`C:\\data\\${reference}`));

    const both = await Promise.all([
      serve("attachments/aa/uno.png"),
      serve("attachments/aa/uno.png"),
    ]);

    expect(both).toEqual(["C:\\data\\attachments/aa/uno.png", "C:\\data\\attachments/aa/uno.png"]);
    expect(served).toHaveBeenCalledTimes(1);
  });

  it("does not ask again for a minute after a refusal, then asks", async () => {
    vi.useFakeTimers();
    served.mockImplementation(() => Promise.reject({ code: "cloudLater" }));

    await expect(serve("attachments/bb/dos.png")).rejects.toEqual({ code: "cloudLater" });
    await expect(serve("attachments/bb/dos.png")).rejects.toEqual({ code: "cloudLater" });
    expect(served).toHaveBeenCalledTimes(1);

    vi.advanceTimersByTime(61 * 1000);
    served.mockImplementation((reference: string) => Promise.resolve(`C:\\data\\${reference}`));
    await expect(serve("attachments/bb/dos.png")).resolves.toBe("C:\\data\\attachments/bb/dos.png");
    expect(served).toHaveBeenCalledTimes(2);
  });
});
