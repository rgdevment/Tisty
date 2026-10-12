// A file from outside is someone else's word, so its widgets show as code until it is kept.
export const disarmed = (body: string): string =>
  body.replace(/^([ \t]*(?:`{3,}|~{3,})[ \t]*)widget(?=[ \t]|$)/gim, "$1html");
