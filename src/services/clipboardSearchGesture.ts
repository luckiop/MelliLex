/** Two right-clicks this close together count as the gesture. */
export const DOUBLE_RIGHT_MS = 500;
/** After the clipboard is in the box, the next left click searches only inside this window. */
export const ARMED_MS = 10_000;

export interface GestureState {
  lastRightAt: number;
  armedUntil: number;
  suppressContextMenu: boolean;
}

export interface GestureResult {
  state: GestureState;
  fill: boolean;
  search: boolean;
  preventContextMenu: boolean;
}

export type GestureEvent =
  | { type: 'right'; now: number }
  | { type: 'left'; now: number }
  | { type: 'arm'; now: number }
  | { type: 'escape' }
  | { type: 'contextmenu' };

export const initialGestureState = (): GestureState => ({
  lastRightAt: 0,
  armedUntil: 0,
  suppressContextMenu: false,
});

const idle = (state: GestureState): GestureResult => ({
  state,
  fill: false,
  search: false,
  preventContextMenu: false,
});

export function reduceClipboardGesture(state: GestureState, event: GestureEvent): GestureResult {
  switch (event.type) {
    case 'right': {
      const isDouble =
        state.lastRightAt > 0 && event.now - state.lastRightAt <= DOUBLE_RIGHT_MS;
      if (!isDouble) {
        return idle({ ...state, lastRightAt: event.now });
      }
      return {
        ...idle(state),
        fill: true,
        state: { ...state, lastRightAt: 0, suppressContextMenu: true },
      };
    }
    case 'arm':
      return idle({
        ...state,
        lastRightAt: 0,
        armedUntil: event.now + ARMED_MS,
      });
    case 'left': {
      if (state.armedUntil > 0 && event.now <= state.armedUntil) {
        return {
          ...idle(state),
          search: true,
          state: { ...state, armedUntil: 0 },
        };
      }
      return idle(state);
    }
    case 'escape':
      return idle({ ...state, armedUntil: 0, suppressContextMenu: false });
    case 'contextmenu':
      if (!state.suppressContextMenu) {
        return idle(state);
      }
      return {
        ...idle(state),
        preventContextMenu: true,
        state: { ...state, suppressContextMenu: false },
      };
    default:
      return idle(state);
  }
}

const HINT_DISMISS_KEY = 'mellilex.clipboardSearchHintDismissed';

export function isSearchBoxCancelTarget(target: EventTarget | null): boolean {
  return target instanceof Element && target.closest('[data-clipboard-cancel]') !== null;
}

export function isClipboardPromptTarget(target: EventTarget | null): boolean {
  return target instanceof Element && target.closest('[data-sonner-toaster]') !== null;
}

export function isClipboardSearchHintDismissed(): boolean {
  try {
    return localStorage.getItem(HINT_DISMISS_KEY) === '1';
  } catch {
    return false;
  }
}

export function dismissClipboardSearchHint(): void {
  try {
    localStorage.setItem(HINT_DISMISS_KEY, '1');
  } catch {
    // The notice can show again if storage is unavailable.
  }
}
