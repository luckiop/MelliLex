import { describe, expect, it } from 'vitest';
import {
  ARMED_MS,
  dismissClipboardSearchHint,
  DOUBLE_RIGHT_MS,
  initialGestureState,
  isClipboardPromptTarget,
  isClipboardSearchHintDismissed,
  isSearchBoxCancelTarget,
  reduceClipboardGesture,
} from '../clipboardSearchGesture';

describe('clipboard search gesture', () => {
  it('fills on a second right click and ignores a later single right click', () => {
    const first = reduceClipboardGesture(initialGestureState(), { type: 'right', now: 1_000 });
    expect(first.fill).toBe(false);

    const second = reduceClipboardGesture(first.state, {
      type: 'right',
      now: 1_000 + DOUBLE_RIGHT_MS,
    });
    expect(second.fill).toBe(true);
    expect(second.state.suppressContextMenu).toBe(true);

    const menu = reduceClipboardGesture(second.state, { type: 'contextmenu' });
    expect(menu.preventContextMenu).toBe(true);
    expect(menu.state.suppressContextMenu).toBe(false);
  });

  it('does not treat two slow right clicks as a double click', () => {
    const first = reduceClipboardGesture(initialGestureState(), { type: 'right', now: 1_000 });
    const second = reduceClipboardGesture(first.state, {
      type: 'right',
      now: 1_000 + DOUBLE_RIGHT_MS + 1,
    });
    expect(second.fill).toBe(false);
  });

  it('searches on the next left click only after arming', () => {
    const unarmed = reduceClipboardGesture(initialGestureState(), { type: 'left', now: 50 });
    expect(unarmed.search).toBe(false);

    const armed = reduceClipboardGesture(initialGestureState(), { type: 'arm', now: 100 });
    const click = reduceClipboardGesture(armed.state, { type: 'left', now: 150 });
    expect(click.search).toBe(true);

    const again = reduceClipboardGesture(click.state, { type: 'left', now: 160 });
    expect(again.search).toBe(false);
  });

  it('expires the armed left click', () => {
    const armed = reduceClipboardGesture(initialGestureState(), { type: 'arm', now: 100 });
    const late = reduceClipboardGesture(armed.state, {
      type: 'left',
      now: 100 + ARMED_MS + 1,
    });
    expect(late.search).toBe(false);
  });

  it('treats the search clear control as cancel and the notice as the prompt', () => {
    const clear = document.createElement('button');
    clear.dataset.clipboardCancel = '';
    const icon = document.createElement('svg');
    clear.appendChild(icon);
    expect(isSearchBoxCancelTarget(icon)).toBe(true);
    expect(isClipboardPromptTarget(icon)).toBe(false);

    const toaster = document.createElement('ol');
    toaster.dataset.sonnerToaster = '';
    const undo = document.createElement('button');
    toaster.appendChild(undo);
    expect(isClipboardPromptTarget(undo)).toBe(true);
    expect(isSearchBoxCancelTarget(undo)).toBe(false);
  });

  it('remembers when the undo notice is closed for good', () => {
    localStorage.removeItem('mellilex.clipboardSearchHintDismissed');
    expect(isClipboardSearchHintDismissed()).toBe(false);
    dismissClipboardSearchHint();
    expect(isClipboardSearchHintDismissed()).toBe(true);
    localStorage.removeItem('mellilex.clipboardSearchHintDismissed');
  });

  it('escape disarms a pending left click', () => {
    const armed = reduceClipboardGesture(initialGestureState(), { type: 'arm', now: 100 });
    const escaped = reduceClipboardGesture(armed.state, { type: 'escape' });
    const click = reduceClipboardGesture(escaped.state, { type: 'left', now: 200 });
    expect(click.search).toBe(false);
  });
});
