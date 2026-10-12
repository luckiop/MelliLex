import { useEffect, useRef } from 'react';
import { useTranslation } from 'react-i18next';
import { readText } from '@tauri-apps/plugin-clipboard-manager';
import { toast } from 'sonner';
import {
  ARMED_MS,
  dismissClipboardSearchHint,
  initialGestureState,
  isClipboardPromptTarget,
  isClipboardSearchHintDismissed,
  isSearchBoxCancelTarget,
  reduceClipboardGesture,
  type GestureState,
} from '../services/clipboardSearchGesture';
import { fillSearchBox, readSearchQuery } from '../services/searchBoxBridge';

const CLIPBOARD_TOAST_ID = 'clipboard-search-undo';

interface ClipboardSearchGestureOptions {
  enabled: boolean;
  onSearch: (text: string) => void | Promise<void>;
}

export function useClipboardSearchGesture({ enabled, onSearch }: ClipboardSearchGestureOptions) {
  const { t } = useTranslation();
  const enabledRef = useRef(enabled);
  const onSearchRef = useRef(onSearch);
  const stateRef = useRef<GestureState>(initialGestureState());
  const readInFlightRef = useRef(false);
  const searchQueuedRef = useRef(false);
  const epochRef = useRef(0);

  enabledRef.current = enabled;
  onSearchRef.current = onSearch;

  useEffect(() => {
    if (!enabled) {
      epochRef.current += 1;
      stateRef.current = initialGestureState();
      readInFlightRef.current = false;
      searchQueuedRef.current = false;
      toast.dismiss(CLIPBOARD_TOAST_ID);
    }
  }, [enabled]);

  useEffect(() => {
    const disarm = () => {
      epochRef.current += 1;
      stateRef.current = reduceClipboardGesture(stateRef.current, { type: 'escape' }).state;
      readInFlightRef.current = false;
      searchQueuedRef.current = false;
      toast.dismiss(CLIPBOARD_TOAST_ID);
    };

    const runSearch = () => {
      const text = readSearchQuery().trim();
      if (!text || !enabledRef.current) {
        return;
      }
      toast.dismiss(CLIPBOARD_TOAST_ID);
      void Promise.resolve(onSearchRef.current(text)).catch(() => {
        toast.error(t('settings.clipboardSearch.failed'));
      });
    };

    const showUndoPrompt = (restoreQuery: string) => {
      if (isClipboardSearchHintDismissed()) {
        return;
      }
      toast(t('settings.clipboardSearch.filled'), {
        id: CLIPBOARD_TOAST_ID,
        description: t('settings.clipboardSearch.hint'),
        duration: ARMED_MS,
        action: {
          label: t('settings.clipboardSearch.undo'),
          onClick: () => {
            disarm();
            fillSearchBox(restoreQuery);
          },
        },
        cancel: {
          label: t('settings.clipboardSearch.dismiss'),
          onClick: () => {
            dismissClipboardSearchHint();
          },
        },
      });
    };

    const onPointerDown = (event: PointerEvent) => {
      if (!enabledRef.current || event.pointerType === 'touch') {
        return;
      }
      if (event.ctrlKey || event.metaKey || event.altKey) {
        return;
      }

      if (event.button === 2) {
        const result = reduceClipboardGesture(stateRef.current, {
          type: 'right',
          now: performance.now(),
        });
        stateRef.current = result.state;
        if (!result.fill) {
          return;
        }

        const previousQuery = readSearchQuery();
        const epoch = (epochRef.current += 1);
        readInFlightRef.current = true;
        searchQueuedRef.current = false;
        void readText()
          .then((raw) => {
            if (epoch !== epochRef.current) {
              return;
            }
            const text = raw.trim();
            if (!enabledRef.current || !text) {
              readInFlightRef.current = false;
              searchQueuedRef.current = false;
              stateRef.current = reduceClipboardGesture(stateRef.current, { type: 'escape' }).state;
              if (!text) {
                toast.error(t('settings.clipboardSearch.empty'));
              }
              return;
            }
            fillSearchBox(text);
            stateRef.current = reduceClipboardGesture(stateRef.current, {
              type: 'arm',
              now: performance.now(),
            }).state;
            const queued = searchQueuedRef.current;
            readInFlightRef.current = false;
            searchQueuedRef.current = false;
            if (queued) {
              stateRef.current = reduceClipboardGesture(stateRef.current, {
                type: 'left',
                now: performance.now(),
              }).state;
              runSearch();
              return;
            }
            showUndoPrompt(previousQuery);
          })
          .catch(() => {
            if (epoch !== epochRef.current) {
              return;
            }
            readInFlightRef.current = false;
            searchQueuedRef.current = false;
            stateRef.current = reduceClipboardGesture(stateRef.current, { type: 'escape' }).state;
            toast.error(t('settings.clipboardSearch.failed'));
          });
        return;
      }

      if (event.button !== 0) {
        return;
      }

      if (isClipboardPromptTarget(event.target)) {
        return;
      }

      if (isSearchBoxCancelTarget(event.target)) {
        disarm();
        return;
      }

      if (readInFlightRef.current) {
        searchQueuedRef.current = true;
        event.preventDefault();
        event.stopPropagation();
        return;
      }

      const result = reduceClipboardGesture(stateRef.current, {
        type: 'left',
        now: performance.now(),
      });
      stateRef.current = result.state;
      if (!result.search) {
        return;
      }
      event.preventDefault();
      event.stopPropagation();
      runSearch();
    };

    const onContextMenu = (event: MouseEvent) => {
      if (!enabledRef.current) {
        return;
      }
      const result = reduceClipboardGesture(stateRef.current, { type: 'contextmenu' });
      stateRef.current = result.state;
      if (result.preventContextMenu) {
        event.preventDefault();
        event.stopPropagation();
      }
    };

    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        disarm();
      }
    };

    window.addEventListener('pointerdown', onPointerDown, true);
    window.addEventListener('contextmenu', onContextMenu, true);
    window.addEventListener('keydown', onKeyDown);
    return () => {
      window.removeEventListener('pointerdown', onPointerDown, true);
      window.removeEventListener('contextmenu', onContextMenu, true);
      window.removeEventListener('keydown', onKeyDown);
    };
  }, [t]);
}
