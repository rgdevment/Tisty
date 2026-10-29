import { describe, expect, it, vi } from "vitest";
import { colored, coloredAll } from "../src/coloring";

const page = (html: string): Document =>
  new DOMParser().parseFromString(`<article class="page">${html}</article>`, "text/html");

const code = (tongue: string, said: string) =>
  `<figure class="code"><pre><code class="language-${tongue}">${said}</code></pre></figure>`;

describe("colouring code on the page", () => {
  it("colours a language it knows and leaves the rest as written", () => {
    const drawn = page(`${code("ts", 'const a = "b";')}${code("nope", "x")}`);
    coloredAll(drawn);
    const [known, odd] = Array.from(drawn.querySelectorAll("code"));
    expect(known.querySelector(".hljs-keyword")?.textContent).toBe("const");
    expect(known.textContent).toBe('const a = "b";');
    expect(odd.children).toHaveLength(0);
  });

  it("leaves code with no language and code outside a figure alone", () => {
    const drawn = page(
      '<figure class="code"><pre><code>const a = 1;</code></pre></figure><pre><code class="language-ts">const b = 2;</code></pre>',
    );
    coloredAll(drawn);
    for (const one of drawn.querySelectorAll("code")) expect(one.children).toHaveLength(0);
  });

  it("leaves a block too long to colour as written", () => {
    const drawn = page(code("ts", `const a = 1;${" ".repeat(64 * 1024)}`));
    coloredAll(drawn);
    expect(drawn.querySelector("code")?.children).toHaveLength(0);
  });

  it("colours a little at a time, and stops once the page is gone", () => {
    const frames: FrameRequestCallback[] = [];
    vi.spyOn(globalThis, "requestAnimationFrame").mockImplementation((then) => {
      frames.push(then);
      return frames.length;
    });
    let now = 0;
    vi.spyOn(performance, "now").mockImplementation(() => {
      now += 10;
      return now;
    });
    const drawn = page(`${code("ts", "const a = 1;")}${code("ts", "const b = 2;")}`);
    let live = true;
    colored(drawn, () => live);
    const [first, second] = Array.from(drawn.querySelectorAll("code"));
    expect(first.querySelector(".hljs-keyword")).not.toBeNull();
    expect(second.children).toHaveLength(0);
    expect(frames).toHaveLength(1);
    live = false;
    frames[0](0);
    expect(second.children).toHaveLength(0);
    vi.restoreAllMocks();
  });
});
