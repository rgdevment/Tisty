import MarkdownIt from "markdown-it";

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

md.renderer.rules.wiki = (tokens, i, _options, env) => {
  const said = tokens[i].content;
  const step = /^#(\d{1,3})$/.exec(said);
  if (!step) return `<span class="ref">${md.utils.escapeHtml(said)}</span>`;

  const at = Number(step[1]);
  const steps = (env as { steps?: string[] } | undefined)?.steps ?? [];
  const named = steps[at - 1];
  return `<span class="ref step" data-step="${at}">${md.utils.escapeHtml(named ?? `#${at}`)}</span>`;
};

export const INSIDE = "data-inside";

export const DOC = "tisty:doc/";

export const docLink = (id: string, title: string): string =>
  `[${title.replace(/([[\]])/g, "\\$1")}](${DOC}${id})`;

export const docCard = (id: string, title: string): string => `!${docLink(id, title)}`;

export const docOf = (target: string): string | null =>
  target.startsWith(DOC) ? target.slice(DOC.length) : null;

const ours = (target: string): boolean =>
  !/^[a-z][a-z0-9+.-]*:/i.test(target) &&
  !/^[a-z]:/i.test(target) &&
  !target.startsWith("/") &&
  !target.startsWith("\\\\");

const open = md.renderer.rules.link_open;
md.renderer.rules.link_open = (tokens, i, options, env, self) => {
  const target = String(tokens[i].attrGet("href") ?? "");
  if (ours(target)) tokens[i].attrSet(INSIDE, target);
  if (docOf(target)) tokens[i].attrJoin("class", "paper");
  return open ? open(tokens, i, options, env, self) : self.renderToken(tokens, i, options);
};

md.renderer.rules.image = (tokens, i, options, env, self) => {
  const target = String(tokens[i].attrGet("src") ?? "");
  tokens[i].attrSet("alt", self.renderInlineAsText(tokens[i].children ?? [], options, env));
  if (ours(target) || docOf(target)) {
    tokens[i].attrSet(INSIDE, target);
    tokens[i].attrSet("src", "");
  }
  return self.renderToken(tokens, i, options);
};

export const composed = (text: string, steps?: string[]): string => md.render(text, { steps });

export const inline = (text: string, steps?: string[]): string => md.renderInline(text, { steps });
