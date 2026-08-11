import MarkdownIt from "markdown-it";

/** `html: false` escapes raw HTML instead of passing it, so no sanitiser is needed. */
const md = new MarkdownIt({ html: false, linkify: true, breaks: true });

md.inline.ruler.before("link", "wiki", (state, silent) => {
  const { src, pos } = state;
  if (src.charCodeAt(pos) !== 0x5b || src.charCodeAt(pos + 1) !== 0x5b) return false;

  const shut = src.indexOf("]]", pos + 2);
  if (shut < 0) return false;
  const name = src.slice(pos + 2, shut).trim();
  if (!name || name.includes("\n") || name.includes("[")) return false;

  if (!silent) {
    const token = state.push("wiki", "", 0);
    token.content = name;
  }
  state.pos = shut + 2;
  return true;
});

/// `[[#4]]` points at a step of this task, `[[whatever]]` at anything else.
/// A step is a number in the same panel, so it gets its own tint rather than
/// reading as one more document nobody wrote — and it is drawn by its text,
/// because «#4» tells the reader nothing a year later.
md.renderer.rules.wiki = (tokens, i, _options, env) => {
  const said = tokens[i].content;
  const step = /^#(\d{1,3})$/.exec(said);
  if (!step) return `<span class="ref">${md.utils.escapeHtml(said)}</span>`;

  const at = Number(step[1]);
  const steps = (env as { steps?: string[] } | undefined)?.steps ?? [];
  const named = steps[at - 1];
  return `<span class="ref step" data-step="${at}">${md.utils.escapeHtml(named ?? `#${at}`)}</span>`;
};

/** Marks a target that lives under the data root, for the view to resolve. */
export const INSIDE = "data-inside";

/** Ours is what is relative: an absolute path is a file of yours, not of the store. */
const ours = (target: string): boolean =>
  !/^[a-z][a-z0-9+.-]*:/i.test(target) &&
  !/^[a-z]:/i.test(target) &&
  !target.startsWith("/") &&
  !target.startsWith("\\\\");

// A link is never followed in place: inside a webview that would replace the
// app, and there is no back button to return with.
const open = md.renderer.rules.link_open;
md.renderer.rules.link_open = (tokens, i, options, env, self) => {
  const target = String(tokens[i].attrGet("href") ?? "");
  if (ours(target)) tokens[i].attrSet(INSIDE, target);
  return open ? open(tokens, i, options, env, self) : self.renderToken(tokens, i, options);
};

// An image under the data root cannot be loaded by path: the webview resolves
// it against its own origin. The view swaps it for a served URL.
md.renderer.rules.image = (tokens, i, options, env, self) => {
  const target = String(tokens[i].attrGet("src") ?? "");
  tokens[i].attrSet("alt", self.renderInlineAsText(tokens[i].children ?? [], options, env));
  if (ours(target)) {
    tokens[i].attrSet(INSIDE, target);
    tokens[i].attrSet("src", "");
  }
  return self.renderToken(tokens, i, options);
};

/// `steps` resolves `[[#4]]` to what that step actually says. Renumbering the
/// list moves what a reference points at, which is the honest behaviour: the
/// entry says «the fourth thing on that list», and the fourth thing changed.
export const composed = (text: string, steps?: string[]): string =>
  md.render(text, { steps });

export const inline = (text: string, steps?: string[]): string =>
  md.renderInline(text, { steps });
