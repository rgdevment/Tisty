import { useLayoutEffect, useRef, useState } from "react";
import type { Seat } from "./pageWriter";

type Lands = NonNullable<Seat["current"]>;

export const useSeated = (file: string | undefined, given: Seat | undefined, lands: Lands) => {
  const [own, setOwn] = useState<{ readonly file?: string; readonly seat: Seat }>(() => ({
    file,
    seat: { current: null },
  }));
  if (own.file !== file) setOwn({ file, seat: { current: null } });
  const seat = given ?? own.seat;
  const landing = useRef(lands);
  landing.current = lands;
  const seated = useRef<Seat | null>(null);

  useLayoutEffect(() => {
    const held: Lands = (next) => landing.current(next);
    seated.current = seat;
    seat.current = held;
    return () => {
      seated.current = null;
      if (seat.current === held) seat.current = null;
    };
  }, [seat]);

  return seated;
};
