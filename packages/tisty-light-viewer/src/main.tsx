import { render } from "@solidjs/web";
import "tisty-sheet/tokens.css";
import "tisty-sheet/page.css";
import "tisty-sheet/tables.css";
import "./frame.css";
import App from "./App";
import { closed } from "./host";

const asked = new URLSearchParams(location.search).get("locale");
document.documentElement.lang = asked === "es" ? "es" : "en";

document.addEventListener("contextmenu", (e) => {
  if (!(e.target as HTMLElement).closest("input")) e.preventDefault();
});

render(
  () => <App asked={asked} close={() => void closed()} />,
  document.getElementById("root") as HTMLElement,
);
