import { useEffect, useRef, useState } from "react";
import { laidIn, valuesIn } from "../front";
import { factsCard, type Told } from "./printing";

// Only what the card says asks for it again, not every key pressed in the page under it.
export default function useCard(file: string | undefined, body: string, kept = 0) {
  const [card, setCard] = useState<readonly Told[] | undefined>();
  const now = useRef(body);
  now.current = body;
  const carded = laidIn(body).carded;
  const said = JSON.stringify(["author", "version", "date"].map((key) => valuesIn(body, key)));

  useEffect(() => {
    if (!file || !carded) {
      setCard(undefined);
      return;
    }
    let live = true;
    factsCard(file, now.current)
      .then((told) => {
        if (live) setCard(told);
      })
      .catch(() => {});
    return () => {
      live = false;
    };
  }, [file, carded, said, kept]);

  return card;
}
