import { render, screen, waitFor } from "@testing-library/react";
import { useEffect } from "react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { adopt, t } from "../locales";
import Signing, { useSigning } from "../ui/Signing";

const invoked = vi.hoisted(() => vi.fn());
vi.mock("@tauri-apps/api/core", () => ({ invoke: invoked }));

const signedAs = (aliasImitates: boolean, nameImitates: boolean) =>
  invoked.mockImplementation((command: string) =>
    Promise.resolve(
      command === "signed"
        ? { alias: "rgdevrnent", name: null, before: [], mine: 0, aliasImitates, nameImitates }
        : null,
    ),
  );

function Held() {
  const signs = useSigning();
  const { lookSigned } = signs;
  useEffect(() => lookSigned(), [lookSigned]);
  return <Signing busy={null} run={vi.fn()} tell={vi.fn()} signs={signs} />;
}

beforeEach(() => {
  invoked.mockReset();
  adopt("en");
});

describe("a name like the maker's in Settings", () => {
  it("warns whoever signs with it that readers will see it flagged", async () => {
    signedAs(true, false);
    render(<Held />);

    expect(await screen.findByText(t("aliasImitatesMaker"))).toBeTruthy();
  });

  it("says nothing of a name that is not like the maker's", async () => {
    signedAs(false, false);
    render(<Held />);

    await waitFor(() => expect(invoked).toHaveBeenCalledWith("signed"));
    expect(screen.queryByText(t("aliasImitatesMaker"))).toBeNull();
  });
});
