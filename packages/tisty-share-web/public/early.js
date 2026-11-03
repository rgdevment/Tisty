// Asked for before the page's code arrives, so the envelope travels while the code does; the code takes it once.
(() => {
  const id = /^\/([0-9A-Za-z]{8})\/?$/.exec(location.pathname)?.[1];
  if (!id || location.hash.length < 2 || "tistyEarly" in globalThis) return;
  globalThis.tistyEarly = { id, answer: fetch(`/api/v1/docs/${id}`) };
})();
