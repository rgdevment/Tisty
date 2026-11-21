import { render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { adopt } from "../locales";
import { articleOf, GROUPS, pageOf, TABS } from "../settingsPages";
import SettingsNav from "../ui/SettingsNav";

beforeEach(() => adopt("en"));

const tabs = () => screen.getAllByRole("tab").map((one) => one.textContent?.trim());

describe("the settings bar", () => {
  it("opens with the overview, then four groups of pages in the order the redesign set", () => {
    render(<SettingsNav tab="summary" onTab={() => {}} onBack={() => {}} />);

    expect(tabs()).toEqual([
      "Overview",
      "Look and window",
      "Tasks and reminders",
      "Documents and signature",
      "Sync",
      "Security",
      "Copies",
      "Storage",
      "Assistants",
      "Health",
      "Help and about",
    ]);
    const bar = screen.getByRole("tablist");
    for (const group of ["This computer", "Your data", "Connections", "Tisty"]) {
      expect(within(bar).getByText(group)).toBeTruthy();
    }
  });

  it("names the groups and pages in Spanish too", () => {
    adopt("es");
    render(<SettingsNav tab="summary" onTab={() => {}} onBack={() => {}} />);

    expect(tabs()).toEqual([
      "Resumen",
      "Aspecto y ventana",
      "Tareas y avisos",
      "Documentos y firma",
      "Sincronización",
      "Seguridad",
      "Copias",
      "Almacenamiento",
      "Asistentes",
      "Salud",
      "Ayuda y acerca de",
    ]);
    expect(screen.getByText("Este equipo")).toBeTruthy();
    expect(screen.getByText("Tus datos")).toBeTruthy();
    expect(screen.getByText("Conexiones")).toBeTruthy();
  });

  it("marks the page it is on and asks for the one pressed", async () => {
    const onTab = vi.fn();
    render(<SettingsNav tab="storage" onTab={onTab} onBack={() => {}} />);

    expect(screen.getByRole("tab", { name: "Storage" }).getAttribute("aria-selected")).toBe("true");
    expect(screen.getByRole("tab", { name: "Overview" }).getAttribute("aria-selected")).toBe(
      "false",
    );
    await userEvent.click(screen.getByRole("tab", { name: "Health" }));
    expect(onTab).toHaveBeenCalledWith("health");
    await userEvent.click(screen.getByRole("tab", { name: "Overview" }));
    expect(onTab).toHaveBeenCalledWith("summary");
  });

  it("puts the waiting update on Help and about", () => {
    render(<SettingsNav tab="summary" waiting="1.50.0" onTab={() => {}} onBack={() => {}} />);

    expect(
      screen.getByRole("tab", { name: "Help and about · a newer version exists" }),
    ).toBeTruthy();
  });

  it("goes back on Escape, unless a dialog is open", async () => {
    const onBack = vi.fn();
    render(<SettingsNav tab="summary" onTab={() => {}} onBack={onBack} />);

    await userEvent.keyboard("{Escape}");
    expect(onBack).toHaveBeenCalledTimes(1);

    const dialog = document.createElement("div");
    dialog.setAttribute("role", "dialog");
    document.body.append(dialog);
    await userEvent.keyboard("{Escape}");
    expect(onBack).toHaveBeenCalledTimes(1);
    dialog.remove();
  });
});

describe("the settings pages", () => {
  it("lists every page in one group exactly once, the overview apart", () => {
    const grouped = GROUPS.flatMap((group) => group.pages);
    expect(new Set(grouped).size).toBe(grouped.length);
    expect([...grouped].sort()).toEqual(TABS.filter((tab) => tab !== "summary").sort());
  });

  it("gives the overview a page of its own", () => {
    expect(pageOf("summary").label).toBe("tabSummary");
    expect(pageOf("summary").lead).toBe("summaryLead");
    expect(pageOf("summary").help).toBeUndefined();
  });

  it("links each page to its article in the language spoken", () => {
    expect(articleOf("security", true)).toEqual({
      slug: "sincronizar-y-respaldar",
      heading: "Cifrar tu nube",
    });
    expect(articleOf("agents", false)).toEqual({ slug: "assistants-and-mcp" });
    expect(articleOf("about", false)).toBeUndefined();
  });
});
