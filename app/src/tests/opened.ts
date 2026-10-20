// biome-ignore lint/suspicious/noExplicitAny: each mock types its own arguments.
type Invoke = (cmd: string, ...rest: any[]) => unknown;

// The window opens a document with its print; a mock that answers `doc_read` answers that too.
export const opening =
  (invoke: Invoke) =>
  (cmd: string, ...rest: unknown[]): unknown =>
    cmd === "doc_opened"
      ? Promise.resolve(invoke("doc_read", ...rest)).then((body) => ({
          body,
          print: `print:${body}`,
        }))
      : invoke(cmd, ...rest);
