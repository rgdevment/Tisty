// Every size Tisty shows, in the window or on the web, is weighed by the same rule.
export const weighed = (bytes: number): string => {
  if (!Number.isFinite(bytes)) return "—";
  const units = ["B", "kB", "MB", "GB", "TB"];
  let step = 0;
  let left = Math.max(0, bytes);
  while (left >= 1000 && step < units.length - 1) {
    left /= 1000;
    step += 1;
  }
  if (step === 0) return `${Math.round(left)} ${units[step]}`;
  // Rounding to one decimal can reach the next unit, and «1000.0 kB» reads a step behind.
  if (Number(left.toFixed(1)) >= 1000 && step < units.length - 1) {
    left /= 1000;
    step += 1;
  }
  return `${left.toFixed(1)} ${units[step]}`;
};
