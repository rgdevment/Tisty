import { Editor } from "@tiptap/core";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { written } from "../ui/writing";

const ipc = vi.hoisted(() => ({ calls: [] as string[] }));

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (cmd: string) => {
    ipc.calls.push(cmd);
    return Promise.resolve(cmd === "widget_lend" ? "w1" : null);
  },
  convertFileSrc: (path: string, scheme: string) => `http://${scheme}.localhost/${path}`,
}));

beforeEach(() => {
  ipc.calls = [];
});

const settled = () => new Promise((done) => setTimeout(done, 20));

const drawn = async (content: string, stranger: boolean) => {
  const element = document.createElement("div");
  document.body.append(element);
  const editor = new Editor({ element, extensions: written(stranger), content, editable: false });
  await settled();
  const lent = ipc.calls.filter((one) => one === "widget_lend").length;
  const apart = [...element.querySelectorAll<HTMLButtonElement>(".lit-fold")].some(
    (one) => !one.hidden,
  );
  editor.destroy();
  return { lent, apart };
};

const WIDGETS = [
  "```widget\n<p>x</p>\n```",
  '```widget title="t"\n<p>x</p>\n```',
  "> ```widget\n> <p>x</p>\n> ```",
  "- ```widget\n  <p>x</p>\n  ```",
  '<pre><code class="language-widget">&lt;p&gt;x&lt;/p&gt;</code></pre>',
];

describe("someone else's file", () => {
  it.each(WIDGETS)("never runs a widget, however it is written: %s", async (content) => {
    expect(await drawn(content, true)).toEqual({ lent: 0, apart: false });
  });

  it("still runs the person's own widgets", async () => {
    expect((await drawn(WIDGETS[0], false)).lent).toBe(1);
  });
});
