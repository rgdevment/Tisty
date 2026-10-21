import { describe, expect, it } from "vitest";
import { dressed } from "../markdown";

describe("a note the core drew, readied for the window, as a whole", () => {
  it("readies a page with cards, links, pictures, steps and a bare address", () => {
    const html = [
      '<h1 id="plan" data-at="0-7">Plan</h1>',
      '<p data-at="8-60">See <a class="card" data-doc="01JABC" href="tisty:doc/01JABC">Notes</a> and',
      ' <a class="card" data-task="01JTASK" href="tisty:task/01JTASK">Pay rent</a>.</p>',
      '<p><a href="https://example.org/a">site</a>, <a href="#">refused</a>, <a href="tisty:doc/01JXYZ">doc</a>,',
      ' <a href="notes/other.md">relative</a> and https://example.org/b? bare.</p>',
      '<p><img src="attachments/ab/cd.png" alt="shot"> <img src="https://example.org/p.png" alt="far"></p>',
      '<p>Do <span class="ref" data-ref="#1">#1</span> then <span class="ref" data-ref="#9">#9</span>.</p>',
      "<pre><code>https://example.org/in-code</code></pre>",
    ].join("");
    expect(dressed(html, ["Buy flour", "Bake"])).toMatchSnapshot();
  });
});
