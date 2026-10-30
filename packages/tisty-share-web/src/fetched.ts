// What the server answered for a link: the sealed bytes and its limits, or the word of its refusal.
export type Fetched =
  | { readonly sealed: Uint8Array; readonly limits: string }
  | { readonly refused: string };

const API = "/api/v1";

const refusalOf = async (answer: Response): Promise<string> => {
  try {
    const said = (await answer.json()) as { refused?: unknown };
    if (typeof said.refused === "string") return said.refused;
  } catch {
    // A body that is no refusal says no more than its status.
  }
  return answer.status >= 500 ? "down" : "unknown";
};

export const fetched = async (id: string, ask: typeof fetch = fetch): Promise<Fetched> => {
  try {
    const [limits, sealed] = await Promise.all([
      ask(`${API}/limits`),
      ask(`${API}/docs/${encodeURIComponent(id)}`),
    ]);
    if (!sealed.ok) return { refused: await refusalOf(sealed) };
    if (!limits.ok) return { refused: await refusalOf(limits) };
    return { sealed: new Uint8Array(await sealed.arrayBuffer()), limits: await limits.text() };
  } catch {
    return { refused: "offline" };
  }
};
