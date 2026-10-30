const SITE = "https://markdown.tisty.app/";

export const LANGUAGES = [
  {
    code: "en",
    file: "en.md",
    dir: "",
    start: "Overview",
    topics: "Guide",
    near: "Next and previous",
    home: "https://www.tisty.app/",
    meet: "Meet Tisty",
    docs: "Docs",
    docsAt: "https://www.tisty.app/docs/",
    before: "Previous",
    after: "Next",
    written: "You write",
    drawn: "Tisty shows",
    name: "EN",
    reading: "Read in English",
    dark: "Light or dark",
    locale: "en_US",
    image: "https://www.tisty.app/tisty-social.png",
  },
  {
    code: "es",
    file: "es.md",
    dir: "es/",
    start: "Vista general",
    topics: "Guía",
    near: "Anterior y siguiente",
    home: "https://www.tisty.app/es/",
    meet: "Conoce Tisty",
    docs: "Ayuda",
    docsAt: "https://www.tisty.app/es/ayuda/",
    before: "Anterior",
    after: "Siguiente",
    written: "Escribes",
    drawn: "Tisty muestra",
    name: "ES",
    reading: "Leer en español",
    dark: "Claro u oscuro",
    locale: "es_ES",
    image: "https://www.tisty.app/tisty-social-es.png",
  },
];

export const escaped = (said) =>
  said.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");

// Pages answers a bare path, and a link ending in .html would cost every click a redirect.
export const href = (slug) => (slug === "index" ? "./" : slug);

const told = (said) => escaped(said).replace(/"/g, "&quot;");

const address = (language, page) =>
  `${SITE}${language.dir}${page.slug === "index" ? "" : page.slug}`;

// It only remembers light or dark; every page reads without it, and only a page with a diagram needs another.
export const THEME = `<script>(()=>{const r=document.documentElement;r.classList.add("js");let s=null;try{s=localStorage.getItem("tisty-guide-theme")}catch{}r.classList.toggle("dark",s?s==="dark":matchMedia("(prefers-color-scheme: dark)").matches)})()</script>`;
const SWITCH = `(()=>{const d=document.documentElement.classList.toggle('dark');try{localStorage.setItem('tisty-guide-theme',d?'dark':'light')}catch{}})()`;
const MOON = `<svg class="moon" viewBox="0 0 24 24" aria-hidden="true"><path d="M20.985 12.486a9 9 0 1 1-9.473-9.472c.405-.022.617.46.402.803a6 6 0 0 0 8.268 8.268c.344-.215.825-.004.803.401"/></svg>`;
const SUN = `<svg class="sun" viewBox="0 0 24 24" aria-hidden="true"><circle cx="12" cy="12" r="4"/><path d="M12 2v2M12 20v2M4.93 4.93l1.41 1.41M17.66 17.66l1.41 1.41M2 12h2M20 12h2M6.34 17.66l-1.41 1.41M19.07 4.93l-1.41 1.41"/></svg>`;

const twinOf = (built, language, page) => {
  const other = LANGUAGES.find((one) => one !== language);
  return { other, twin: built.get(other.code)[page.at] ?? built.get(other.code)[0] };
};

const described = (language, page, title) =>
  JSON.stringify({
    "@context": "https://schema.org",
    "@type": page.slug === "index" ? "WebSite" : "TechArticle",
    ...(page.slug === "index" ? { name: title } : { headline: page.title }),
    description: page.said,
    inLanguage: language.code,
    url: address(language, page),
    isPartOf: { "@type": "WebSite", name: "Tisty Markdown", url: SITE },
    publisher: { "@type": "Organization", name: "Tisty", url: "https://www.tisty.app/" },
  }).replace(/</g, "\\u003c");

const heads = (language, page, title, other, twin) => {
  const here = address(language, page);
  const first = LANGUAGES[0];
  return `<meta name="description" content="${told(page.said)}">
<link rel="canonical" href="${here}">
<link rel="alternate" hreflang="${language.code}" href="${here}">
<link rel="alternate" hreflang="${other.code}" href="${address(other, twin)}">
<link rel="alternate" hreflang="x-default" href="${address(first, language === first ? page : twin)}">
<meta property="og:type" content="${page.slug === "index" ? "website" : "article"}">
<meta property="og:site_name" content="Tisty Markdown">
<meta property="og:title" content="${told(title)}">
<meta property="og:description" content="${told(page.said)}">
<meta property="og:url" content="${here}">
<meta property="og:image" content="${language.image}">
<meta property="og:locale" content="${language.locale}">
<meta property="og:locale:alternate" content="${other.locale}">
<meta name="twitter:card" content="summary_large_image">
<script type="application/ld+json">${described(language, page, title)}</script>`;
};

// Plain HTML with its styles inside and relative links, so the same folder serves the web and the app.
export const sheetOf = (built, language, page, { style, logo }) => {
  const { other, twin } = twinOf(built, language, page);
  const pages = built.get(language.code);
  const up = language.dir ? "../" : "";
  const listed = pages
    .map(
      (one) =>
        `<li><a href="${href(one.slug)}"${one.slug === page.slug ? ' aria-current="page"' : ""}>${escaped(one.label)}</a>${one.slug === page.slug && one.subs.length ? `<ol class="subs">${one.subs.map((sub) => `<li><a href="#${sub.id}">${escaped(sub.title)}</a></li>`).join("")}</ol>` : ""}</li>`,
    )
    .join("");
  const before = pages[page.at - 1];
  const after = pages[page.at + 1];
  const near = [
    before &&
      `<a rel="prev" href="${href(before.slug)}" title="${language.before}"><span aria-hidden="true">←</span>${escaped(before.label)}</a>`,
    after &&
      `<a rel="next" href="${href(after.slug)}" title="${language.after}">${escaped(after.label)}<span aria-hidden="true">→</span></a>`,
  ]
    .filter(Boolean)
    .join("");
  const title = page.slug === "index" ? "Tisty Markdown" : `${page.title} · Tisty Markdown`;
  return `<!doctype html>
<html lang="${language.code}">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<meta name="color-scheme" content="light dark">
<meta name="theme-color" media="(prefers-color-scheme: light)" content="#faf8f4">
<meta name="theme-color" media="(prefers-color-scheme: dark)" content="#0e0d0b">
<title>${escaped(title)}</title>
${heads(language, page, title, other, twin)}
<link rel="icon" href="${up}${logo}" type="image/svg+xml">
${THEME}
<style>${style}</style>
${AHEAD}
</head>
<body>
<header class="top">
<a class="brand" href="./"><img src="${up}${logo}" alt="" width="30" height="30">Tisty Markdown</a>
<a class="docs" href="${language.docsAt}">${language.docs}</a>
<a class="out" href="${language.home}">${language.meet}</a>
<a class="lang" lang="${other.code}" hreflang="${other.code}" title="${other.reading}" href="${language.dir ? "../" : other.dir}${twin.slug === "index" ? "" : twin.slug}">${other.name}</a>
<button class="theme" type="button" aria-label="${language.dark}" title="${language.dark}" onclick="${SWITCH}">${MOON}${SUN}</button>
</header>
<div class="wrap">
<nav class="topics" aria-label="${language.topics}"><div class="search" data-search hidden></div><ol>${listed}</ol></nav>
<main data-pagefind-body>${page.html}<nav class="near" aria-label="${language.near}" data-pagefind-ignore>${near}</nav></main>
</div>${SEARCH}${page.html.includes('class="widget"') ? HOSTS : ""}${page.html.includes('class="diagram"') ? `<script type="module" src="${up}diagrams/diagrams.js"></script>` : ""}
</body>
</html>
`;
};

// Without its own page, Pages answers a wrong address with the start page and a 200.
export const missing = ({ style, logo }) => `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Not here · Tisty Markdown</title>
<meta name="robots" content="noindex">
<link rel="icon" href="/${logo}" type="image/svg+xml">
${THEME}
<style>${style}</style>
</head>
<body>
<header class="top"><a class="brand" href="/"><img src="/${logo}" alt="" width="36" height="36">Tisty Markdown</a><a class="out" href="https://www.tisty.app/">Meet Tisty</a></header>
<div class="wrap"><main class="page"><h1>Nothing here</h1><p>This page is not part of the guide. <a href="/">Start from the beginning</a> · <a href="/es/" lang="es">Empieza desde el inicio</a></p></main></div>
</body>
</html>
`;

export const mapped = (built) => {
  const urls = LANGUAGES.flatMap((language) =>
    built.get(language.code).map((page) => {
      const { other, twin } = twinOf(built, language, page);
      return `<url><loc>${address(language, page)}</loc><xhtml:link rel="alternate" hreflang="${language.code}" href="${address(language, page)}"/><xhtml:link rel="alternate" hreflang="${other.code}" href="${address(other, twin)}"/></url>`;
    }),
  );
  return `<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9" xmlns:xhtml="http://www.w3.org/1999/xhtml">
${urls.join("\n")}
</urlset>
`;
};

// Only where a widget lives: it hears how tall each one is and tells them light or dark.
const HOSTS = `<script>(()=>{const r=document.documentElement;const frames=()=>document.querySelectorAll("iframe.widget");const tell=(w)=>w&&w.postMessage({tisty:"theme",dark:r.classList.contains("dark")},"*");addEventListener("message",(e)=>{if(!e.data||e.data.tisty!=="height")return;for(const f of frames())if(f.contentWindow===e.source){f.style.height=Math.min(Math.max(e.data.h,40),2000)+"px";tell(e.source)}});new MutationObserver(()=>{for(const f of frames())tell(f.contentWindow)}).observe(r,{attributes:true,attributeFilter:["class"]})})()</script>`;

const SEARCH = `<script>(()=>{const box=document.querySelector("[data-search]");if(!box)return;const sheet=document.createElement("link");sheet.rel="stylesheet";sheet.href="/pagefind/pagefind-ui.css";const code=document.createElement("script");code.src="/pagefind/pagefind-ui.js";code.onload=()=>{new PagefindUI({element:box,showSubResults:true,showImages:false,resetStyles:false});box.hidden=false};document.head.append(sheet,code)})()</script>`;

// Declarative, not a script: the browser readies a page while the pointer rests on its link.
const AHEAD = `<script type="speculationrules">{"prerender":[{"where":{"selector_matches":".topics a, .near a, .page a[href]:not([href^='http'])"},"eagerness":"moderate"}]}</script>`;

// The logo's name carries its print, so it can be kept for a year; everything else for a day.
export const headers = (logo) => `/${logo}
  Cache-Control: public, max-age=31536000, immutable
/fonts/*
  Cache-Control: public, max-age=31536000, immutable
/tisty.svg
  Cache-Control: public, max-age=86400
/rina.jpg
  Cache-Control: public, max-age=86400
/es/tisty.svg
  Cache-Control: public, max-age=86400
/es/rina.jpg
  Cache-Control: public, max-age=86400
`;

export const robots = `User-agent: *\nAllow: /\n\nSitemap: ${SITE}sitemap.xml\n`;
