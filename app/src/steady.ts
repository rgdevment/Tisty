// Every refresh weighs what the window already holds, so each answer is spelled out only once.
const spelled = new WeakMap<object, string>();

const spelledOut = (one: unknown): string => {
  if (typeof one !== "object" || one === null) return JSON.stringify(one);
  const known = spelled.get(one);
  if (known !== undefined) return known;
  const said = JSON.stringify(one);
  spelled.set(one, said);
  return said;
};

export const steady = <T>(was: T, found: T): T =>
  was === found || spelledOut(was) === spelledOut(found) ? was : found;
