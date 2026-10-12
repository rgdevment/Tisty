import type { Editor as Writing } from "@tiptap/core";
import { BulletList } from "@tiptap/extension-bullet-list";
import { TaskList } from "@tiptap/extension-task-list";
import type { Node as ProseNode } from "@tiptap/pm/model";
import taskLists from "markdown-it-task-lists";

const tasked = (item: Element): boolean => item.classList.contains("task-list-item");

const apart = (item: Element): boolean => item.getAttribute("data-apart") === "true";

interface Lined {
  type: string;
  hidden?: boolean;
  map?: [number, number] | null;
  attrSet: (name: string, value: string) => void;
}

const PLAIN = new Set([
  "paragraph_open",
  "paragraph_close",
  "inline",
  "list_item_open",
  "list_item_close",
  "bullet_list_open",
  "bullet_list_close",
  "ordered_list_open",
  "ordered_list_close",
]);

const spaced = (md: {
  core: { ruler: { push: (name: string, rule: (state: { tokens: Lined[] }) => void) => void } };
}): void => {
  md.core.ruler.push("spaced", (state) => {
    const lists: { token: Lined; blank: boolean; tight: boolean }[] = [];
    const items: { token: Lined; end: number }[] = [];
    for (const token of state.tokens) {
      const list = lists[lists.length - 1];
      if (token.type.endsWith("_list_open")) {
        lists.push({ token, blank: false, tight: true });
      } else if (token.type.endsWith("_list_close")) {
        const done = lists.pop();
        if (done?.tight) done.token.attrSet("data-tight", "true");
      } else if (token.type === "list_item_open") {
        if (list?.blank) token.attrSet("data-apart", "true");
        items.push({ token, end: token.map ? token.map[0] : 0 });
      } else if (token.type === "list_item_close") {
        const held = items.pop();
        if (!held) continue;
        if (list) list.blank = held.token.map ? held.token.map[1] > held.end : false;
        const up = items[items.length - 1];
        if (up) up.end = Math.max(up.end, held.end);
      } else {
        if (token.type === "paragraph_open" && list && !token.hidden) list.tight = false;
        if (!PLAIN.has(token.type)) for (const one of lists) one.tight = false;
        if (token.map && items.length) {
          const held = items[items.length - 1];
          held.end = Math.max(held.end, token.map[1]);
        }
      }
    }
  });
};

const parted = (list: Element): void => {
  const kids = Array.from(list.childNodes).filter((one) => one.nodeType === 1) as Element[];
  const runs: { task: boolean; items: Element[] }[] = [];
  for (const one of kids) {
    const task = tasked(one);
    const last = runs[runs.length - 1];
    if (last?.task === task) last.items.push(one);
    else runs.push({ task, items: [one] });
  }
  const alone = runs.length < 2 ? runs[0] : null;
  if (alone && (!alone.task || list.tagName === "UL")) {
    if (alone.task) list.setAttribute("data-type", "taskList");
    return;
  }
  let count = Number(list.getAttribute("start") ?? 1) || 1;
  for (const run of runs) {
    const fresh = list.ownerDocument.createElement(run.task ? "ul" : list.tagName);
    if (run.task) fresh.setAttribute("data-type", "taskList");
    else if (count > 1) fresh.setAttribute("start", String(count));
    count += run.items.length;
    if (run.items.every((one, at) => at === 0 || !apart(one))) {
      fresh.setAttribute("data-tight", "true");
    }
    if (apart(run.items[0])) fresh.setAttribute("data-apart", "true");
    for (const one of run.items) fresh.append(one);
    list.before(fresh);
  }
  list.remove();
};

const LISTS = new Set(["bulletList", "taskList"]);

interface Listing {
  flushClose: (size: number) => void;
  renderList: (node: ProseNode, delim: string, first: (index: number) => string) => void;
}

function glued(
  this: { editor: Writing },
  state: Listing,
  node: ProseNode,
  parent: ProseNode | null,
  index: number,
): void {
  const before = parent && index > 0 ? parent.child(index - 1) : null;
  if (
    before &&
    LISTS.has(before.type.name) &&
    before.type.name !== node.type.name &&
    before.attrs.tight !== false &&
    node.attrs.tight !== false &&
    node.attrs.apart !== true
  ) {
    state.flushClose(1);
  }
  const marked = (
    this.editor.storage as unknown as {
      markdown: { options: { bulletListMarker?: string } };
    }
  ).markdown.options.bulletListMarker;
  state.renderList(node, "  ", () => `${marked || "-"} `);
}

const APART = {
  apart: {
    default: false,
    rendered: false,
    parseHTML: (element: HTMLElement) => element.getAttribute("data-apart") === "true",
  },
};

export const Listed = BulletList.extend({
  addAttributes() {
    return { ...this.parent?.(), ...APART };
  },

  addStorage() {
    return { markdown: { serialize: glued } };
  },
});

export const Tightened = TaskList.extend({
  addAttributes() {
    return {
      ...this.parent?.(),
      tight: {
        default: true,
        rendered: false,
        parseHTML: (element: HTMLElement) =>
          element.getAttribute("data-tight") === "true" || !element.querySelector("p"),
      },
      ...APART,
    };
  },

  addStorage() {
    return {
      markdown: {
        serialize: glued,
        parse: {
          setup(md: unknown) {
            (md as { use: (one: unknown) => void }).use(taskLists);
            spaced(md as Parameters<typeof spaced>[0]);
          },
          updateDOM(element: HTMLElement) {
            for (const list of [...element.querySelectorAll(".contains-task-list")]) parted(list);
          },
        },
      },
    };
  },
});
