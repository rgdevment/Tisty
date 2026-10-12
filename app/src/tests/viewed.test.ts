import { describe, expect, it } from "vitest";
import { disarmed } from "../viewed";

describe("a file from outside", () => {
  it("shows its widgets as code, titled or not, with either fence", () => {
    const body = '```widget\n<p>a</p>\n```\n\n~~~~ widget title="x"\n<p>b</p>\n~~~~\n';
    expect(disarmed(body)).toBe('```html\n<p>a</p>\n```\n\n~~~~ html title="x"\n<p>b</p>\n~~~~\n');
  });

  it("leaves the word alone anywhere but a fence that opens a widget", () => {
    const body = "a widget here\n```widgets\nx\n```\n```js\nwidget\n```\n";
    expect(disarmed(body)).toBe(body);
  });
});
