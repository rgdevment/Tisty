import { beforeEach, describe, expect, it, vi } from "vitest";

const net = vi.hoisted(() => ({ kept: 0, fetched: null as unknown }));

vi.mock("../core", () => ({
  glimpseKept: () => {
    net.kept += 1;
    return Promise.resolve(null);
  },
  glimpseFetch: () => Promise.resolve(net.fetched),
}));

import { GLIMPSES } from "../ui/webcarding";

beforeEach(() => {
  net.kept = 0;
  net.fetched = null;
});

describe("the glimpses the window keeps", () => {
  it("asks the core once per link, however many times the page is drawn", async () => {
    await GLIMPSES.kept("https://uno.cl");
    await GLIMPSES.kept("https://uno.cl");
    expect(net.kept).toBe(1);
  });

  it("keeps a glimpse brought on request, and nothing when the site says nothing", async () => {
    net.fetched = { title: "Dos" };
    await GLIMPSES.fetched("https://dos.cl");
    expect(await GLIMPSES.kept("https://dos.cl")).toEqual({ title: "Dos" });
    net.fetched = {};
    await GLIMPSES.fetched("https://tres.cl");
    expect(await GLIMPSES.kept("https://tres.cl")).toBeNull();
    expect(net.kept).toBe(1);
  });
});
