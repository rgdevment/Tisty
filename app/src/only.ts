import { type Dispatch, type SetStateAction, useEffect, useMemo } from "react";
import type { List } from "./core";
import { useGrouped } from "./grouping";
import type { Chosen } from "./views";

export const useOnly = (chosen: Chosen): [Chosen, boolean] => {
  const byList = useGrouped() === "list";
  const seen = useMemo(
    () => (byList && chosen.lists?.length ? { ...chosen, lists: undefined } : chosen),
    [chosen, byList],
  );
  return [seen, byList];
};

export const useOnlyAlive = (
  known: List[] | undefined,
  chosen: Chosen,
  setChosen: Dispatch<SetStateAction<Chosen>>,
) => {
  useEffect(() => {
    const wanted = chosen.lists;
    if (!known || !wanted?.length) return;
    const alive = wanted.filter((id) => known.some((one) => one.id === id));
    if (alive.length === wanted.length) return;
    window.localStorage.setItem("tisty.only", JSON.stringify(alive));
    setChosen((was) => ({ ...was, lists: alive }));
  }, [known, chosen.lists, setChosen]);
};
