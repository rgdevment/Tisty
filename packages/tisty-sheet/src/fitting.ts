const LEAST = 40;
const LOOSE = 80;

const wished = (col: HTMLElement): number => {
  col.dataset.wide ??= String(Number.parseFloat(col.style.width) || 0);
  return Number(col.dataset.wide);
};

const roomOf = (box: HTMLElement): number => {
  const style = box.ownerDocument.defaultView?.getComputedStyle(box);
  const padded = (one?: string) => Number.parseFloat(one ?? "") || 0;
  return box.clientWidth - padded(style?.paddingLeft) - padded(style?.paddingRight);
};

const laid = (cols: HTMLElement[], wishes: number[], asked: number, total: number) => {
  for (const [at, col] of cols.entries())
    if (wishes[at])
      col.style.width = `${Math.max(LEAST, Math.floor((wishes[at] * total) / asked))}px`;
};

export const fittedIn = (box: HTMLElement) => {
  const table = box.querySelector<HTMLElement>(":scope > table");
  const cols = Array.from(box.querySelectorAll<HTMLElement>(":scope > table > colgroup > col"));
  const wishes = cols.map(wished);
  const asked = wishes.reduce((all, one) => all + one, 0);
  const dragged = box.ownerDocument.documentElement.classList.contains("widening");
  if (!table || !asked || dragged) return;
  const room = roomOf(box);
  if (room <= 0) return;
  const loose = wishes.filter((one) => !one).length;
  let total = Math.min(asked, room - loose * LOOSE);
  laid(cols, wishes, asked, total);
  // Borders, padding and words that never break are only known once laid, so one look makes up for them.
  const over = table.offsetWidth - room;
  if (over > 0) {
    total = Math.max(0, total - over);
    laid(cols, wishes, asked, total);
  }
  if (total < asked) box.dataset.fit = String(total / asked);
  else delete box.dataset.fit;
};
