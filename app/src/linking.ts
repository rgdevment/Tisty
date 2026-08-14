export const addressed = (raw: string): string | null => {
  const target = raw.trim();
  if (!target || /\s/.test(target)) return null;
  const spelt = /:\/\//.test(target) || /^(mailto|tel):/i.test(target);
  return spelt ? target : `https://${target}`;
};
