import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it } from "vitest";
import type { Pick } from "../core";
import { t } from "../locales";
import Versions from "../ui/Versions";

const shown = () => {
  const picked: (Pick | null)[] = [];
  render(<Versions named="Kit de transmisión" onPick={(pick) => picked.push(pick)} />);
  return picked;
};

describe("two whole versions of a document", () => {
  it("names the document and hands back the side picked", async () => {
    const picked = shown();

    expect(screen.getByText(/Kit de transmisión/)).toBeTruthy();
    await userEvent.click(screen.getByText(t("versionsKeepTheirs")));

    expect(picked).toEqual(["theirs"]);
  });

  it("leaves it for later without picking a side", async () => {
    const picked = shown();

    await userEvent.click(screen.getByText(t("versionsLater")));

    expect(picked).toEqual([null]);
  });
});
