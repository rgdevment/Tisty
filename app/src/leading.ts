const PICTURE = "(?:\\p{Extended_Pictographic}|\\p{Regional_Indicator}{2})";
const TRAILS = `(?:\\uFE0F|[\\u{1F3FB}-\\u{1F3FF}]|\\u200D${PICTURE}\\uFE0F?)*`;
const ONLY = new RegExp(`^${PICTURE}${TRAILS}$`, "u");

export const onlyMark = (text: string): boolean => ONLY.test(text.trim());

const LED = new RegExp(`^(${PICTURE}${TRAILS})\\s*(.+)$`, "u");

export interface Led {
  mark: string | null;
  rest: string;
}

// Only if something is left to read once the emoji is taken out.
export const led = (title: string): Led => {
  const found = LED.exec(title.trim());
  if (!found) return { mark: null, rest: title };
  const rest = found[2].trim();
  return rest ? { mark: found[1], rest } : { mark: null, rest: title };
};
