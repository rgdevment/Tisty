import { describe, expect, it } from "vitest";
import { dressed } from "../markdown";
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

describe("lo que el núcleo dibuja y se pinta con innerHTML", () => {
  it("convierte en ficha la imagen que el html crudo dejó, sin pedirla afuera", () => {
    const html = dressed('<div data-at="0-30"><img src="x"></div>');

    expect(html).toContain('data-inside="x"');
    expect(html).toContain('src=""');
  });

  it("no marca como interno un src de otro esquema", () => {
    const html = dressed('<p><img src="https://evil.example/p.png" alt="x"></p>');

    expect(html).toContain("https://evil.example/p.png");
    expect(html).not.toContain("data-inside");
  });
});
