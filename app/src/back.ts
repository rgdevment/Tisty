const TOGETHER = 1_000;

// A window coming back is seen again and focused, in either order and a moment apart, and one answer serves both.
export const cameBack = (answer: () => void) => {
  let last = 0;
  return () => {
    const now = Date.now();
    if (now - last < TOGETHER) return;
    last = now;
    answer();
  };
};
