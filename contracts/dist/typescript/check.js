const WHOLE = {"version":"1.1.0","contracts":{"text":{},"table":{},"code":{},"diagram":{},"formula":{},"attachment":{"fields":{"at":{"type":"whole","required":true},"name":{"type":"file_name","required":true,"at_least":1},"kind":{"type":"line"},"size":{"type":"whole"}}},"picture":{"fields":{"at":{"type":"whole","required":true},"name":{"type":"file_name","required":true,"at_least":1},"kind":{"type":"picture_kind","required":true},"size":{"type":"whole","required":true},"width":{"type":"whole","required":true,"at_least":1,"at_most":1600},"height":{"type":"whole","required":true,"at_least":1,"at_most":1600}}},"widget":{"fields":{"title":{"type":"text"},"body":{"type":"text","required":true}}},"document":{"except":["by","born_by","edited_by","owner","properties"],"fields":{"id":{"type":"id","required":true,"at_most":64},"title":{"type":"line","at_most":8192},"tags":{"type":"list","of":"tag_word","at_most":64},"lang":{"type":"language"},"sheet":{"type":"sheet"},"signed_as":{"type":"name"},"made":{"type":"time"},"wrote":{"type":"time"},"body":{"type":"text","required":true,"bytes_at_most":512000},"html":{"type":"text","required":true},"pages":{"type":"list","of":"page"},"pictures":{"type":"list","of":"picture"},"files":{"type":"list","of":"attachment"},"files_hash":{"type":"hash"},"cited":{"type":"list","of":"cited"},"languages":{"type":"list","of":"translation"}}},"envelope":{"fields":{"contract":{"type":"text","required":true,"at_least":1,"at_most":32},"kind":{"type":"envelope_kind","required":true},"id":{"type":"id","required":true,"at_most":64},"version":{"type":"whole","required":true,"at_least":1},"from":{"type":"whole"},"sealed":{"type":"bool","required":true},"head":{"type":"hash","required_when":"sealed","only_when":"sealed"},"commitment":{"type":"hash","required_when":"sealed","only_when":"sealed"},"parts":{"type":"list","of":"envelope_part","required":true,"at_least":1}}},"sealing":{"fields":{"id":{"type":"id","required":true,"at_most":64},"version":{"type":"whole","required":true,"at_least":1},"from":{"type":"whole"},"link_key":{"type":"key"},"files_hash":{"type":"hash"},"files_key":{"type":"key"}}},"opening":{"fields":{"id":{"type":"id","required":true,"at_most":64},"kind":{"type":"envelope_kind","required":true},"link_key":{"type":"key"}}},"envelope_part":{"fields":{"name":{"type":"part_name","required":true},"compressed":{"type":"bool","required":true},"sealed_size":{"type":"whole","required":true},"open_size":{"type":"whole","required":true}}},"sync":{},"link":{},"front_matter":{"fields":{"title":{"type":"text","at_most":500},"tags":{"type":"list","of":"tag_word","at_most":64},"paper":{"type":"paper"},"sheets":{"type":"sheets_word"},"orientation":{"type":"orientation"},"lang":{"type":"language"},"translationKey":{"type":"text","at_most":500},"task":{"type":"text","at_most":500},"scheduled":{"type":"text","at_most":500},"due":{"type":"text","at_most":500},"remind":{"type":"list","of":"text"},"list":{"type":"text","at_most":500},"priority":{"type":"said_priority"},"repeat":{"type":"text","at_most":500}}},"page":{"except":["pages","pictures","files","files_hash","cited","languages","signed_as","by","born_by","edited_by","owner","properties"],"fields":{"id":{"type":"id","required":true,"at_most":64},"title":{"type":"line","at_most":8192},"tags":{"type":"list","of":"tag_word","at_most":64},"lang":{"type":"language"},"sheet":{"type":"sheet"},"made":{"type":"time"},"wrote":{"type":"time"},"body":{"type":"text","required":true,"bytes_at_most":512000},"html":{"type":"text","required":true},"page_of":{"type":"id","required":true,"at_most":64},"at":{"type":"whole","required":true,"at_least":1}}},"translation":{"fields":{"lang":{"type":"language","required":true},"document":{"type":"id","required":true,"at_most":64},"title":{"type":"line","at_most":8192},"key":{"type":"key"}}},"tag":{"fields":{"name":{"type":"tag_word","required":true}}},"signature":{"fields":{"root":{"type":"key","required":true},"machine":{"type":"key","required":true},"since":{"type":"time","required":true},"certificate":{"type":"signed","required":true},"signed":{"type":"signed","required":true}}},"revocation":{"fields":{"root":{"type":"key","required":true},"machine":{"type":"key","required":true},"at":{"type":"time","required":true},"signed":{"type":"signed","required":true}}},"served":{"fields":{"received":{"type":"time"},"revocations":{"type":"list","of":"revocation"}}},"cited":{"fields":{"document":{"type":"id","required":true,"at_most":64},"key":{"type":"key"},"title":{"type":"line","at_most":8192}}},"sheet":{"fields":{"paper":{"type":"paper","otherwise":"a4"},"on":{"type":"bool"},"orientation":{"type":"orientation","otherwise":"portrait"}}},"style":{},"task":{"fields":{"id":{"type":"text","required":true,"at_least":1},"title":{"type":"text","required":true,"at_least":1},"status":{"type":"status","required":true},"priority":{"type":"priority"},"description":{"type":"text"},"date":{"type":"text"},"deadline":{"type":"text"},"list":{"type":"text"},"tags":{"type":"list","of":"tag_word"},"repeat":{"type":"text"},"part_of":{"type":"text"},"after":{"type":"text"},"source":{"type":"text"},"steps":{"type":"list","of":"step"},"notes":{"type":"list","of":"note"},"reminders":{"type":"list","of":"reminder"}}},"step":{"fields":{"text":{"type":"text","required":true,"at_least":1},"done":{"type":"bool","required":true}}},"note":{"fields":{"at":{"type":"moment","required":true},"body":{"type":"text","required":true,"at_least":1}}},"reminder":{"fields":{"at":{"type":"text","required":true,"at_least":1}}}},"enums":{"picture_kind":["image/png","image/jpeg"],"envelope_kind":["main","files"],"part_name":["first","rest","picture","file"],"sync_every":["off","every5","every10"],"sheets_word":["on"],"said_priority":["do","decide","delegate","minor"],"verdict":["holds","retired","doubtful","refused","unreadable"],"paper":["a4","letter","folio","tabloid"],"orientation":["portrait","landscape"],"status":["open","done","dropped"],"priority":["do","decide","delegate","unset","minor"]},"allowed":{"a":{"attributes":["class","href"],"classes":["card"],"class_starts":[],"class_ends":[]},"blockquote":{"attributes":["class","data-callout"],"classes":["callout"],"class_starts":[],"class_ends":[]},"br":{"attributes":[],"classes":[],"class_starts":[],"class_ends":[]},"circle":{"attributes":["cx","cy","fill","fill-opacity","opacity","r","stroke","stroke-dasharray","stroke-opacity","stroke-width","transform"],"classes":[],"class_starts":[],"class_ends":[]},"clippath":{"attributes":["id"],"classes":[],"class_starts":[],"class_ends":[]},"code":{"attributes":["class"],"classes":["math"],"class_starts":["language-"],"class_ends":[]},"col":{"attributes":["style"],"classes":[],"class_starts":[],"class_ends":[]},"colgroup":{"attributes":[],"classes":[],"class_starts":[],"class_ends":[]},"dd":{"attributes":[],"classes":[],"class_starts":[],"class_ends":[]},"defs":{"attributes":[],"classes":[],"class_starts":[],"class_ends":[]},"details":{"attributes":["open"],"classes":[],"class_starts":[],"class_ends":[]},"div":{"attributes":["class","id"],"classes":["footnote","math","table"],"class_starts":[],"class_ends":[]},"dl":{"attributes":["class"],"classes":["front"],"class_starts":[],"class_ends":[]},"dt":{"attributes":[],"classes":[],"class_starts":[],"class_ends":[]},"ellipse":{"attributes":["cx","cy","fill","fill-opacity","opacity","rx","ry","stroke","stroke-dasharray","stroke-opacity","stroke-width","transform"],"classes":[],"class_starts":[],"class_ends":[]},"em":{"attributes":[],"classes":[],"class_starts":[],"class_ends":[]},"figcaption":{"attributes":[],"classes":[],"class_starts":[],"class_ends":[]},"figure":{"attributes":["class"],"classes":["code","diagram"],"class_starts":[],"class_ends":[]},"g":{"attributes":["clip-path","dominant-baseline","fill","fill-opacity","fill-rule","font-family","font-size","font-style","font-weight","opacity","stroke","stroke-dasharray","stroke-linecap","stroke-linejoin","stroke-opacity","stroke-width","text-anchor","transform"],"classes":[],"class_starts":[],"class_ends":[]},"h1":{"attributes":["id"],"classes":[],"class_starts":[],"class_ends":[]},"h2":{"attributes":["id"],"classes":[],"class_starts":[],"class_ends":[]},"h3":{"attributes":["id"],"classes":[],"class_starts":[],"class_ends":[]},"h4":{"attributes":["id"],"classes":[],"class_starts":[],"class_ends":[]},"h5":{"attributes":["id"],"classes":[],"class_starts":[],"class_ends":[]},"h6":{"attributes":["id"],"classes":[],"class_starts":[],"class_ends":[]},"hr":{"attributes":[],"classes":[],"class_starts":[],"class_ends":[]},"img":{"attributes":["alt","height","loading","src","width"],"classes":[],"class_starts":[],"class_ends":[]},"li":{"attributes":["class"],"classes":["task"],"class_starts":[],"class_ends":[]},"line":{"attributes":["marker-end","marker-mid","marker-start","opacity","stroke","stroke-dasharray","stroke-linecap","stroke-opacity","stroke-width","transform","x1","x2","y1","y2"],"classes":[],"class_starts":[],"class_ends":[]},"lineargradient":{"attributes":["gradientunits","id","x1","x2","y1","y2"],"classes":[],"class_starts":[],"class_ends":[]},"mark":{"attributes":["data-pen"],"classes":[],"class_starts":[],"class_ends":[]},"marker":{"attributes":["id","markerheight","markerunits","markerwidth","orient","refx","refy","viewbox"],"classes":[],"class_starts":[],"class_ends":[]},"math":{"attributes":["display","displaystyle"],"classes":[],"class_starts":[],"class_ends":[]},"menclose":{"attributes":["notation"],"classes":[],"class_starts":[],"class_ends":[]},"merror":{"attributes":["class"],"classes":[],"class_starts":["math-core-"],"class_ends":[]},"mfrac":{"attributes":["displaystyle","linethickness","scriptlevel","style"],"classes":[],"class_starts":[],"class_ends":[]},"mi":{"attributes":["mathvariant","style"],"classes":[],"class_starts":[],"class_ends":[]},"mmultiscripts":{"attributes":[],"classes":[],"class_starts":[],"class_ends":[]},"mn":{"attributes":["mathvariant","style"],"classes":[],"class_starts":[],"class_ends":[]},"mo":{"attributes":["accent","fence","form","largeop","lspace","mathvariant","maxsize","minsize","movablelimits","rspace","separator","stretchy","style","symmetric"],"classes":[],"class_starts":[],"class_ends":[]},"mover":{"attributes":["accent","displaystyle","scriptlevel"],"classes":[],"class_starts":[],"class_ends":[]},"mpadded":{"attributes":["depth","height","lspace","style","voffset","width"],"classes":[],"class_starts":[],"class_ends":[]},"mphantom":{"attributes":[],"classes":[],"class_starts":[],"class_ends":[]},"mprescripts":{"attributes":[],"classes":[],"class_starts":[],"class_ends":[]},"mroot":{"attributes":[],"classes":[],"class_starts":[],"class_ends":[]},"mrow":{"attributes":["class","displaystyle","scriptlevel","style"],"classes":[],"class_starts":["menclose-"],"class_ends":[]},"mspace":{"attributes":["style","width"],"classes":[],"class_starts":[],"class_ends":[]},"msqrt":{"attributes":[],"classes":[],"class_starts":[],"class_ends":[]},"msub":{"attributes":[],"classes":[],"class_starts":[],"class_ends":[]},"msubsup":{"attributes":[],"classes":[],"class_starts":[],"class_ends":[]},"msup":{"attributes":[],"classes":[],"class_starts":[],"class_ends":[]},"mtable":{"attributes":["displaystyle","scriptlevel","style"],"classes":[],"class_starts":[],"class_ends":[]},"mtd":{"attributes":["style"],"classes":[],"class_starts":[],"class_ends":[]},"mtext":{"attributes":["mathvariant","style"],"classes":[],"class_starts":[],"class_ends":[]},"mtr":{"attributes":["style"],"classes":[],"class_starts":[],"class_ends":[]},"munder":{"attributes":["accentunder","displaystyle","scriptlevel"],"classes":[],"class_starts":[],"class_ends":[]},"munderover":{"attributes":["accent","accentunder","displaystyle","scriptlevel"],"classes":[],"class_starts":[],"class_ends":[]},"ol":{"attributes":["start"],"classes":[],"class_starts":[],"class_ends":[]},"p":{"attributes":["class"],"classes":["refdef"],"class_starts":[],"class_ends":[]},"path":{"attributes":["clip-path","d","fill","fill-opacity","fill-rule","marker-end","marker-mid","marker-start","opacity","stroke","stroke-dasharray","stroke-linecap","stroke-linejoin","stroke-opacity","stroke-width","transform"],"classes":[],"class_starts":[],"class_ends":[]},"polygon":{"attributes":["fill","fill-opacity","marker-end","marker-mid","marker-start","opacity","points","stroke","stroke-dasharray","stroke-opacity","stroke-width","transform"],"classes":[],"class_starts":[],"class_ends":[]},"polyline":{"attributes":["fill","marker-end","marker-mid","marker-start","opacity","points","stroke","stroke-dasharray","stroke-linecap","stroke-linejoin","stroke-opacity","stroke-width","transform"],"classes":[],"class_starts":[],"class_ends":[]},"pre":{"attributes":[],"classes":[],"class_starts":[],"class_ends":[]},"rect":{"attributes":["clip-path","fill","fill-opacity","height","opacity","rx","ry","stroke","stroke-dasharray","stroke-opacity","stroke-width","transform","width","x","y"],"classes":[],"class_starts":[],"class_ends":[]},"s":{"attributes":[],"classes":[],"class_starts":[],"class_ends":[]},"span":{"attributes":["class","data-done","data-hue","data-ico","data-tag","style","title"],"classes":["check","fn-label","front-sep","ico","refdef-label","refdef-to","tag"],"class_starts":["hljs-","math-core-"],"class_ends":["_"]},"stop":{"attributes":["offset","stop-color","stop-opacity"],"classes":[],"class_starts":[],"class_ends":[]},"strong":{"attributes":[],"classes":[],"class_starts":[],"class_ends":[]},"summary":{"attributes":[],"classes":[],"class_starts":[],"class_ends":[]},"sup":{"attributes":["class"],"classes":["fnref"],"class_starts":[],"class_ends":[]},"svg":{"attributes":["aria-hidden","aria-label","data-theme","fill","font-family","font-size","font-style","font-weight","height","preserveaspectratio","role","stroke","stroke-linecap","stroke-linejoin","stroke-width","text-anchor","viewbox","width","xmlns"],"classes":[],"class_starts":[],"class_ends":[]},"table":{"attributes":[],"classes":[],"class_starts":[],"class_ends":[]},"tbody":{"attributes":[],"classes":[],"class_starts":[],"class_ends":[]},"td":{"attributes":["class","data-amiss","data-key","style"],"classes":["whole"],"class_starts":[],"class_ends":[]},"text":{"attributes":["clip-path","dominant-baseline","dx","dy","fill","font-family","font-size","font-style","font-weight","opacity","stroke","stroke-width","text-anchor","transform","x","y"],"classes":[],"class_starts":[],"class_ends":[]},"tfoot":{"attributes":[],"classes":[],"class_starts":[],"class_ends":[]},"th":{"attributes":["data-currency","data-total","data-type","scope","style"],"classes":[],"class_starts":[],"class_ends":[]},"thead":{"attributes":[],"classes":[],"class_starts":[],"class_ends":[]},"tr":{"attributes":[],"classes":[],"class_starts":[],"class_ends":[]},"tspan":{"attributes":["dominant-baseline","dx","dy","fill","font-family","font-size","font-style","font-weight","stroke","text-anchor","x","y"],"classes":[],"class_starts":[],"class_ends":[]},"u":{"attributes":[],"classes":[],"class_starts":[],"class_ends":[]},"ul":{"attributes":[],"classes":[],"class_starts":[],"class_ends":[]}},"passthrough":{"tags":["a","abbr","acronym","area","article","aside","b","bdi","bdo","blockquote","br","caption","center","cite","code","col","colgroup","data","dd","del","details","dfn","div","dl","dt","em","figcaption","figure","footer","h1","h2","h3","h4","h5","h6","header","hgroup","hr","i","img","ins","kbd","li","map","mark","nav","ol","p","pre","q","rp","rt","rtc","ruby","s","samp","small","span","strike","strong","sub","summary","sup","table","tbody","td","th","thead","time","tr","tt","u","ul","var","wbr"],"attributes":{"*":["lang","title"],"a":["href","hreflang","rel"],"bdo":["dir"],"blockquote":["cite"],"col":["align","char","charoff","span"],"colgroup":["align","char","charoff","span"],"del":["cite","datetime"],"hr":["align","size","width"],"img":["align","alt","height","src","width"],"ins":["cite","datetime"],"ol":["start"],"q":["cite"],"table":["align","char","charoff","summary"],"tbody":["align","char","charoff"],"td":["align","char","charoff","colspan","headers","rowspan"],"tfoot":["align","char","charoff"],"th":["align","char","charoff","colspan","headers","rowspan","scope"],"thead":["align","char","charoff"],"tr":["align","char","charoff"]}},"urls":{"schemes":["http","https","mailto","tisty"],"attributes":["href","src","cite"],"references":["page","picture","file","cited"],"paints":["fill","stroke","marker-start","marker-end","marker-mid","clip-path"]},"styles":[{"on":"col","property":"width","unit":"px"},{"on":"th","property":"text-align","values":["left","center","right"]},{"on":"td","property":"text-align","values":["left","center","right"]},{"on":"mrow","property":"color","color":true},{"on":"mi","property":"color","color":true},{"on":"mn","property":"color","color":true},{"on":"mo","property":"color","color":true},{"on":"mtext","property":"color","color":true},{"on":"mfrac","property":"color","color":true},{"on":"mrow","property":"font-size","sizes":["em","rem","ex","%","px"]},{"on":"mi","property":"font-size","sizes":["em","rem","ex","%","px"]},{"on":"mn","property":"font-size","sizes":["em","rem","ex","%","px"]},{"on":"mo","property":"font-size","sizes":["em","rem","ex","%","px"]},{"on":"mtext","property":"font-size","sizes":["em","rem","ex","%","px"]},{"on":"mrow","property":"margin-left","sizes":["em","rem","ex","%","px"]},{"on":"mo","property":"margin-left","sizes":["em","rem","ex","%","px"]},{"on":"mspace","property":"margin-left","sizes":["em","rem","ex","%","px"]},{"on":"mpadded","property":"margin-left","sizes":["em","rem","ex","%","px"]},{"on":"mrow","property":"margin-right","sizes":["em","rem","ex","%","px"]},{"on":"mo","property":"margin-right","sizes":["em","rem","ex","%","px"]},{"on":"mspace","property":"margin-right","sizes":["em","rem","ex","%","px"]},{"on":"mpadded","property":"margin-right","sizes":["em","rem","ex","%","px"]},{"on":"mrow","property":"padding-left","sizes":["em","rem","ex","%","px"]},{"on":"mtd","property":"padding-left","sizes":["em","rem","ex","%","px"]},{"on":"mtr","property":"padding-left","sizes":["em","rem","ex","%","px"]},{"on":"mtable","property":"padding-left","sizes":["em","rem","ex","%","px"]},{"on":"mspace","property":"padding-left","sizes":["em","rem","ex","%","px"]},{"on":"mpadded","property":"padding-left","sizes":["em","rem","ex","%","px"]},{"on":"mrow","property":"padding-right","sizes":["em","rem","ex","%","px"]},{"on":"mtd","property":"padding-right","sizes":["em","rem","ex","%","px"]},{"on":"mtr","property":"padding-right","sizes":["em","rem","ex","%","px"]},{"on":"mtable","property":"padding-right","sizes":["em","rem","ex","%","px"]},{"on":"mspace","property":"padding-right","sizes":["em","rem","ex","%","px"]},{"on":"mpadded","property":"padding-right","sizes":["em","rem","ex","%","px"]},{"on":"mrow","property":"padding-top","sizes":["em","rem","ex","%","px"]},{"on":"mtd","property":"padding-top","sizes":["em","rem","ex","%","px"]},{"on":"mtr","property":"padding-top","sizes":["em","rem","ex","%","px"]},{"on":"mtable","property":"padding-top","sizes":["em","rem","ex","%","px"]},{"on":"mspace","property":"padding-top","sizes":["em","rem","ex","%","px"]},{"on":"mpadded","property":"padding-top","sizes":["em","rem","ex","%","px"]},{"on":"mfrac","property":"padding-top","sizes":["em","rem","ex","%","px"]},{"on":"mrow","property":"padding-bottom","sizes":["em","rem","ex","%","px"]},{"on":"mtd","property":"padding-bottom","sizes":["em","rem","ex","%","px"]},{"on":"mtr","property":"padding-bottom","sizes":["em","rem","ex","%","px"]},{"on":"mtable","property":"padding-bottom","sizes":["em","rem","ex","%","px"]},{"on":"mspace","property":"padding-bottom","sizes":["em","rem","ex","%","px"]},{"on":"mpadded","property":"padding-bottom","sizes":["em","rem","ex","%","px"]},{"on":"mfrac","property":"padding-bottom","sizes":["em","rem","ex","%","px"]},{"on":"mtable","property":"width","sizes":["em","rem","ex","%","px"]},{"on":"mtd","property":"width","sizes":["em","rem","ex","%","px"]},{"on":"mspace","property":"width","sizes":["em","rem","ex","%","px"]},{"on":"mtd","property":"text-align","values":["left","right","center"]},{"on":"mtd","property":"justify-items","values":["start","end","center"]},{"on":"mtable","property":"border-top","values":["0.05em solid currentcolor","0.05em dashed currentcolor"]},{"on":"mtr","property":"border-top","values":["0.05em solid currentcolor","0.05em dashed currentcolor"]},{"on":"mtd","property":"border-top","values":["0.05em solid currentcolor","0.05em dashed currentcolor"]},{"on":"mtable","property":"border-bottom","values":["0.05em solid currentcolor","0.05em dashed currentcolor"]},{"on":"mtr","property":"border-bottom","values":["0.05em solid currentcolor","0.05em dashed currentcolor"]},{"on":"mtd","property":"border-bottom","values":["0.05em solid currentcolor","0.05em dashed currentcolor"]},{"on":"mtable","property":"border-left","values":["0.05em solid currentcolor","0.05em dashed currentcolor"]},{"on":"mtr","property":"border-left","values":["0.05em solid currentcolor","0.05em dashed currentcolor"]},{"on":"mtd","property":"border-left","values":["0.05em solid currentcolor","0.05em dashed currentcolor"]},{"on":"mtable","property":"border-right","values":["0.05em solid currentcolor","0.05em dashed currentcolor"]},{"on":"mtr","property":"border-right","values":["0.05em solid currentcolor","0.05em dashed currentcolor"]},{"on":"mtd","property":"border-right","values":["0.05em solid currentcolor","0.05em dashed currentcolor"]},{"on":"mrow","property":"math-shift","values":["compact"]},{"on":"mtable","property":"math-shift","values":["compact"]},{"on":"mtd","property":"math-shift","values":["compact"]},{"on":"span","property":"font-variant-caps","values":["small-caps"]},{"on":"mtext","property":"font-variant-caps","values":["small-caps"]}],"html_rules":[{"tag":"svg","attribute":"data-theme","only":["light","dark"],"says":"A drawing is for one of the two themes."},{"tag":"img","requires":["src","alt","width","height"],"says":"A picture says its size, and its place on the sheet is held."},{"tag":"code","attribute":"class","never":["language-mermaid","language-mmd"],"says":"A diagram travels drawn, as SVG."},{"tag":"img","attribute":"src","starts_with":"tisty:picture/","says":"A picture travels with its document, which asks nobody else for one."}]};
export const VERSION = WHOLE.version;
export const ALLOWED = WHOLE.allowed;
export const PASSTHROUGH = WHOLE.passthrough;
export const STYLES = WHOLE.styles;
export const URLS = WHOLE.urls;

// A tag called constructor must not find what every object inherits.
const own = (held, name) => (held && Object.hasOwn(held, name) ? held[name] : undefined);

const daysIn = (year, month) => {
  const leap = year % 4 === 0 && (year % 100 !== 0 || year % 400 === 0);
  return [31, leap ? 29 : 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31][month - 1];
};

const isTime = (said) => {
  const found = /^(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})Z$/.exec(said);
  if (!found) return false;
  const [year, month, day, hour, minute, second] = found.slice(1).map(Number);
  return month >= 1 && month <= 12 && day >= 1 && day <= daysIn(year, month) && hour <= 23 && minute <= 59 && second <= 59;
};

// A moment nothing signs may hold a fraction of a second, as the log keeps them.
const isMoment = (said) => isTime(said.replace(/\.\d{1,9}Z$/, "Z"));

const UNSEEN = /[\u00AD\u034F\u061C\u115F\u1160\u17B4\u17B5\u180B-\u180F\u200B\u200E\u200F\u2028-\u202E\u2060-\u206F\u2800\u3164\uFEFF\uFFA0\uFFF9-\uFFFB\u{1D173}-\u{1D17A}\u{E0000}-\u{E007F}\p{Cc}]/u;

// Many scripts write with a joiner, and it only hides a letter where no script needs one: between Latin, Greek or Cyrillic.
const needsNoJoiner = (one) => one === undefined || one < 0x0590 || (one >= 0x1e00 && one <= 0x1fff);

const isLine = (said) => {
  const points = [...said].map((one) => one.codePointAt(0));
  const joined = (one, at) => (one !== 0x200c && one !== 0x200d) || (!needsNoJoiner(points[at - 1]) && !needsNoJoiner(points[at + 1]));
  return !UNSEEN.test(said) && points.every(joined);
};

const isFileName = (said) => isLine(said) && !/[/\\]/.test(said);

const isName = (said) => {
  const letters = said.match(/\p{Alphabetic}/gu)?.length ?? 0;
  const digits = said.match(/[0-9]/g)?.length ?? 0;
  return said.trim() === said && [...said].length <= 80 && letters > 0 && digits <= letters && isLine(said);
};

const BASE64URL = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

// The last character holds spare bits, and they are zero: one key has one spelling, so comparing two is comparing them.
const base64urlOf = (said, length, spare) =>
  said.length === length && /^[A-Za-z0-9_-]+$/.test(said) && BASE64URL.indexOf(said[length - 1]) % spare === 0;

const isId = (said) => /^[A-Za-z0-9_-]+$/.test(said);

const spacesOff = (said) => said.replace(/^[\t\n\f\r ]+|[\t\n\f\r ]+$/g, "");

const lengthOf = (said) => [...said].length;

const isLanguage = (said) => /^[a-z]{2,3}(-[A-Z][a-z]{3})?(-([A-Z]{2}|[0-9]{3}))?$/.test(said);

// Accents come off the alphabets that have them, and elsewhere a mark is how the word is written.
const isTagWord = (said) =>
  said === said.toLowerCase() &&
  !/[\u0000-\u052F\u1E00-\u1FFF]\p{M}/u.test(said.normalize("NFD")) &&
  /^[\p{Alphabetic}\p{M}\p{N}]+(-[\p{Alphabetic}\p{M}\p{N}]+)*$/u.test(said);

const checkType = (type, value, at, out) => {
  if (type === "text") {
    if (typeof value !== "string") out.push(at + " is not text");
  } else if (type === "bool") {
    if (typeof value !== "boolean") out.push(at + " is not true or false");
  } else if (type === "whole") {
    if (!Number.isSafeInteger(value) || value < 0) out.push(at + " is not a whole number");
  } else if (type === "id") {
    if (typeof value !== "string" || !isId(value)) out.push(at + " is not an id");
  } else if (type === "time") {
    if (typeof value !== "string" || !isTime(value)) out.push(at + " is not a date in whole seconds");
  } else if (type === "moment") {
    if (typeof value !== "string" || !isMoment(value)) out.push(at + " is not a date");
  } else if (type === "name") {
    if (typeof value !== "string" || !isName(value)) out.push(at + " is not a name a person can read as written");
  } else if (type === "line") {
    if (typeof value !== "string" || !isLine(value)) out.push(at + " is not one line a person can read as written");
  } else if (type === "file_name") {
    if (typeof value !== "string" || !isFileName(value)) out.push(at + " is not the name of a file");
  } else if (type === "language") {
    if (typeof value !== "string" || !isLanguage(value)) out.push(at + " is not a language tag");
  } else if (type === "tag_word") {
    if (typeof value !== "string" || !isTagWord(value)) out.push(at + " is not a tag");
  } else if (type === "key") {
    if (typeof value !== "string" || !base64urlOf(value, 43, 4)) out.push(at + " is not a key");
  } else if (type === "hash") {
    if (typeof value !== "string" || !base64urlOf(value, 43, 4)) out.push(at + " is not a hash");
  } else if (type === "signed") {
    if (typeof value !== "string" || !base64urlOf(value, 86, 16)) out.push(at + " is not a signature");
  } else if (WHOLE.enums[type]) {
    if (!WHOLE.enums[type].includes(value)) out.push(at + " is none of " + WHOLE.enums[type].join(", "));
  } else {
    checkFields(type, value, at, out);
  }
};

function checkFields(name, value, at, out) {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    out.push(at + " is not a " + name);
    return;
  }
  for (const field of WHOLE.contracts[name].except ?? []) {
    if (value[field] !== undefined) out.push((at === "" ? field : at + "." + field) + " is not something a " + name + " has");
  }
  for (const [field, said] of Object.entries(WHOLE.contracts[name].fields ?? {})) {
    const here = at === "" ? field : at + "." + field;
    const given = value[field];
    if (given === undefined || given === null) {
      if (said.required || (said.required_when && value[said.required_when] === true)) out.push(here + " is missing");
      continue;
    }
    if (said.only_when && value[said.only_when] !== true) out.push(here + " is only for what is " + said.only_when);
    // A word this version does not list wears its default, so a later one can add it.
    if (said.otherwise !== undefined && typeof given === "string") continue;
    if (said.type === "list") {
      if (!Array.isArray(given)) {
        out.push(here + " is not a list");
        continue;
      }
      if (said.at_least !== undefined && given.length < said.at_least) out.push(here + " has too few");
      if (said.at_most !== undefined && given.length > said.at_most) out.push(here + " has too many");
      given.forEach((one, n) => checkType(said.of, one, here + "[" + n + "]", out));
      continue;
    }
    checkType(said.type, given, here, out);
    const size = typeof given === "string" ? lengthOf(given) : given;
    if (typeof given === "string" && said.bytes_at_most !== undefined && new TextEncoder().encode(given).length > said.bytes_at_most) {
      out.push(here + " is too long");
    }
    if (typeof size === "number" && (typeof given === "string" || said.type === "whole")) {
      if (said.at_least !== undefined && size < said.at_least) out.push(here + " is too short or too low");
      if (said.at_most !== undefined && size > said.at_most) out.push(here + " is too long or too high");
    }
  }
}

export function check(contract, value) {
  if (!own(WHOLE.contracts, contract)) return ["there is no contract called " + contract];
  const out = [];
  checkFields(contract, value, "", out);
  return out;
}

const lowered = (said) => said.replace(/[A-Z]/g, (letter) => letter.toLowerCase());

const until = (said, stops) => {
  const at = said.search(stops);
  return at < 0 ? said.length : at;
};

const words = (said) => said.split(/[\t\n\f\r ]+/).filter(Boolean);

// Each tag the HTML opens, with its attributes. It walks as check_html does in Rust, so one page gets one verdict.
const opened = (html) => {
  const out = [];
  let rest = html;
  for (let start = rest.indexOf("<"); start >= 0; start = rest.indexOf("<")) {
    rest = rest.slice(start + 1);
    if (rest.startsWith("!") || rest.startsWith("?")) out.push({ tag: "!", attributes: [], unclosed: false });
    // A closing tag says nothing but its name: a quote in one would hide from here what a browser goes on to run.
    if (rest.startsWith("/")) {
      const closes = /^\/([a-zA-Z][^\t\n\f\r />]*)[\t\n\f\r ]*>/.exec(rest);
      out.push({ tag: closes ? "</" + lowered(closes[1]) : "/", attributes: [], unclosed: false });
    }
    if (!/^[a-zA-Z]/.test(rest)) continue;
    let end = until(rest, /[\t\n\f\r />]/);
    const one = { tag: lowered(rest.slice(0, end)), attributes: [], unclosed: false };
    rest = rest.slice(end);
    for (;;) {
      rest = rest.replace(/^[\t\n\f\r /]+/, "");
      if (rest === "") break;
      if (rest[0] === ">") {
        rest = rest.slice(1);
        break;
      }
      end = until(rest, /[\t\n\f\r =/>]/);
      const name = lowered(rest.slice(0, end));
      rest = rest.slice(end).replace(/^[\t\n\f\r ]+/, "");
      if (rest[0] !== "=") {
        one.attributes.push([name, ""]);
        continue;
      }
      rest = rest.slice(1).replace(/^[\t\n\f\r ]+/, "");
      if (rest[0] === '"' || rest[0] === "'") {
        const close = rest.indexOf(rest[0], 1);
        if (close < 0) {
          one.unclosed = true;
          rest = "";
          break;
        }
        one.attributes.push([name, rest.slice(1, close)]);
        rest = rest.slice(close + 1);
      } else {
        end = until(rest, /[\t\n\f\r >]/);
        one.attributes.push([name, rest.slice(0, end)]);
        rest = rest.slice(end);
      }
    }
    out.push(one);
  }
  return out;
};

const brokenBy = (rule, value) =>
  (rule.never ?? []).some((one) => words(value).includes(one)) ||
  (rule.starts_with !== undefined && !value.startsWith(rule.starts_with)) ||
  (rule.only !== undefined && !rule.only.includes(value));

// One spelling for a place too, and it fits a whole number in every language.
const referenceIn = (bare) => {
  const place = bare.slice(bare.indexOf(":") + 1);
  const slash = place.indexOf("/");
  const kind = place.slice(0, Math.max(slash, 0));
  const at = place.slice(slash + 1);
  return slash >= 0 && WHOLE.urls.references.includes(kind) && /^(0|[1-9][0-9]{0,14})$/.test(at) ? { kind, at: Number(at) } : null;
};

// Browsers drop spaces and control characters and decode entities, so a scheme hidden behind either counts as wrong.
// A link says where it goes: nothing without a scheme means anything once it travels, except a heading of its own page.
const wrongUrl = (value) => {
  const bare = [...value].filter((one) => one > " ").join("");
  if (bare.startsWith("#")) return false;
  const colon = bare.indexOf(":");
  if (colon < 0) return true;
  const scheme = lowered(bare.slice(0, colon));
  if (!WHOLE.urls.schemes.includes(scheme)) return true;
  return scheme === "tisty" && referenceIn(bare) === null;
};

// A paint or a marker names something in its own drawing, and nothing else.
const wrongPaint = (value) => {
  const said = spacesOff(lowered(value));
  if (said.includes("&") || said.includes("\\")) return true;
  if (!said.includes("url(")) return false;
  return !/^url\(#[^\t\n\f\r ()]+\)$/.test(said);
};

const isSizeIn = (said, unit) => said.endsWith(unit) && /^\d+(\.\d+)?$/.test(said.slice(0, said.length - unit.length));

const isSized = (said, units) => {
  if (/^-?0*\.?0+$/.test(said)) return true;
  return units.some((unit) => said.endsWith(unit) && /^-?\d*\.?\d+$/.test(said.slice(0, said.length - unit.length)));
};

const isColor = (said) => /^#([0-9a-f]{3}|[0-9a-f]{6}|[0-9a-f]{8})$/.test(said);

const holdsStyle = (rule, said) => {
  if (rule.unit) return isSizeIn(said, rule.unit);
  if (rule.sizes) return isSized(said, rule.sizes);
  if (rule.color) return isColor(said);
  return rule.values.includes(said);
};

const wrongStyles = (tag, value) =>
  value
    .split(";")
    .map(spacesOff)
    .filter(Boolean)
    .filter((one) => {
      const colon = one.indexOf(":");
      const property = spacesOff(lowered(one.slice(0, Math.max(colon, 0))));
      const said = spacesOff(lowered(one.slice(colon + 1)));
      return !WHOLE.styles.some((rule) => rule.on === tag && rule.property === property && holdsStyle(rule, said));
    });

// Each place inside what travels that the HTML leads to, read as a browser reads its attributes.
export function referencesIn(html) {
  const out = [];
  for (const { attributes } of opened(html)) {
    for (const [name, value] of attributes) {
      const bare = [...value].filter((one) => one > " ").join("");
      const one = WHOLE.urls.attributes.includes(name) && lowered(bare).startsWith("tisty:") ? referenceIn(bare) : null;
      if (one) out.push(one);
    }
  }
  return out;
}

// Each picture the HTML draws, with the size it holds for it.
export function picturesIn(html) {
  const out = [];
  for (const { tag, attributes } of opened(html)) {
    if (tag !== "img") continue;
    const said = (name) => attributes.find(([one]) => one === name)?.[1];
    const one = said("src")?.startsWith("tisty:picture/") ? referenceIn(said("src")) : null;
    if (one) out.push({ at: one.at, width: said("width"), height: said("height") });
  }
  return out;
}

export function checkHtml(html) {
  const out = [];
  const through = WHOLE.passthrough.attributes;
  for (const { tag, attributes, unclosed } of opened(html)) {
    if (tag === "!") {
      out.push("a comment is in no contract");
      continue;
    }
    if (tag === "/") {
      out.push("a closing tag that says more than its name is in no contract");
      continue;
    }
    if (tag.startsWith("</")) {
      if (!own(WHOLE.allowed, tag.slice(2)) && !WHOLE.passthrough.tags.includes(tag.slice(2))) out.push(tag + "> is in no contract");
      continue;
    }
    for (const rule of WHOLE.html_rules) {
      if (rule.tag === tag && rule.requires?.some((name) => !attributes.some(([said]) => said === name))) out.push(rule.says);
    }
    const drawn = own(WHOLE.allowed, tag);
    if (!drawn && !WHOLE.passthrough.tags.includes(tag)) {
      out.push("<" + tag + "> is in no contract");
      continue;
    }
    if (unclosed) out.push("<" + tag + "> leaves a quote open");
    for (const [name, value] of attributes) {
      const listed = drawn?.attributes.includes(name) || through["*"].includes(name) || own(through, tag)?.includes(name);
      if (!listed) out.push(name + " on <" + tag + "> is in no contract");
      if (name === "class") {
        for (const one of words(value)) {
          const known =
            !one.includes("&") &&
            (drawn?.classes.includes(one) ||
            drawn?.class_starts.some((start) => one.startsWith(start)) ||
              drawn?.class_ends.some((end) => one.endsWith(end)));
          if (!known) out.push("the class " + one + " on <" + tag + "> is in no contract");
        }
      }
      if (WHOLE.urls.attributes.includes(name) && wrongUrl(value)) out.push(name + " on <" + tag + "> goes where no contract lets it");
      if (WHOLE.urls.paints.includes(name) && wrongPaint(value)) out.push(name + " on <" + tag + "> asks for what is not in its drawing");
      if (name === "style") {
        for (const one of wrongStyles(tag, value)) out.push("the style " + one + " on <" + tag + "> is in no contract");
      }
      for (const rule of WHOLE.html_rules) {
        if (rule.tag === tag && rule.attribute === name && brokenBy(rule, value)) out.push(rule.says);
      }
    }
  }
  return [...new Set(out)];
}
