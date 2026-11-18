import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import Composed from "../ui/Composed";

const calls = vi.hoisted(() => [] as string[]);

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    if (cmd === "open_link") {
      calls.push(`openUrl:${String(args?.url ?? "")}`);
      return Promise.resolve(null);
    }
    calls.push(`${cmd}:${String(args?.reference ?? args?.path ?? "")}`);
    return Promise.resolve(cmd === "served" ? "/store/x.png" : null);
  },
  convertFileSrc: (at: string) => `asset://${at}`,
}));

describe("following a link inside a document", () => {
  beforeEach(() => {
    calls.length = 0;
  });

  it("never asks the file manager for anything when the target is ours", async () => {
    render(
      <Composed
        className=""
        html={
          '<p><a data-inside="attachments/bb/report-9999.pdf" href="attachments/bb/report-9999.pdf">the report</a></p>'
        }
      />,
    );

    await userEvent.click(screen.getByText("the report"));

    expect(calls).toEqual(["opened:attachments/bb/report-9999.pdf"]);
  });

  it("still opens a real web address outside", async () => {
    render(<Composed className="" html={'<p><a href="https://example.org">out</a></p>'} />);

    await userEvent.click(screen.getByText("out"));

    expect(calls).toEqual(["openUrl:https://example.org"]);
  });

  it("hands the address of a document of the store to the app, never to the file manager", async () => {
    const href = "tisty://doc/personal/minuta?id=dev_a-0002";
    const onDoc = vi.fn();
    render(<Composed className="" onDoc={onDoc} html={`<p><a href="${href}">la minuta</a></p>`} />);

    await userEvent.click(screen.getByText("la minuta"));

    expect(calls).toEqual([`openUrl:${href}`]);
    expect(onDoc).not.toHaveBeenCalled();
  });
});
