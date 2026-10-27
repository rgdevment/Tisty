import type { Idea, Mind, Shape } from "./mindmap";

/** Where an idea sits: the child to take at each level from the root; the root is `[]`. */
export type Path = readonly number[];

export interface Changed {
  readonly mind: Mind;
  readonly chosen: Path;
}

/** The first idea, reading from the top, that says these words. */
export const pathOfWords = (mind: Mind, text: string): Path | null => {
  const search = (idea: Idea, path: Path): Path | null => {
    if (idea.text === text) return path;
    for (const [at, child] of idea.children.entries()) {
      const found = search(child, [...path, at]);
      if (found) return found;
    }
    return null;
  };
  return search(mind.root, []);
};

export const ideaAt = (mind: Mind, path: Path): Idea | null => {
  let idea: Idea | undefined = mind.root;
  for (const step of path) idea = idea?.children[step];
  return idea ?? null;
};

const fresh = (text: string): Idea => ({ text, shape: "plain", children: [], after: [] });

// Rebuilds only the ideas on the way down, so everything else keeps its lines as read.
const along = (idea: Idea, path: Path, change: (one: Idea) => Idea): Idea => {
  if (!path.length) return change(idea);
  const [step, ...rest] = path;
  const children = idea.children.map((child, at) =>
    at === step ? along(child, rest, change) : child,
  );
  return { ...idea, children };
};

const within = (mind: Mind, path: Path, change: (one: Idea) => Idea): Mind => ({
  ...mind,
  root: along(mind.root, path, change),
});

// A changed idea is written anew; its indent goes too, since the writer gives it its siblings' one.
const touched = (idea: Idea, change: Partial<Idea>): Idea => {
  const { line: _line, indent: _indent, ...rest } = idea;
  return { ...rest, ...change };
};

// Nothing changes, so the map stays the very same object and nothing is written.
const changing = (mind: Mind, path: Path, change: Partial<Idea>, same: (idea: Idea) => boolean) => {
  const idea = ideaAt(mind, path);
  if (!idea || same(idea)) return { mind, chosen: path };
  return { mind: within(mind, path, (one) => touched(one, change)), chosen: path };
};

// An idea with no words would write a line Mermaid cannot read, so emptying one changes nothing.
export const renamed = (mind: Mind, path: Path, text: string): Changed =>
  changing(mind, path, { text }, (idea) => idea.text === text || !text.trim());

export const reshaped = (mind: Mind, path: Path, shape: Shape): Changed =>
  changing(mind, path, { shape }, (idea) => idea.shape === shape);

export const withChild = (mind: Mind, path: Path, text: string): Changed => {
  const parent = ideaAt(mind, path);
  if (!parent) return { mind, chosen: path };
  return {
    mind: within(mind, path, (idea) => ({ ...idea, children: [...idea.children, fresh(text)] })),
    chosen: [...path, parent.children.length],
  };
};

// The root has no siblings, so a sibling of it is a child.
export const withSibling = (mind: Mind, path: Path, text: string): Changed => {
  if (!path.length) return withChild(mind, path, text);
  const parent = path.slice(0, -1);
  const at = path[path.length - 1] + 1;
  return {
    mind: within(mind, parent, (idea) => ({
      ...idea,
      children: [...idea.children.slice(0, at), fresh(text), ...idea.children.slice(at)],
    })),
    chosen: [...parent, at],
  };
};

const cut = (mind: Mind, path: Path): Mind => {
  const parent = path.slice(0, -1);
  const at = path[path.length - 1];
  return within(mind, parent, (idea) => ({
    ...idea,
    children: idea.children.filter((_, one) => one !== at),
  }));
};

// What was chosen next is the idea before it, or its parent when it was the first.
export const without = (mind: Mind, path: Path): Changed => {
  if (!path.length || !ideaAt(mind, path)) return { mind, chosen: path };
  const at = path[path.length - 1];
  const parent = path.slice(0, -1);
  return { mind: cut(mind, path), chosen: at > 0 ? [...parent, at - 1] : parent };
};

const under = (path: Path, other: Path) =>
  other.length >= path.length && path.every((step, at) => other[at] === step);

/** Moves an idea, with all below it, to the end of another; never into itself or its own branch. */
export const moved = (mind: Mind, path: Path, into: Path): Changed | null => {
  const idea = ideaAt(mind, path);
  if (!path.length || !idea || !ideaAt(mind, into) || under(path, into)) return null;
  const parent = path.slice(0, -1);
  const at = path[path.length - 1];
  // Taking it out shifts the later siblings on the way to where it goes.
  const target = into.map((step, level) =>
    level === parent.length && under(parent, into) && step > at ? step - 1 : step,
  );
  const left = cut(mind, path);
  const landing = ideaAt(left, target);
  if (!landing) return null;
  return {
    mind: within(left, target, (one) => ({ ...one, children: [...one.children, idea] })),
    chosen: [...target, landing.children.length],
  };
};
