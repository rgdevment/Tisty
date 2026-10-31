import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { fill, t } from "../locales";

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: unknown }[],
  secrets: [] as { line: number; named: string }[],
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
    if (cmd === "share_secrets") return Promise.resolve(ipc.secrets);
    return Promise.resolve(cmd === "share_doc" ? CARD : null);
  },
}));

import Sharing from "../ui/Sharing";

const asked = (cmd: string) => ipc.calls.filter((one) => one.cmd === cmd).map((one) => one.args);

beforeEach(() => {
  ipc.calls = [];
  ipc.secrets = [];
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
    await userEvent.click(screen.getByRole("button", { name: t("shareGo") }));
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
    const form = screen
      .getByRole("button", { name: t("shareGo") })
      .closest("form") as HTMLFormElement;
    // Both in one go, before the dialog can redraw between them.
    act(() => {
      form.dispatchEvent(new Event("submit", { bubbles: true, cancelable: true }));
      form.dispatchEvent(new Event("submit", { bubbles: true, cancelable: true }));
    });
    await waitFor(() => expect(asked("share_doc")).toHaveLength(1));
  });

  it("asks for a password before sharing with one, and hands out the edit link only when asked", async () => {
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
    const go = screen.getByRole("button", { name: t("shareGo") });
    expect(go).toHaveProperty("disabled", true);
    await userEvent.type(screen.getByLabelText(t("sharePassword")), "ñandú");
    await userEvent.click(screen.getByRole("radio", { name: t("shareCanEdit") }));
    fireEvent.change(screen.getByLabelText(t("shareEnds")), { target: { value: "7" } });
    await userEvent.click(go);
    await waitFor(() => expect(asked("share_doc")).toHaveLength(1));
    expect(asked("share_doc")[0]).toEqual({
      file: "notas.md",
      chosen: { password: "ñandú", editable: true, idleDays: 7 },
    });
    expect(screen.getByRole("option", { name: fill("shareEndsIdle", "7") })).toBeTruthy();
  });

  it("names the lines that read like a key and still shares when asked to", async () => {
    ipc.secrets = [
      { line: 4, named: "AWS_SECRET_ACCESS_KEY" },
      { line: 9, named: "" },
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
    expect(said).toContain(`${fill("shareSecretAt", "4")}AWS_SECRET_ACCESS_KEY`);
    expect(said).toContain(fill("shareSecretAt", "9"));
    expect(asked("share_secrets")).toEqual([{ file: "deploy.md" }]);

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
    await waitFor(() => expect(asked("share_secrets")).toHaveLength(1));
    expect(screen.queryByRole("alert")).toBeNull();
    expect(screen.getByRole("button", { name: t("shareGo") })).toBeTruthy();
  });
});
