import { act, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Admitted, Newcomer } from "../core";
import { LetMeIn, LetThemIn } from "../ui/Admitting";

const CODE = "48210 93755 12880 63349";

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
  shown: { step: "waiting", code: null, by: null } as unknown,
  newcomers: [] as unknown,
  refuse: null as null | { cmd: string; code: string },
}));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    ipc.calls.push({ cmd, args: args ?? {} });
    if (ipc.refuse?.cmd === cmd) return Promise.reject({ code: ipc.refuse.code });
    if (cmd === "admitting_shown") return Promise.resolve(ipc.shown);
    if (cmd === "admitting_look") return Promise.resolve(ipc.newcomers);
    return Promise.resolve(null);
  },
}));

const sent = (cmd: string) => ipc.calls.filter((one) => one.cmd === cmd);

const mac: Newcomer = { device: "dev_b", name: "MacBook", at: 0, taken: false };

const letItIn = (): HTMLElement => {
  const all = screen.getAllByRole("button", { name: "Let it in" });
  return all[all.length - 1];
};

const later = (ms: number) => act(() => new Promise((done) => setTimeout(done, ms)));

beforeEach(() => {
  ipc.calls.length = 0;
  ipc.shown = { step: "waiting", code: null, by: null };
  ipc.newcomers = [];
  ipc.refuse = null;
});

describe("a new computer asking to come in", () => {
  it("waits, shows the code to type on the other computer, and trusts it once taken there", async () => {
    const onIn = vi.fn();
    render(<LetMeIn onClose={() => {}} onIn={onIn} />);
    expect(await screen.findByText(/This computer waits here meanwhile/)).toBeTruthy();
    expect(sent("admitting_ask")).toHaveLength(1);

    ipc.shown = { step: "compare", code: CODE, by: "Mac de la oficina" } satisfies Admitted;
    await later(3100);
    expect(screen.getByText(CODE)).toBeTruthy();
    expect(screen.getByText(/Type these twenty digits on «Mac de la oficina»/)).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: "It took them" }));
    expect(sent("admitting_trust")[0]?.args.code).toBe(CODE);

    ipc.shown = { step: "done", code: null, by: null } satisfies Admitted;
    await later(3100);
    expect(screen.getByText(/now reads your encrypted cloud/)).toBeTruthy();
    await later(3100);
    expect(onIn).toHaveBeenCalledTimes(1);
  }, 20_000);

  it("hands nothing over when the other computer did not take the digits, and gives the ask up when closed", async () => {
    ipc.shown = { step: "compare", code: CODE, by: "Mac" } satisfies Admitted;
    const onClose = vi.fn();
    render(<LetMeIn onClose={onClose} onIn={() => {}} />);
    await userEvent.click(await screen.findByRole("button", { name: "It did not take them" }));

    expect(screen.getByText(/nothing was handed over/)).toBeTruthy();
    expect(sent("admitting_trust")).toHaveLength(0);
    await userEvent.click(screen.getByRole("button", { name: "Cancel" }));
    await waitFor(() => expect(sent("admitting_cancel")).toHaveLength(1));
    expect(onClose).toHaveBeenCalled();
  });
});

describe("letting a new computer in from one that reads the cloud", () => {
  it("takes the ask, asks for the twenty digits the new computer shows and hands the key over", async () => {
    ipc.newcomers = [mac, { ...mac, device: "dev_c", name: "PC", taken: true }];
    render(<LetThemIn onClose={() => {}} />);

    expect(await screen.findByText("Another computer is letting it in")).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: "Let it in" }));
    const field = await screen.findByRole("textbox", { name: "The code the new computer shows" });
    expect(sent("admitting_offer")[0]?.args.device).toBe("dev_b");
    await userEvent.type(field, "48210 93755");
    expect(letItIn().hasAttribute("disabled")).toBe(true);
    await userEvent.type(field, " 12880 63349");
    await userEvent.click(letItIn());

    expect(await screen.findByText(/Press «It took them» on «MacBook»/)).toBeTruthy();
    expect(sent("admitting_grant")[0]?.args).toEqual({ device: "dev_b", code: CODE });
  });

  it("keeps only the twenty digits, grouped as the new computer shows them", async () => {
    ipc.newcomers = [mac];
    render(<LetThemIn onClose={() => {}} />);
    await userEvent.click(await screen.findByRole("button", { name: "Let it in" }));
    const field = await screen.findByRole("textbox", { name: "The code the new computer shows" });

    await userEvent.type(field, "48a2-10937551288063349999");

    expect((field as HTMLInputElement).value).toBe(CODE);
    expect(screen.getByText("20 of 20 digits")).toBeTruthy();
  });

  it("says the digits are not the ones expected and stays on the form", async () => {
    ipc.newcomers = [mac];
    ipc.refuse = { cmd: "admitting_grant", code: "admitWrongCode" };
    render(<LetThemIn onClose={() => {}} />);
    await userEvent.click(await screen.findByRole("button", { name: "Let it in" }));
    await userEvent.type(await screen.findByRole("textbox"), "00000000000000000000");
    await userEvent.click(letItIn());

    expect((await screen.findByRole("alert")).textContent).toMatch(/not the ones expected/);
    expect(screen.getByRole("textbox")).toBeTruthy();
  });

  it("looks again when the ask is gone, rather than leaving a form that cannot work", async () => {
    ipc.newcomers = [mac];
    ipc.refuse = { cmd: "admitting_grant", code: "admitGone" };
    render(<LetThemIn onClose={() => {}} />);
    await userEvent.click(await screen.findByRole("button", { name: "Let it in" }));
    await userEvent.type(await screen.findByRole("textbox"), "00000000000000000000");
    ipc.newcomers = [];
    await userEvent.click(letItIn());

    expect(await screen.findByText(/No new computer is asking to come in/)).toBeTruthy();
    expect(screen.getByRole("alert").textContent).toMatch(/no longer asking to come in/);
  });

  it("takes the ask away when there is no such computer", async () => {
    ipc.newcomers = [mac];
    render(<LetThemIn onClose={() => {}} />);
    await userEvent.click(await screen.findByRole("button", { name: "Let it in" }));
    await userEvent.click(await screen.findByRole("button", { name: "There is no such computer" }));

    expect(await screen.findByText(/the request was taken away/)).toBeTruthy();
    expect(sent("admitting_refuse")[0]?.args.device).toBe("dev_b");
    expect(sent("admitting_grant")).toHaveLength(0);
  });

  it("says where to start when nobody asks, and looks again", async () => {
    render(<LetThemIn onClose={() => {}} />);
    expect(await screen.findByText(/No new computer is asking to come in/)).toBeTruthy();
    expect(screen.getByRole("status").textContent).toBe("Still looking every few seconds…");

    await userEvent.click(screen.getByRole("button", { name: "Look again" }));

    await waitFor(() => expect(sent("admitting_look")).toHaveLength(2));
  });

  it("says in plain words why the computers could not be looked at", async () => {
    ipc.refuse = { cmd: "admitting_look", code: "admitNotNow" };
    render(<LetThemIn onClose={() => {}} />);

    expect((await screen.findByRole("alert")).textContent).toMatch(/both computers are on/);
  });
});
