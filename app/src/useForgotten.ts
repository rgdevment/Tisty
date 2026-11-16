import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import type { Filed } from "./core";

export interface Forgotten {
  doc: string;
  last: number;
  lastOpenedHere: boolean;
}

export const docsForgotten = (): Promise<Forgotten[]> => invoke("docs_forgotten");
export const docAtRandom = (): Promise<string | null> => invoke("doc_at_random");

export interface Recalled {
  doc: Filed;
  last: number;
  lastOpenedHere: boolean;
}

// The core names a document by the first of its languages; the Library shows the one the reader reads.
export const standingFor = (docs: readonly Filed[], doc: string): Filed | undefined =>
  docs.find((one) => one.id === doc) ?? docs.find((one) => one.tongue?.[0] === doc);

const inTheOpen = (one: Filed) => !one.away && !one.flagged && !one.pageOf && !one.gone;

export function useForgotten(docs: readonly Filed[], asked: boolean): Recalled[] {
  const [lost, setLost] = useState<Forgotten[]>([]);
  const many = docs.length;
  useEffect(() => {
    if (!asked || many === 0) {
      setLost([]);
      return;
    }
    let current = true;
    docsForgotten()
      .then((found) => current && setLost(found ?? []))
      .catch(() => current && setLost([]));
    return () => {
      current = false;
    };
  }, [asked, many]);
  return lost.flatMap(({ doc, last, lastOpenedHere }) => {
    const standing = standingFor(docs, doc);
    return standing && inTheOpen(standing) ? [{ doc: standing, last, lastOpenedHere }] : [];
  });
}

export async function atRandom(docs: readonly Filed[]): Promise<Filed | undefined> {
  const picked = await docAtRandom();
  return picked ? standingFor(docs, picked) : undefined;
}
