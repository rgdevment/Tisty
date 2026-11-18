import { type Asked, type Step, settle } from "./settling";

self.onmessage = (asked: MessageEvent<Asked>) => {
  settle(asked.data, (step: Step) =>
    self.postMessage(step, {
      transfer: [step.x.buffer as ArrayBuffer, step.y.buffer as ArrayBuffer],
    }),
  );
};
