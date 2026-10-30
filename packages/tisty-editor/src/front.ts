export const frontLines = (body: string): string[] | null => {
  const lines = body.replace(/^﻿/, "").split("\n");
  if (lines[0]?.trimEnd() !== "---" || !lines[1]?.trim()) return null;
  const shut = lines.findIndex((line, at) => at > 1 && /^(---|\.\.\.)$/.test(line.trimEnd()));
  return shut < 0 ? null : lines.slice(0, shut + 1);
};
