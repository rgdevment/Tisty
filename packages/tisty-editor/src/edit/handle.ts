import { type EditorView, ViewPlugin, type ViewUpdate } from "@codemirror/view";
import { chooseAt, movedTo, unitAt } from "./blocks";

type Unit = NonNullable<ReturnType<typeof unitAt>>;

const STILL_PX = 4;

class Grip {
  readonly grip: HTMLElement;
  readonly drop: HTMLElement;
  over: Unit | null = null;

  constructor(readonly view: EditorView) {
    const page = view.dom.ownerDocument;
    this.grip = page.createElement("div");
    this.grip.className = "cm-grip";
    this.grip.textContent = "⋮⋮";
    this.grip.hidden = true;
    this.drop = page.createElement("div");
    this.drop.className = "cm-drop";
    this.drop.hidden = true;
    view.dom.append(this.grip, this.drop);
    this.grip.addEventListener("mousedown", (event) => this.pressed(event));
    view.dom.addEventListener("mousemove", this.moved);
    view.dom.addEventListener("mouseleave", this.left);
  }

  // The grip sits in the margin, outside the words, so the whole editor is listened to and not only its text.
  private readonly moved = (event: MouseEvent) => this.hovered(event);
  private readonly left = () => this.hide();

  private top(at: number): number | null {
    const box = this.view.coordsAtPos(at, 1);
    return box ? box.top - this.view.dom.getBoundingClientRect().top : null;
  }

  private placeAt(y: number): number | null {
    const content = this.view.contentDOM.getBoundingClientRect();
    return this.view.posAtCoords({ x: content.left + 4, y }, false);
  }

  hovered(event: MouseEvent) {
    if (event.buttons) return;
    const at = this.placeAt(event.clientY);
    const unit = at === null ? null : unitAt(this.view.state, at, false);
    const top = unit && this.top(unit.from);
    if (!unit || top === null) return this.hide();
    this.over = unit;
    this.grip.style.top = `${top}px`;
    this.grip.hidden = false;
  }

  hide() {
    if (this.grip.matches(":active")) return;
    this.grip.hidden = true;
  }

  private target(y: number): number | null {
    const at = this.placeAt(y);
    const under = at === null ? null : unitAt(this.view.state, at, false);
    if (!under) return null;
    const top = this.view.coordsAtPos(under.from, 1)?.top ?? y;
    const bottom = this.view.coordsAtPos(under.to, -1)?.bottom ?? y;
    return y < (top + bottom) / 2 ? under.from : under.next;
  }

  private pressed(event: MouseEvent) {
    const unit = this.over;
    if (event.button !== 0 || !unit) return;
    event.preventDefault();
    const page = this.view.dom.ownerDocument;
    const startY = event.clientY;
    let target: number | null = null;
    const move = (one: MouseEvent) => {
      if (Math.abs(one.clientY - startY) < STILL_PX && target === null) return;
      target = this.target(one.clientY);
      const top = target === null ? null : this.lineTop(target);
      this.drop.hidden = top === null;
      if (top !== null) this.drop.style.top = `${top}px`;
    };
    const up = () => {
      page.removeEventListener("mousemove", move);
      page.removeEventListener("mouseup", up);
      this.drop.hidden = true;
      this.view.focus();
      const spec = target === null ? null : movedTo(this.view.state, unit, target);
      if (spec) this.view.dispatch(spec);
      else chooseAt(this.view, unit);
    };
    page.addEventListener("mousemove", move);
    page.addEventListener("mouseup", up);
  }

  private lineTop(at: number): number | null {
    if (at < this.view.state.doc.length) return this.top(at);
    const end = this.view.coordsAtPos(at, -1);
    return end ? end.bottom - this.view.dom.getBoundingClientRect().top : null;
  }

  update(update: ViewUpdate) {
    if (!update.docChanged) return;
    this.over = null;
    this.grip.hidden = true;
  }

  destroy() {
    this.view.dom.removeEventListener("mousemove", this.moved);
    this.view.dom.removeEventListener("mouseleave", this.left);
    this.grip.remove();
    this.drop.remove();
  }
}

export const gripping = ViewPlugin.fromClass(Grip);
