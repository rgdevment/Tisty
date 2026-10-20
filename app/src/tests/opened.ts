type Invoke = (cmd: string, args?: Record<string, unknown>) => unknown;

// The window opens a document with its print; a mock that answers `doc_read` answers that too.
export const opening =
  (invoke: Invoke) =>
  (cmd: string, args?: Record<string, unknown>): unknown =>
    cmd === "doc_opened"
      ? Promise.resolve(invoke("doc_read", args)).then((body) => ({ body, print: `print:${body}` }))
      : invoke(cmd, args);
