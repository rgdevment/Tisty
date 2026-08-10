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

md.renderer.rules.wiki = (tokens, i) =>
  `<span class="ref">${md.utils.escapeHtml(tokens[i].content)}</span>`;

/** Marks a target that lives under the data root, for the view to resolve. */
export const INSIDE = "data-inside";

/** Anything without a scheme is ours: attachments and documents are relative. */
const ours = (target: string): boolean =>
  !/^[a-z][a-z0-9+.-]{1,}:/i.test(target) && !target.startsWith("//") && !target.startsWith("/");

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

export const composed = (text: string): string => md.render(text);

export const inline = (text: string): string => md.renderInline(text);
