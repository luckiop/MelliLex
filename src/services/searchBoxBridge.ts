type FillListener = (value: string) => void;

let query = '';
let holdUntil = 0;
const listeners = new Set<FillListener>();

export function isSearchBoxHeld() {
  return Date.now() < holdUntil;
}

export function publishSearchQuery(value: string) {
  query = value;
}

export function readSearchQuery() {
  return query;
}

export function fillSearchBox(value: string) {
  query = value;
  holdUntil = Date.now() + 2000;
  listeners.forEach((listener) => listener(value));
}

export function subscribeSearchBox(listener: FillListener) {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}
