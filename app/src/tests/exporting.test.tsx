import { describe, expect, it, vi } from "vitest";
import { bared } from "../ui/writing";

vi.mock("@tauri-apps/api/core", () => ({ invoke: () => Promise.resolve(null) }));

describe("copying a document as strict markdown", () => {
  it("gives up the underline rather than leaving a tag behind", () => {
    expect(bared("<u>subrayado</u> y **fuerte**")).toBe("subrayado y **fuerte**");
  });

  it("gives up an underline that carries attributes, closing tag and all", () => {
    expect(bared('<u class="x">hola</u>')).toBe("hola");
  });

  it("leaves alone what a person wrote inside a code block", () => {
    const teaching = "Así se subraya:\n\n```html\n<u>hola</u>\n```";

    expect(bared(teaching)).toBe(teaching);
  });

  it("keeps everything markdown can say, untouched", () => {
    const whole = "# Título\n\n**fuerte** y *suave*\n\n- uno\n- dos\n\n| a | b |\n| --- | --- |\n| 1 | 2 |";

    expect(bared(whole)).toBe(whole);
  });

  it("never eats a less-than sign a person wrote", () => {
    expect(bared("si a < b entonces")).toBe("si a < b entonces");
  });

  it("leaves an ordinary document exactly as it was", () => {
    expect(bared("uno\n\ndos\n\ntres")).toBe("uno\n\ndos\n\ntres");
  });
});
