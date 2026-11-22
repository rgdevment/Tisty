type Held = { readonly ctrlKey: boolean; readonly metaKey: boolean };

export const onMac = typeof navigator !== "undefined" && navigator.userAgent.includes("Macintosh");

export const modKey = (event: Held): boolean =>
  onMac ? event.metaKey : event.ctrlKey || event.metaKey;
