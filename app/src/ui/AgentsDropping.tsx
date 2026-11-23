import { useEffect, useState } from "react";
import { asksDrop, type DropAnswer, type DropAsked } from "../agentsDropping";
import { DropReview } from "./AgentsReview";

interface Open extends DropAsked {
  done: (answer: DropAnswer) => void;
}

export default function AgentsDropping() {
  const [open, setOpen] = useState<Open | null>(null);
  useEffect(() => {
    // One question at a time: a second waits for the first's answer instead of taking its place.
    let turn: Promise<unknown> = Promise.resolve();
    asksDrop((asked) => {
      const answer = turn.then(
        () => new Promise<DropAnswer>((done) => setOpen({ ...asked, done })),
      );
      turn = answer.catch(() => undefined);
      return answer;
    });
    return () => asksDrop(null);
  }, []);
  if (!open) return null;
  return (
    <DropReview
      said={open.said}
      reach={open.reach}
      onAnswer={(answer) => {
        setOpen(null);
        open.done(answer);
      }}
    />
  );
}
