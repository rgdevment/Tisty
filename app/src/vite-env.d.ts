/// <reference types="vite/client" />

declare module "markdown-it-mark";

declare module "markdown-it-task-lists" {
  const plugin: (md: unknown, options?: Record<string, unknown>) => void;
  export default plugin;
}
