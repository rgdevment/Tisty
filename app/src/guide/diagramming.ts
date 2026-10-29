import { diagrams } from "tisty-reader/page";

const root = document.documentElement;
const draw = () => diagrams(document, root.classList.contains("dark"), () => true);

void draw();
new MutationObserver(() => void draw()).observe(root, {
  attributes: true,
  attributeFilter: ["class"],
});
