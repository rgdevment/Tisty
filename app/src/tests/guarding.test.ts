import { describe, expect, it } from "vitest";
import { composed, inline } from "../markdown";
import { previewOf } from "../previews";

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
