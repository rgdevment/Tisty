import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useRef, useState } from "react";
import { type Task, taskOf, type Waking } from "./core";
import type { Chosen } from "./views";

const taskWaiting = (): Promise<string | null> => invoke("task_waiting");
const wakeDue = (): Promise<Waking | null> => invoke("wake_due");
export const wakeAsked = (): Promise<void> => invoke("wake_asked");
export const urgentOffered = (): Promise<boolean> => invoke("urgent_offered");
export const openNoticeSettings = (): Promise<void> => invoke("open_notice_settings");

interface Seeing {
  setChosen: (chosen: Chosen) => void;
  setHeld: (task: Task | undefined) => void;
  setSelected: (id: string | undefined) => void;
}

export const seenFrom = (one: Task): Chosen =>
  one.list ? { named: "lists", list: one.list } : { named: "tasks", slice: "all" };

export function useCalledTasks(seeing: Seeing) {
  const hands = useRef(seeing);
  hands.current = seeing;
  useEffect(() => {
    const see = () =>
      taskWaiting()
        .then((id) => (id ? taskOf(id) : undefined))
        .then((one) => {
          if (!one) return;
          const { setChosen, setHeld, setSelected } = hands.current;
          setChosen(seenFrom(one));
          setHeld(one);
          setSelected(one.id);
        })
        .catch(() => {});
    // Listening first: a reminder pressed while the waiting one is read is still heard.
    const heard = listen("showtask", see);
    heard.then(see, see);
    return () => {
      heard.then((off) => off()).catch(() => {});
    };
  }, []);
}

export function useWakeOffer(looked: unknown) {
  const [waking, setWaking] = useState<Waking | null>(null);
  const putOff = useRef(false);
  useEffect(() => {
    if (putOff.current || looked === undefined) return;
    wakeDue()
      .then((due) => {
        if (due && !putOff.current) setWaking(due);
      })
      .catch(() => {});
  }, [looked]);
  const settleWaking = useCallback(() => {
    putOff.current = true;
    setWaking(null);
  }, []);
  return { waking, settleWaking };
}
