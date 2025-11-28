// Helpers for resolving and updating nested renderer state via hierarchy-provided paths.
import { clone } from "./form";

export type StatePath = (string | number)[];

export function getValueAtPath<T>(state: T, path: StatePath): unknown {
  if (!path.length) {
    return state;
  }

  let current: unknown = state;
  for (const segment of path) {
    if (current == null) {
      return undefined;
    }

    if (typeof segment === "number" && Array.isArray(current)) {
      current = current[segment];
    } else if (typeof segment === "string" && typeof current === "object") {
      current = (current as Record<string, unknown>)[segment];
    } else {
      return undefined;
    }
  }

  return current;
}

export function setValueAtPath<T>(state: T, path: StatePath, value: unknown): T {
  if (!path.length) {
    return value as T;
  }

  const nextState = clone(state);
  let cursor: any = nextState;

  for (let index = 0; index < path.length; index += 1) {
    const segment = path[index];
    const isLeaf = index === path.length - 1;

    if (isLeaf) {
      cursor[segment as any] = value;
      break;
    }

    const nextSegment = path[index + 1];
    let nextValue = cursor[segment as any];

    if (nextValue === undefined) {
      nextValue = typeof nextSegment === "number" ? [] : {};
      cursor[segment as any] = nextValue;
    } else if (Array.isArray(nextValue)) {
      nextValue = [...nextValue];
      cursor[segment as any] = nextValue;
    } else if (typeof nextValue === "object" && nextValue !== null) {
      nextValue = { ...nextValue };
      cursor[segment as any] = nextValue;
    }

    cursor = cursor[segment as any];
  }

  return nextState;
}
