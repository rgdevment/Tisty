import init, { drawn } from "../wasm/tisty_share_draw.js";
import wasm from "../wasm/tisty_share_draw_bg.wasm?url";

// Loaded only for an envelope sealed before Tisty sent the page drawn, and gone once none is left (decision 91).
export const drawnApart = async (text: string): Promise<string> => {
  await init({ module_or_path: wasm });
  return drawn(text);
};
