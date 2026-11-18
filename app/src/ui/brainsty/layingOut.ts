import type { Asked, Step } from "./settling";

export type Laying = (step: Step) => void;

export const startWorker = (): Worker =>
  new Worker(new URL("./layout.worker.ts", import.meta.url), { type: "module" });

export function layOut(asked: Asked, onStep: Laying, worker: Worker = startWorker()): () => void {
  worker.onmessage = (heard: MessageEvent<Step>) => {
    onStep(heard.data);
    if (heard.data.done) worker.terminate();
  };
  worker.onerror = () => {
    worker.terminate();
    onStep({ x: new Float32Array(), y: new Float32Array(), done: true, iterations: 0, ms: 0 });
  };
  worker.postMessage(asked, [asked.x.buffer, asked.y.buffer, asked.edges.buffer]);
  return () => worker.terminate();
}
