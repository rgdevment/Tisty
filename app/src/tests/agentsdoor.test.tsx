import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { t } from "../locales";
import Agents from "../ui/Agents";
import { IDLE } from "../ui/working";
import { knownFor } from "./knownFor";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: () => Promise.resolve(null),
}));

const known = knownFor();

const shown = (binary?: string) =>
  render(
    <Agents
      busy={IDLE}
      quietly={() => {}}
      tell={() => {}}
      fail={() => {}}
      binary={binary}
      known={known}
    />,
  );

const connected = () => fireEvent.click(screen.getByRole("tab", { name: "Connected" }));

describe("how to connect an assistant", () => {
  it("says the snippets carry the path when there is one beside the window", () => {
    shown("C:/Programs/Tisty/tisty.exe");
    connected();

    expect(screen.getByText(t("agentsHow"))).toBeTruthy();
  });

  it("says they call the name alone when the Store's copy has no path to give", () => {
    shown();
    connected();

    expect(screen.getByText(t("agentsHowAlias"))).toBeTruthy();
    expect(screen.queryByText(t("agentsHow"))).toBeNull();
  });
});
