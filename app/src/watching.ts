import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef, useState } from "react";
import { type Ready, type Underway, updateReady } from "./core";
import { noticeBehind } from "./refusal";

const LOOKS_AGAIN = 6 * 60 * 60 * 1000;
const TIGHT = 1308;

export function useUpdates() {
  const [ready, setReady] = useState<Ready | null>(null);
  const [underway, setUnderway] = useState<Underway | null>(null);
  const [behind, setBehind] = useState(false);

  const looked = useRef(false);

  const going = useRef(false);
  useEffect(() => {
    going.current = !!underway;
  }, [underway]);

  useEffect(() => {
    // An update under way owns what About shows; looking again would pull the offer from under it.
    if (going.current) return;
    const first = !looked.current;
    looked.current = true;
    updateReady(behind || first)
      .then(setReady)
      .catch(() => {});
  }, [behind]);

  useEffect(() => {
    const again = setInterval(() => {
      if (going.current) return;
      updateReady()
        .then(setReady)
        .catch(() => {});
    }, LOOKS_AGAIN);
    return () => clearInterval(again);
  }, []);

  useEffect(() => {
    noticeBehind(setBehind);
    return () => noticeBehind(null);
  }, []);

  useEffect(() => {
    const off = listen<Underway>("updating", (said) => setUnderway(said.payload));
    return () => {
      void off.then((stop) => stop());
    };
  }, []);

  const lookAgain = () =>
    updateReady(true)
      .then(setReady)
      .catch(() => {});

  return { ready, lookAgain, underway, setUnderway, behind, setBehind };
}

export function useTight() {
  const [tight, setTight] = useState(() => window.innerWidth < TIGHT);
  useEffect(() => {
    const look = () => setTight(window.innerWidth < TIGHT);
    window.addEventListener("resize", look);
    return () => window.removeEventListener("resize", look);
  }, []);
  return tight;
}

export function useAloud() {
  const [aloud, setAloud] = useState("");
  const twice = useRef(0);
  const say = (words: string) => {
    twice.current += 1;
    setAloud(words + "\u200b".repeat(twice.current % 2));
  };
  return { aloud, say };
}
