import { act, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { fill, t } from "../locales";
import { plainly } from "../refusal";

const ipc = vi.hoisted(() => ({
  calls: [] as { cmd: string; args: unknown }[],
  secrets: [] as { line: number; named: string; under: string | null }[],
  name: "Ana Pérez" as string | null,
  alias: "ana" as string | null,
  pages: 0,
  cited: 0,
  pictures: 0,
  citedPictures: 0,
  shares: [] as { file: string; title: string }[],
  level: undefined as { shares: number; cited: number; pictures: number } | undefined,
  pending: undefined as Promise<unknown> | undefined,
  changed: false,
  full: false,
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
    if (cmd === "share_doc" && ipc.full) {
      ipc.full = false;
      return Promise.reject({ code: "shareFull", name: "2" });
    }
    if (cmd === "share_look")
      return (
        ipc.pending ??
        Promise.resolve({
          secrets: ipc.secrets,
          name: ipc.name,
          alias: ipc.alias,
          pages: ipc.pages,
          cited: ipc.cited,
          pictures: ipc.pictures,
          citedPictures: ipc.citedPictures,
          shares: ipc.shares,
          level: ipc.level,
        })
      );
    if (cmd === "share_stop") {
      const { file } = args as { file: string };
      ipc.shares = ipc.shares.filter((one) => one.file !== file);
    }
    return Promise.resolve(cmd === "share_doc" ? CARD : null);
  },
}));

import Sharing from "../ui/Sharing";

const asked = (cmd: string) => ipc.calls.filter((one) => one.cmd === cmd).map((one) => one.args);

beforeEach(() => {
  ipc.calls = [];
  ipc.secrets = [];
  ipc.name = "Ana Pérez";
  ipc.alias = "ana";
  ipc.pages = 0;
  ipc.cited = 0;
  ipc.pictures = 0;
  ipc.citedPictures = 0;
  ipc.shares = [];
  ipc.level = undefined;
  ipc.pending = undefined;
  ipc.changed = false;
  ipc.full = false;
});

describe("the share dialog", () => {
  it("shares to read only, following saves, under the alias and for a day unless asked otherwise", async () => {
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
    const ends = screen.getByLabelText(t("shareEnds")) as HTMLSelectElement;
    expect(ends.selectedOptions[0]?.textContent).toBe(t("shareEndsIdleDay"));
    expect([...ends.options].map((one) => one.value)).toEqual(["1", "3", "7", "15"]);
    await userEvent.click(await screen.findByRole("button", { name: t("shareGo") }));
    await waitFor(() => expect(onShared).toHaveBeenCalledWith(CARD));
    expect(asked("share_doc")).toEqual([
      {
        file: "notas.md",
        chosen: {
          password: null,
          editable: false,
          idleDays: 1,
          signs: "alias",
          still: false,
          cites: false,
        },
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

  it("asks for a password before sharing with one, and hands out the edit link when asked", async () => {
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
    // Reading only is the default; editing is chosen (decision 67).
    await userEvent.click(screen.getByRole("radio", { name: t("shareCanEdit") }));
    fireEvent.change(screen.getByLabelText(t("shareEnds")), { target: { value: "7" } });
    await userEvent.click(go);
    await waitFor(() => expect(asked("share_doc")).toHaveLength(1));
    expect(asked("share_doc")[0]).toEqual({
      file: "notas.md",
      chosen: {
        password: "ñandú",
        editable: true,
        idleDays: 7,
        signs: "alias",
        still: false,
        cites: false,
      },
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

  it("goes under the name or only the key when chosen, and stays as it is when asked", async () => {
    render(
      <Sharing
        file="notas.md"
        title="Notas"
        onShared={() => {}}
        onClose={() => {}}
        onError={() => {}}
      />,
    );
    await userEvent.click(
      await screen.findByRole("radio", { name: fill("shareSignName", "Ana Pérez") }),
    );
    await userEvent.click(screen.getByRole("checkbox", { name: t("shareSynced") }));
    expect(screen.getByText(t("sharedStill"))).toBeTruthy();
    await userEvent.click(await screen.findByRole("button", { name: t("shareGo") }));
    await waitFor(() => expect(asked("share_doc")).toHaveLength(1));
    expect(asked("share_doc")[0]).toMatchObject({ chosen: { signs: "name", still: true } });
  });

  it("offers only the names fit to sign, and says when the key signs alone", async () => {
    ipc.alias = null;
    const { unmount } = render(
      <Sharing
        file="notas.md"
        title="Notas"
        onShared={() => {}}
        onClose={() => {}}
        onError={() => {}}
      />,
    );
    // Without an alias the full name is still never picked for the person.
    const keyOnly = await screen.findByRole("radio", { name: t("shareSignKey") });
    expect((keyOnly as HTMLInputElement).checked).toBe(true);
    expect(screen.queryByRole("radio", { name: /«ana»/ })).toBeNull();
    unmount();
    ipc.name = null;
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
    await act(async () => answer({ secrets: [], name: null, alias: "ana" }));
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
    expect(await screen.findByText(t("shareChanged"))).toBeTruthy();
    expect(onShared).not.toHaveBeenCalled();
    expect(onError).not.toHaveBeenCalled();
    await userEvent.click(await screen.findByRole("button", { name: t("shareGo") }));
    await waitFor(() => expect(onShared).toHaveBeenCalled());
  });

  it("says its pages go with it and shares what it cites only when asked", async () => {
    ipc.pages = 3;
    ipc.cited = 2;
    render(
      <Sharing
        file="guia.md"
        title="Guía"
        onShared={() => {}}
        onClose={() => {}}
        onError={() => {}}
      />,
    );
    expect(await screen.findByText(fill("sharePages", "3"))).toBeTruthy();
    const cites = screen.getByRole("checkbox", { name: fill("shareCites", "2") });
    expect((cites as HTMLInputElement).checked).toBe(false);
    await userEvent.click(cites);
    await userEvent.click(await screen.findByRole("button", { name: t("shareGo") }));
    await waitFor(() => expect(asked("share_doc")).toHaveLength(1));
    expect(asked("share_doc")[0]).toMatchObject({ chosen: { cites: true } });
  });

  it("says nothing of pages or citations when it has none", async () => {
    render(
      <Sharing
        file="notas.md"
        title="Notas"
        onShared={() => {}}
        onClose={() => {}}
        onError={() => {}}
      />,
    );
    await screen.findByRole("button", { name: t("shareGo") });
    // Only the choice to keep it in sync, never one about what it cites.
    expect(
      screen
        .getAllByRole("checkbox")
        .map((one) => one.getAttribute("aria-label") ?? one.parentElement?.textContent),
    ).toEqual([t("shareSynced")]);
    expect(screen.queryByText(t("shareCitesWhy"))).toBeNull();
  });

  it("says how many it shares and, once asked to be sure, stops one to make room for this one", async () => {
    ipc.level = { shares: 2, cited: 2, pictures: 2 };
    ipc.shares = [
      { file: "uno.md", title: "Uno" },
      { file: "dos.md", title: "Dos" },
    ];
    const onShared = vi.fn();
    render(
      <Sharing
        file="tres.md"
        title="Tres"
        onShared={onShared}
        onClose={() => {}}
        onError={() => {}}
      />,
    );
    expect(await screen.findByText(fill("shareFullCount", "2"))).toBeTruthy();
    expect(screen.getByText(fill("shareFull", "2"))).toBeTruthy();
    const go = screen.getByRole("button", { name: t("shareGo") }) as HTMLButtonElement;
    expect(go.disabled).toBe(true);

    await userEvent.click(screen.getByRole("button", { name: `${t("shareStop")}: Uno` }));
    expect(asked("share_stop")).toEqual([]);
    expect(screen.getByText(fill("shareStopAsk", "Uno"))).toBeTruthy();
    const row = () => screen.getByText(fill("shareStopAsk", "Uno")).closest("li") as HTMLElement;
    await userEvent.click(within(row()).getByRole("button", { name: t("cancel") }));
    expect(screen.queryByText(fill("shareStopAsk", "Uno"))).toBeNull();
    await userEvent.click(screen.getByRole("button", { name: `${t("shareStop")}: Uno` }));
    await userEvent.click(within(row()).getByRole("button", { name: t("shareStop") }));
    expect(asked("share_stop")).toEqual([{ file: "uno.md" }]);
    await waitFor(() => expect(asked("share_look")).toHaveLength(2));
    await waitFor(() => expect(screen.queryByText(fill("shareFull", "2"))).toBeNull());
    await userEvent.click(await screen.findByRole("button", { name: t("shareGo") }));
    await waitFor(() => expect(onShared).toHaveBeenCalled());
  });

  it("holds back a share whose pictures pass what one share carries, and says why", async () => {
    ipc.level = { shares: 2, cited: 2, pictures: 2 };
    ipc.cited = 1;
    ipc.pictures = 1;
    ipc.citedPictures = 2;
    render(
      <Sharing
        file="viaje.md"
        title="Viaje"
        onShared={() => {}}
        onClose={() => {}}
        onError={() => {}}
      />,
    );
    const go = (await screen.findByRole("button", { name: t("shareGo") })) as HTMLButtonElement;
    expect(go.disabled).toBe(false);
    await userEvent.click(screen.getByRole("checkbox", { name: t("shareCitesOne") }));
    const alert = await screen.findByRole("alert");
    expect(alert.textContent).toContain(fill("sharePicturesFull", "2"));
    expect(alert.textContent).toContain(fill("sharePicturesCount", "3"));
    expect(go.disabled).toBe(true);
  });

  it("looks again and offers what to stop when the server says the person shares too many already", async () => {
    ipc.level = { shares: 2, cited: 2, pictures: 2 };
    ipc.full = true;
    const onError = vi.fn();
    render(
      <Sharing
        file="tres.md"
        title="Tres"
        onShared={() => {}}
        onClose={() => {}}
        onError={onError}
      />,
    );
    await userEvent.click(await screen.findByRole("button", { name: t("shareGo") }));
    await waitFor(() => expect(asked("share_look")).toHaveLength(2));
    ipc.shares = [{ file: "uno.md", title: "Uno" }];
    expect(await screen.findByText(fill("shareFull", "2"))).toBeTruthy();
    expect(onError).not.toHaveBeenCalled();
    expect((screen.getByRole("button", { name: t("shareGo") }) as HTMLButtonElement).disabled).toBe(
      true,
    );
  });

  it("offers nothing it cites when its pages already fill one share", async () => {
    ipc.level = { shares: 2, cited: 2, pictures: 2 };
    ipc.pages = 2;
    ipc.cited = 1;
    render(
      <Sharing
        file="guia.md"
        title="Guía"
        onShared={() => {}}
        onClose={() => {}}
        onError={() => {}}
      />,
    );
    expect(await screen.findByText(fill("sharePages", "2"))).toBeTruthy();
    expect(screen.queryByRole("checkbox", { name: t("shareCitesOne") })).toBeNull();
    await userEvent.click(await screen.findByRole("button", { name: t("shareGo") }));
    await waitFor(() => expect(asked("share_doc")).toHaveLength(1));
    expect(asked("share_doc")[0]).toMatchObject({ chosen: { cites: false } });
  });

  it("says plainly why the server would take no more shares or pictures", () => {
    expect(plainly({ code: "shareFull", name: "2" })).toBe(fill("shareFull", "2"));
    expect(plainly({ code: "sharePicturesFull", name: "2" })).toBe(fill("sharePicturesFull", "2"));
    expect(plainly({ code: "shareTooManyCited", name: "2" })).toBe(fill("shareTooManyCited", "2"));
  });
});
