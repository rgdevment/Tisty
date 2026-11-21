import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { adopt, t } from "../locales";
import HelpBand from "../ui/HelpBand";

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
  refuse: "",
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args: Record<string, unknown> = {}) => {
    ipc.calls.push({ cmd, args });
    if (cmd === ipc.refuse) return Promise.reject({ code: "internal" });
    if (cmd === "guide") return Promise.resolve({ id: "guide-0001", title: "How Tisty works" });
    return Promise.resolve(null);
  },
}));

const sent = (cmd: string) => ipc.calls.filter((one) => one.cmd === cmd);

beforeEach(() => {
  adopt("en");
  ipc.calls.length = 0;
  ipc.refuse = "";
});

describe("the help band on Help and about", () => {
  it("offers the welcome again, without touching what is written", async () => {
    const greet = vi.fn();
    render(<HelpBand onGreet={greet} onDoc={() => {}} onChanged={() => {}} />);

    await userEvent.click(screen.getByRole("button", { name: /show it again/i }));

    expect(greet).toHaveBeenCalled();
    expect(sent("choose_sync")).toHaveLength(0);
  });

  it("opens the guide instead of only saying where it went", async () => {
    const opened = vi.fn();
    const changed = vi.fn();
    render(<HelpBand onGreet={() => {}} onDoc={opened} onChanged={changed} />);

    await userEvent.click(screen.getByRole("button", { name: /open the guide/i }));

    await waitFor(() => expect(sent("guide")).toHaveLength(1));
    expect(opened).toHaveBeenCalledWith("guide-0001");
    expect(changed).toHaveBeenCalled();
  });

  it("opens the help on tisty.app in the language spoken", async () => {
    render(<HelpBand onGreet={() => {}} onDoc={() => {}} onChanged={() => {}} />);

    expect(screen.getByText(t("helpOnSiteWhy"))).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: /^open$/i }));

    await waitFor(() => expect(sent("open_link")[0]?.args.url).toBe(t("helpMoreAt")));
  });

  it("says so beside the row when the site cannot be opened", async () => {
    ipc.refuse = "open_link";
    render(<HelpBand onGreet={() => {}} onDoc={() => {}} onChanged={() => {}} />);

    await userEvent.click(screen.getByRole("button", { name: /^open$/i }));

    await waitFor(() =>
      expect(document.querySelector("[data-setting=site] [data-note=err]")).toBeTruthy(),
    );
  });
});
