import { useState } from "react";
import Pick from "./Pick";

export default function Glyphs({ onPick }: { onPick: (key: string, hue?: string) => void }) {
  const [hue, setHue] = useState<string>();

  return (
    <Pick
      autoFocus
      keepFocus
      clears={false}
      colour={hue}
      onIcon={(key) => key && onPick(key, hue)}
      onColour={setHue}
    />
  );
}
