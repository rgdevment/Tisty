import { render } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import type { Certainty, Mark as Kind } from "../core";
import Field, { type Mark } from "../ui/Field";

const at = (
  from: number,
  to: number,
  certainty: Certainty = "sure",
  extra: { offered?: boolean; overruled?: boolean; kind?: Kind } = {},
): Mark => ({
  span: { from, to, mark: extra.kind ?? "date", certainty },
  offered: extra.offered ?? false,
  overruled: extra.overruled,
});

function painted(value: string, marks: Mark[]) {
  const { container } = render(
    <Field icon="+" value={value} hint="add a task" marks={marks} onChange={() => {}} />,
  );
  const mirror = container.querySelector("[aria-hidden]");
  return Array.from(mirror?.querySelectorAll("span") ?? []).map((run) => ({
    text: run.textContent ?? "",
    look: run.className,
  }));
}

describe("the mirror", () => {
  it("wraps only what the parser claimed", () => {
    expect(painted("comprar pan mañana", [at(12, 18)]).map((run) => run.text)).toEqual([
      "comprar pan ",
      "mañana",
    ]);
  });

  it("counts code points, the only unit Rust and JS agree on", () => {
    expect(painted("🎉 fiesta mañana", [at(9, 15)]).map((run) => run.text)).toEqual([
      "🎉 fiesta ",
      "mañana",
    ]);
  });

  it("ignores a span the text no longer has room for", () => {
    const runs = painted("pan", [at(0, 99)]);
    expect(runs.map((run) => run.text)).toEqual(["pan"]);
    expect(runs[0].look).toBe("");
  });

  it("ignores a second span that overlaps the first", () => {
    expect(painted("comprar pan", [at(0, 7), at(3, 11)]).map((run) => run.text)).toEqual([
      "comprar",
      " pan",
    ]);
  });

  it("keeps the four states of a mark visually apart", () => {
    const only = (mark: Mark) => painted("mañana", [mark])[0].look;
    const sure = only(at(0, 6));
    const assumed = only(at(0, 6, "assumed"));
    const offered = only(at(0, 6, "sure", { offered: true }));
    const overruled = only(at(0, 6, "sure", { overruled: true }));

    expect(new Set([sure, assumed, offered, overruled]).size).toBe(4);
    expect(sure).toContain("bg-mark-date");
    expect(assumed).toContain("decoration-dotted");
    expect(offered).toContain("decoration-dashed");
    expect(overruled).toContain("line-through");
  });

  it("stays out of the way when nothing was parsed", () => {
    const { container } = render(
      <Field icon="⌕" value="pan" hint="search" onChange={() => {}} />,
    );
    expect(container.querySelector("[aria-hidden]")).toBeNull();
  });
});
