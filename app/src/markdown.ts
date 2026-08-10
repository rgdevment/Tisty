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

// Every link leaves for the browser; inside the webview it would replace the app.
const open = md.renderer.rules.link_open;
md.renderer.rules.link_open = (tokens, i, options, env, self) => {
  tokens[i].attrSet("target", "_blank");
  tokens[i].attrSet("rel", "noreferrer");
  return open ? open(tokens, i, options, env, self) : self.renderToken(tokens, i, options);
};

export const composed = (text: string): string => md.render(text);

export const inline = (text: string): string => md.renderInline(text);
