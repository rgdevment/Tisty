import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import Composed from "../ui/Composed";

const calls = vi.hoisted(() => [] as string[]);

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string, args?: Record<string, unknown>) => {
    calls.push(`${cmd}:${String(args?.reference ?? args?.path ?? "")}`);
    return Promise.resolve(cmd === "served" ? "/store/x.png" : null);
  },
  convertFileSrc: (at: string) => `asset://${at}`,
}));

vi.mock("@tauri-apps/plugin-opener", () => ({
  openUrl: (at: string) => {
    calls.push(`openUrl:${at}`);
    return Promise.resolve();
  },
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
});
