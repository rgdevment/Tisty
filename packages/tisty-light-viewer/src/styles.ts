const BLOCKS = [
  ["<blockquote", () => import("tisty-sheet/callouts.css")],
  ['<figure class="code"', () => import("tisty-sheet/code.css")],
  ['<figure class="diagram"', () => import("tisty-sheet/diagrams.css")],
  ["<table", () => import("tisty-sheet/tables.css")],
] as const;

// A block's look comes only with a page that holds one, and its words show even if that look never arrives.
export const styledFor = (pieces: readonly string[]): Promise<unknown> =>
  Promise.all(
    BLOCKS.filter(([mark]) => pieces.some((piece) => piece.includes(mark))).map(([, load]) =>
      load().catch(() => {}),
    ),
  );
