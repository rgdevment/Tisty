// The page names blocks by their bytes in the file; whoever edits holds the text as characters.
export const charsAt = (body: string, bytes: number): number =>
  new TextDecoder().decode(new TextEncoder().encode(body).subarray(0, bytes)).length;

export const spanned = (said: string | null): [number, number] | null => {
  const found = /^(\d+)-(\d+)$/.exec(said ?? "");
  return found ? [Number(found[1]), Number(found[2])] : null;
};
