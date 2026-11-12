export const WINKS_ONCE_IN = 100;

const KEPT = "tisty.wink";

interface Due {
  seen: number;
  at: number;
}

const fresh = (): Due => ({ seen: 0, at: 1 + Math.floor(Math.random() * WINKS_ONCE_IN) });

const sound = (due: Partial<Due> | null): due is Due =>
  typeof due?.seen === "number" &&
  typeof due.at === "number" &&
  due.seen >= 0 &&
  due.seen < WINKS_ONCE_IN &&
  due.at >= 1 &&
  due.at <= WINKS_ONCE_IN;

function kept(): Due {
  try {
    const said: Partial<Due> | null = JSON.parse(localStorage.getItem(KEPT) ?? "null");
    return sound(said) ? said : fresh();
  } catch {
    return fresh();
  }
}

// Once in every run of a hundred openings, at a place drawn anew for each run.
export function winksNow(): boolean {
  const due = kept();
  const seen = due.seen + 1;
  try {
    localStorage.setItem(
      KEPT,
      JSON.stringify(seen >= WINKS_ONCE_IN ? fresh() : { seen, at: due.at }),
    );
  } catch {
    return Math.random() * WINKS_ONCE_IN < 1;
  }
  return seen === due.at;
}
