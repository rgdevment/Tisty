const SITE = "https://markdown.tisty.app/";

export const LANGUAGES = [
  {
    code: "en",
    file: "en.md",
    dir: "",
    start: "Start",
    topics: "Guide",
    near: "Next and previous",
    home: "https://www.tisty.app/",
    meet: "Meet Tisty",
    name: "English",
    dark: "Light or dark",
    locale: "en_US",
    image: "https://www.tisty.app/tisty-social.png",
  },
  {
    code: "es",
    file: "es.md",
    dir: "es/",
    start: "Inicio",
    topics: "Guía",
    near: "Anterior y siguiente",
    home: "https://www.tisty.app/es/",
    meet: "Conoce Tisty",
    name: "Español",
    dark: "Claro u oscuro",
    locale: "es_ES",
    image: "https://www.tisty.app/tisty-social-es.png",
  },
];

export const escaped = (said) =>
  said.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");

const told = (said) => escaped(said).replace(/"/g, "&quot;");

const address = (language, page) =>
  `${SITE}${language.dir}${page.slug === "index" ? "" : page.slug}`;

// The one script the guide carries: it only remembers light or dark, and every page reads whole without it.
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

// Plain HTML with one local stylesheet and relative links, so the same folder serves the web and the app.
export const sheetOf = (built, language, page) => {
  const { other, twin } = twinOf(built, language, page);
  const pages = built.get(language.code);
  const up = language.dir ? "../" : "";
  const listed = pages
    .map(
      (one) =>
        `<li><a href="${one.slug}.html"${one.slug === page.slug ? ' aria-current="page"' : ""}>${escaped(one.label)}</a></li>`,
    )
    .join("");
  const before = pages[page.at - 1];
  const after = pages[page.at + 1];
  const near = [
    before && `<a rel="prev" href="${before.slug}.html">← ${escaped(before.label)}</a>`,
    after && `<a rel="next" href="${after.slug}.html">${escaped(after.label)} →</a>`,
  ]
    .filter(Boolean)
    .join("");
  const title = page.slug === "index" ? "Tisty Markdown" : `${page.title} · Tisty Markdown`;
  return `<!doctype html>
<html lang="${language.code}">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>${escaped(title)}</title>
${heads(language, page, title, other, twin)}
<link rel="icon" href="${up}favicon.svg" type="image/svg+xml">
${THEME}
<link rel="stylesheet" href="${up}guide.css">
</head>
<body>
<header class="top">
<a class="brand" href="index.html"><img src="${up}favicon.svg" alt="" width="30" height="28">Tisty Markdown</a>
<a class="out" href="${language.home}">${language.meet}</a>
<a class="lang" lang="${other.code}" hreflang="${other.code}" href="${language.dir ? "../" : other.dir}${twin.slug}.html">${other.name}</a>
<button class="theme" type="button" aria-label="${language.dark}" title="${language.dark}" onclick="${SWITCH}">${MOON}${SUN}</button>
</header>
<div class="wrap">
<nav class="topics" aria-label="${language.topics}"><ol>${listed}</ol></nav>
<main>${page.html}<nav class="near" aria-label="${language.near}">${near}</nav></main>
</div>
</body>
</html>
`;
};

// Without its own page, Pages answers a wrong address with the start page and a 200.
export const missing = `<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Not here · Tisty Markdown</title>
<meta name="robots" content="noindex">
<link rel="icon" href="/favicon.svg" type="image/svg+xml">
${THEME}
<link rel="stylesheet" href="/guide.css">
</head>
<body>
<header class="top"><a class="brand" href="/"><img src="/favicon.svg" alt="" width="30" height="28">Tisty Markdown</a><a class="out" href="https://www.tisty.app/">Meet Tisty</a></header>
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

export const robots = `User-agent: *\nAllow: /\n\nSitemap: ${SITE}sitemap.xml\n`;
