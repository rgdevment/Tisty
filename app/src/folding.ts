export const folded = (text: string): string =>
  text.normalize("NFD").replace(/\p{M}/gu, "").toLowerCase();
