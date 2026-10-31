import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { fill, t } from "../locales";

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: unknown }[],
  secrets: [] as { line: number; named: string; under: string | null }[],
  signsAs: "Ana" as string | null,
  pending: undefined as Promise<unknown> | undefined,
  changed: false,
}));

const CARD = {
  read: "https://tisty.cl/k7Qm2xPa#llave",
  edit: null,
  gated: false,
  version: 1,
  info: null,
};

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args: unknown) => {
    ipc.calls.push({ cmd, args });
    if (cmd === "share_doc" && ipc.changed) {
      ipc.changed = false;
      return Promise.reject({ code: "shareChanged" });
    }
    if (cmd === "share_look")
      return ipc.pending ?? Promise.resolve({ secrets: ipc.secrets, signsAs: ipc.signsAs });
    return Promise.resolve(cmd === "share_doc" ? CARD : null);
  },
}));

import Sharing from "../ui/Sharing";

const asked = (cmd: string) => ipc.calls.filter((one) => one.cmd === cmd).map((one) => one.args);

beforeEach(() => {
  ipc.calls = [];
  ipc.secrets = [];
  ipc.signsAs = "Ana";
  ipc.pending = undefined;
  ipc.changed = false;
});

describe("the share dialog", () => {
  it("shares to read only and for the server's own time unless asked otherwise, always signed", async () => {
    const onShared = vi.fn();
    render(
      <Sharing
        file="notas.md"
        title="Notas"
        onShared={onShared}
        onClose={() => {}}
        onError={() => {}}
      />,
    );
    await userEvent.click(await screen.findByRole("button", { name: t("shareGo") }));
    await waitFor(() => expect(onShared).toHaveBeenCalledWith(CARD));
    expect(asked("share_doc")).toEqual([
      {
        file: "notas.md",
        chosen: { password: null, editable: false, idleDays: null },
      },
    ]);
  });

  it("shares once however quickly it is pressed again", async () => {
    render(
      <Sharing
        file="notas.md"
        title="Notas"
        onShared={() => {}}
        onClose={() => {}}
        onError={() => {}}
      />,
    );
    const go = await screen.findByRole("button", { name: t("shareGo") });
    const form = go.closest("form") as HTMLFormElement;
    // Both in one go, before the dialog can redraw between them.
    act(() => {
      form.dispatchEvent(new Event("submit", { bubbles: true, cancelable: true }));
      form.dispatchEvent(new Event("submit", { bubbles: true, cancelable: true }));
    });
    await waitFor(() => expect(asked("share_doc")).toHaveLength(1));
  });

  it("asks for a password before sharing with one, and hands out no edit link yet", async () => {
    render(
      <Sharing
        file="notas.md"
        title="Notas"
        onShared={() => {}}
        onClose={() => {}}
        onError={() => {}}
      />,
    );
    await userEvent.click(screen.getByRole("radio", { name: new RegExp(t("shareGated")) }));
    const go = await screen.findByRole("button", { name: t("shareGo") });
    expect(go).toHaveProperty("disabled", true);
    await userEvent.type(screen.getByLabelText(t("sharePassword")), "ñandú");
    // No edit link is offered until a share can be edited from Tisty.
    expect(screen.queryByRole("radio", { name: t("shareCanEdit") })).toBeNull();
    fireEvent.change(screen.getByLabelText(t("shareEnds")), { target: { value: "7" } });
    await userEvent.click(go);
    await waitFor(() => expect(asked("share_doc")).toHaveLength(1));
    expect(asked("share_doc")[0]).toEqual({
      file: "notas.md",
      chosen: { password: "ñandú", editable: false, idleDays: 7 },
    });
    expect(screen.getByRole("option", { name: fill("shareEndsIdle", "7") })).toBeTruthy();
  });

  it("names the lines that read like a key and still shares when asked to", async () => {
    ipc.secrets = [
      { line: 4, named: "AWS_SECRET_ACCESS_KEY", under: "Deploy" },
      { line: 9, named: "", under: null },
      { line: 12, named: "ghp_", under: "Deploy" },
      { line: 15, named: "sk_live_", under: "Pagos" },
    ];
    const onShared = vi.fn();
    render(
      <Sharing
        file="deploy.md"
        title="Deploy"
        onShared={onShared}
        onClose={() => {}}
        onError={() => {}}
      />,
    );

    const warning = await screen.findByRole("alert");
    const said = warning.textContent ?? "";
    expect(said).toContain(t("shareSecrets"));
    expect(said).toContain(`${fill("shareSecretUnder", "Deploy")}AWS_SECRET_ACCESS_KEY`);
    expect(said).toContain(t("shareSecretTop"));
    expect(said).toContain(fill("shareSecretsMore", "1"));
    expect(said).not.toContain("sk_live_");
    expect(asked("share_look")).toEqual([{ file: "deploy.md" }]);

    await userEvent.click(screen.getByRole("button", { name: t("shareGoAnyway") }));
    await waitFor(() => expect(onShared).toHaveBeenCalled());
  });

  it("says nothing when no line reads like a key", async () => {
    render(
      <Sharing
        file="notas.md"
        title="Notas"
        onShared={() => {}}
        onClose={() => {}}
        onError={() => {}}
      />,
    );
    // Nothing goes out while the document is still being looked over.
    const looking = screen.getByRole("button", { name: t("shareLooking") }) as HTMLButtonElement;
    expect(looking.disabled).toBe(true);
    await waitFor(() => expect(asked("share_look")).toHaveLength(1));
    expect(screen.queryByRole("alert")).toBeNull();
    expect(await screen.findByRole("button", { name: t("shareGo") })).toBeTruthy();
  });

  it("says the name the share goes as, or that the key signs alone", async () => {
    const { unmount } = render(
      <Sharing
        file="notas.md"
        title="Notas"
        onShared={() => {}}
        onClose={() => {}}
        onError={() => {}}
      />,
    );
    expect(await screen.findByText(new RegExp(fill("shareSignsAs", "Ana")))).toBeTruthy();
    unmount();
    ipc.signsAs = null;
    render(
      <Sharing
        file="notas.md"
        title="Notas"
        onShared={() => {}}
        onClose={() => {}}
        onError={() => {}}
      />,
    );
    expect(await screen.findByText(new RegExp(t("shareSignsAlone")))).toBeTruthy();
  });

  it("looks a new file over afresh, forgetting the last one's warning", async () => {
    ipc.secrets = [{ line: 4, named: "AWS_SECRET_ACCESS_KEY", under: "Deploy" }];
    const props = { title: "x", onShared: () => {}, onClose: () => {}, onError: () => {} };
    const { rerender } = render(<Sharing file="a.md" {...props} />);
    await screen.findByRole("alert");
    let answer: (look: unknown) => void = () => {};
    ipc.pending = new Promise((done) => {
      answer = done;
    });
    rerender(<Sharing file="b.md" {...props} />);
    await waitFor(() => expect(screen.queryByRole("alert")).toBeNull());
    expect(
      (screen.getByRole("button", { name: t("shareLooking") }) as HTMLButtonElement).disabled,
    ).toBe(true);
    await act(async () => answer({ secrets: [], signsAs: "Ana" }));
    expect(await screen.findByRole("button", { name: t("shareGo") })).toBeTruthy();
  });

  it("says nothing about the name when the look could not be made", async () => {
    ipc.pending = Promise.reject(new Error("unreadable"));
    render(
      <Sharing
        file="notas.md"
        title="Notas"
        onShared={() => {}}
        onClose={() => {}}
        onError={() => {}}
      />,
    );
    expect(await screen.findByRole("button", { name: t("shareGo") })).toBeTruthy();
    expect(screen.queryByText(new RegExp(t("shareSignsAlone")))).toBeNull();
  });

  it("looks the words over again when they changed after the look, and shares nothing meanwhile", async () => {
    ipc.changed = true;
    const onShared = vi.fn();
    const onError = vi.fn();
    render(
      <Sharing
        file="notas.md"
        title="Notas"
        onShared={onShared}
        onClose={() => {}}
        onError={onError}
      />,
    );
    await userEvent.click(await screen.findByRole("button", { name: t("shareGo") }));
    await waitFor(() => expect(asked("share_look")).toHaveLength(2));
    expect(onShared).not.toHaveBeenCalled();
    expect(onError).not.toHaveBeenCalled();
    await userEvent.click(await screen.findByRole("button", { name: t("shareGo") }));
    await waitFor(() => expect(onShared).toHaveBeenCalled());
  });
});
