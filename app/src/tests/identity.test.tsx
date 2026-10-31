import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { Identity as Shown } from "../identity";
import type { Word } from "../ui/Card";
import Identity from "../ui/Identity";
import SettingsNav from "../ui/SettingsNav";

const WORDS = Array.from({ length: 24 }, (_, at) => `word${at + 1}`);

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: Record<string, unknown> }[],
  shown: null as unknown,
  wrong: [] as number[],
  refuse: null as string | null,
}));

const asked = vi.hoisted(() => ({ file: null as string | null }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    ipc.calls.push({ cmd, args: args ?? {} });
    if (cmd === ipc.refuse) return Promise.reject({ code: "identityNone" });
    if (cmd === "identity_shown") return Promise.resolve(ipc.shown);
    if (cmd === "identity_phrase" || cmd === "identity_begin")
      return Promise.resolve(WORDS.join(" "));
    if (cmd === "identity_confirm") return Promise.resolve(ipc.wrong);
    if (cmd === "page_pdf") return Promise.resolve([37, 80, 68, 70]);
    return Promise.resolve(null);
  },
}));

vi.mock("@tauri-apps/plugin-dialog", () => ({
  save: () => Promise.resolve(asked.file),
}));

const held: Shown = {
  held: true,
  elsewhere: false,
  fingerprint: "48210 93755 12880 63349",
  suffix: "k7qp",
  confirmed: false,
  alias: "Mario",
  machines: [
    { id: "dev_here", name: "MacBook", here: true, ours: true },
    { id: "dev_office", name: "Office PC", here: false, ours: false },
  ],
};

const sent = (cmd: string) => ipc.calls.filter((one) => one.cmd === cmd);

function Shell({ onSync = () => {}, phrased }: { onSync?: () => void; phrased?: number }) {
  const [said, setSaid] = useState<Word>();
  const [trouble, setTrouble] = useState<Word>();
  return (
    <Identity
      phrased={phrased}
      busy={null}
      said={said}
      trouble={trouble}
      tell={setSaid}
      fail={setTrouble}
      onSync={onSync}
    />
  );
}

const seePhrase = async () => {
  await userEvent.click(await screen.findByRole("button", { name: "See the phrase" }));
  await screen.findByText("word24");
};

beforeEach(() => {
  ipc.calls.length = 0;
  ipc.shown = held;
  ipc.wrong = [];
  ipc.refuse = null;
  asked.file = null;
});

describe("your identity", () => {
  it("says only when it is born while there is none", async () => {
    ipc.shown = { ...held, held: false, fingerprint: null, machines: [] };
    render(<Shell />);

    expect(await screen.findByText(/born when you create your 24-word phrase/)).toBeTruthy();
    expect(screen.queryByRole("button", { name: "See the phrase" })).toBeNull();
  });

  it("creates the phrase here, without a cloud to encrypt", async () => {
    ipc.shown = { ...held, held: false, fingerprint: null, machines: [] };
    render(<Shell />);

    await userEvent.click(await screen.findByRole("button", { name: "Create my phrase" }));
    expect(ipc.calls.some((one) => one.cmd === "identity_begin")).toBe(true);
  });

  it("takes the phrase typed on another computer, and gives this one its key", async () => {
    ipc.shown = { ...held, held: false, fingerprint: null, machines: [] };
    render(<Shell />);

    await userEvent.click(await screen.findByRole("button", { name: "I already have my phrase" }));
    await userEvent.type(screen.getByLabelText(/Type your 24 words/), WORDS.join(" "));
    await userEvent.click(screen.getByRole("button", { name: "Give this computer its key" }));
    await waitFor(() =>
      expect(ipc.calls.find((one) => one.cmd === "identity_recover")?.args).toEqual({
        said: WORDS.join(" "),
      }),
    );
  });

  it("shows the fingerprint, the name, the phrase's standing and whose computers they are", async () => {
    const onSync = vi.fn();
    render(<Shell onSync={onSync} />);

    expect(await screen.findByText("48210 93755 12880 63349")).toBeTruthy();
    expect(screen.getByText("Mario · k7qp")).toBeTruthy();
    expect(screen.getByText("not confirmed")).toBeTruthy();
    expect(screen.getByText("This computer")).toBeTruthy();
    expect(screen.getByText("Office PC")).toBeTruthy();
    expect(screen.getAllByText("yours")).toHaveLength(1);

    await userEvent.click(screen.getByRole("button", { name: "Remove a computer" }));
    expect(onSync).toHaveBeenCalled();
  });

  it("says when the phrase was confirmed", async () => {
    ipc.shown = { ...held, confirmed: true };
    render(<Shell />);

    expect(await screen.findByText("confirmed")).toBeTruthy();
  });

  it("looks again once the phrase was confirmed from the encryption card", async () => {
    const { rerender } = render(<Shell phrased={0} />);
    expect(await screen.findByText("not confirmed")).toBeTruthy();

    ipc.shown = { ...held, confirmed: true };
    rerender(<Shell phrased={1} />);

    expect(await screen.findByText("confirmed")).toBeTruthy();
  });

  it("offers no phrase on a computer that does not keep it", async () => {
    ipc.shown = { ...held, held: false, elsewhere: true, fingerprint: null };
    render(<Shell />);

    expect(await screen.findByText(/does not keep its phrase yet/)).toBeTruthy();
    expect(screen.queryByRole("button", { name: "See the phrase" })).toBeNull();
    expect(screen.queryByText("not confirmed")).toBeNull();
  });

  it("says why it could not be read", async () => {
    ipc.refuse = "identity_shown";
    render(<Shell />);

    expect(await screen.findByText(/does not keep your identity's phrase/)).toBeTruthy();
  });

  it("says why the phrase could not be shown", async () => {
    render(<Shell />);
    ipc.refuse = "identity_phrase";

    await userEvent.click(await screen.findByRole("button", { name: "See the phrase" }));

    expect(await screen.findByText(/does not keep your identity's phrase/)).toBeTruthy();
    expect(screen.queryByRole("dialog")).toBeNull();
  });
});

describe("the phrase", () => {
  it("shows all 24 words in order and closes without asking anything", async () => {
    render(<Shell />);
    await seePhrase();

    expect(screen.getAllByRole("listitem").map((one) => one.textContent)).toEqual(
      expect.arrayContaining(["1word1", "24word24"]),
    );
    expect(screen.getByText("Keep it outside Tisty.")).toBeTruthy();

    await userEvent.click(screen.getByRole("button", { name: "Close" }));
    expect(screen.queryByRole("dialog")).toBeNull();
  });

  it("saves the sheet where the person chose and says so inside the window", async () => {
    asked.file = "/tmp/phrase.pdf";
    render(<Shell />);
    await seePhrase();

    await userEvent.click(screen.getByRole("button", { name: "Save as PDF" }));
    expect(screen.getByText(/Whoever has this sheet/)).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: "Save" }));

    expect(await screen.findByText(/The PDF was saved/)).toBeTruthy();
    const page = String(sent("page_pdf")[0].args.html);
    expect(page).toContain("word1");
    expect(page).toContain("word24");
    expect(page).toContain("48210 93755 12880 63349");
    expect(sent("page_pdf")[0].args.secret).toBe(true);
    expect(sent("keep_file")[0].args.at).toBe("/tmp/phrase.pdf");
  });

  it("writes nothing when the save dialog is left", async () => {
    render(<Shell />);
    await seePhrase();

    await userEvent.click(screen.getByRole("button", { name: "Save as PDF" }));
    await userEvent.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() => expect(screen.getByRole("button", { name: "Save" })).toBeTruthy());
    expect(sent("page_pdf")).toHaveLength(0);
    expect(sent("keep_file")).toHaveLength(0);

    await userEvent.click(screen.getByRole("button", { name: "Cancel" }));
    expect(screen.getByRole("button", { name: "I've saved it" })).toBeTruthy();
  });

  it("says inside the window why the sheet could not be saved", async () => {
    asked.file = "/tmp/phrase.pdf";
    ipc.refuse = "keep_file";
    render(<Shell />);
    await seePhrase();

    await userEvent.click(screen.getByRole("button", { name: "Save as PDF" }));
    await userEvent.click(screen.getByRole("button", { name: "Save" }));

    expect(await screen.findByRole("alert")).toBeTruthy();
  });
});

describe("confirming the phrase", () => {
  const places = () =>
    screen.getAllByText(/^Word \d+$/).map((one) => Number(one.textContent?.split(" ")[1]));

  const typeAll = async () => {
    for (const box of screen.getAllByRole("textbox")) {
      await userEvent.type(box, "something");
    }
  };

  it("asks for one word from each third of the phrase", async () => {
    render(<Shell />);
    await seePhrase();
    await userEvent.click(screen.getByRole("button", { name: "I've saved it" }));

    const asked = places();
    expect(asked).toHaveLength(3);
    expect(asked[0]).toBeGreaterThanOrEqual(1);
    expect(asked[0]).toBeLessThanOrEqual(8);
    expect(asked[1]).toBeGreaterThanOrEqual(9);
    expect(asked[1]).toBeLessThanOrEqual(16);
    expect(asked[2]).toBeGreaterThanOrEqual(17);
    expect(asked[2]).toBeLessThanOrEqual(24);
    expect(screen.getByRole("button", { name: "Confirm" }).hasAttribute("disabled")).toBe(true);
  });

  it("names the word that does not match, and lets the others stand", async () => {
    render(<Shell />);
    await seePhrase();
    await userEvent.click(screen.getByRole("button", { name: "I've saved it" }));
    const [first, second] = places();
    ipc.wrong = [second];

    await typeAll();
    await userEvent.click(screen.getByRole("button", { name: "Confirm" }));

    expect(await screen.findByText("check it")).toBeTruthy();
    expect(screen.getAllByText("✓")).toHaveLength(2);
    const said = sent("identity_confirm")[0].args.said as [number, string][];
    expect(said.map(([at]) => at)).toContain(first);
    expect(screen.getByRole("dialog")).toBeTruthy();

    fireEvent.change(screen.getAllByRole("textbox")[1], { target: { value: "again" } });
    expect(screen.queryByText("check it")).toBeNull();
  });

  it("closes and says the phrase is confirmed once every word matches", async () => {
    render(<Shell />);
    await seePhrase();
    await userEvent.click(screen.getByRole("button", { name: "I've saved it" }));

    await typeAll();
    ipc.shown = { ...held, confirmed: true };
    await userEvent.click(screen.getByRole("button", { name: "Confirm" }));

    await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull());
    expect(await screen.findByText("Your phrase is confirmed.")).toBeTruthy();
    expect(await screen.findByText("confirmed")).toBeTruthy();
  });

  it("says inside the window why it could not check", async () => {
    render(<Shell />);
    await seePhrase();
    await userEvent.click(screen.getByRole("button", { name: "I've saved it" }));
    ipc.refuse = "identity_confirm";

    await typeAll();
    await userEvent.click(screen.getByRole("button", { name: "Confirm" }));

    expect(await screen.findByRole("alert")).toBeTruthy();
    await userEvent.click(screen.getByRole("button", { name: "See the phrase again" }));
    expect(screen.getByText("word24")).toBeTruthy();
  });
});

describe("the settings sidebar", () => {
  const nav = (waiting?: string) => {
    const onBack = vi.fn();
    const onTab = vi.fn();
    render(
      <>
        <SettingsNav tab="security" waiting={waiting} onTab={onTab} onBack={onBack} />
        <input aria-label="field" />
      </>,
    );
    return { onBack, onTab };
  };

  it("goes back with its button or Escape, never while typing", async () => {
    const { onBack } = nav();

    await userEvent.click(screen.getByRole("button", { name: /Back to Tisty/ }));
    expect(onBack).toHaveBeenCalledTimes(1);

    fireEvent.keyDown(window, { key: "Escape" });
    expect(onBack).toHaveBeenCalledTimes(2);

    fireEvent.keyDown(screen.getByLabelText("field"), { key: "Escape" });
    expect(onBack).toHaveBeenCalledTimes(2);
  });

  it("leaves a dialog to close itself", () => {
    const { onBack } = nav();
    const dialog = document.createElement("div");
    dialog.setAttribute("role", "dialog");
    document.body.append(dialog);

    fireEvent.keyDown(window, { key: "Escape" });
    expect(onBack).not.toHaveBeenCalled();
    dialog.remove();
  });

  it("marks About while a newer version waits, and opens each section", async () => {
    const { onTab } = nav("1.36.0");

    expect(screen.getByRole("tab", { name: /About · a newer version exists/ })).toBeTruthy();
    expect(screen.getByRole("tab", { name: "Security" }).getAttribute("aria-selected")).toBe(
      "true",
    );

    await userEvent.click(screen.getByRole("tab", { name: /About/ }));
    expect(onTab).toHaveBeenCalledWith("about");
  });
});
