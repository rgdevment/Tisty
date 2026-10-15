import { render } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import Composed from "../ui/Composed";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: () => Promise.resolve(null),
  convertFileSrc: (at: string) => at,
}));

describe("a note drawn by the core, in the window", () => {
  it("draws its icons and names its callouts, as the page does", () => {
    const view = render(
      <Composed
        html='<p><span class="ico" data-ico="page" data-hue="green"></span> hecho</p><blockquote class="callout" data-callout="tip"><p>ojo</p></blockquote>'
        className="prose"
      />,
    );

    const icon = view.container.querySelector("span.ico");
    expect(icon?.querySelector("svg.glyph")).not.toBeNull();
    expect(icon?.classList.contains("ico-green")).toBe(true);
    expect(view.container.querySelector("blockquote")?.dataset.said).toBeTruthy();
  });
});
