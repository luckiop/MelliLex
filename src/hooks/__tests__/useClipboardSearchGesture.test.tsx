import { fireEvent, render, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { readText } from '@tauri-apps/plugin-clipboard-manager';
import { toast } from 'sonner';
import { useClipboardSearchGesture } from '../useClipboardSearchGesture';
import { dismissClipboardSearchHint } from '../../services/clipboardSearchGesture';
import { fillSearchBox, readSearchQuery } from '../../services/searchBoxBridge';
import '../../i18n';

vi.mock('@tauri-apps/plugin-clipboard-manager', () => ({
  readText: vi.fn(),
}));

vi.mock('sonner', () => ({
  toast: Object.assign(vi.fn(), {
    dismiss: vi.fn(),
    error: vi.fn(),
    success: vi.fn(),
  }),
}));

function Harness({ onSearch }: { onSearch: (text: string) => void }) {
  useClipboardSearchGesture({ enabled: true, onSearch });
  return (
    <div>
      <button type="button" data-clipboard-cancel="">
        clear
      </button>
      <button type="button">page</button>
      <ol data-sonner-toaster="">
        <button type="button">notice</button>
      </ol>
    </div>
  );
}

function rightClick(target: HTMLElement) {
  fireEvent.pointerDown(target, { button: 2, pointerType: 'mouse' });
}

async function armClipboard(page: HTMLElement) {
  rightClick(page);
  rightClick(page);
  await waitFor(() => expect(readSearchQuery()).toBe('captain'));
}

describe('useClipboardSearchGesture', () => {
  beforeEach(() => {
    localStorage.removeItem('mellilex.clipboardSearchHintDismissed');
    fillSearchBox('previous');
    vi.mocked(readText).mockReset();
    vi.mocked(readText).mockResolvedValue('captain');
    vi.mocked(toast).mockClear();
  });

  it('searches on the next left click and shows an undo notice', async () => {
    const onSearch = vi.fn();
    const view = render(<Harness onSearch={onSearch} />);
    const page = view.getByRole('button', { name: 'page' });

    await armClipboard(page);
    expect(toast).toHaveBeenCalledWith(
      'Clipboard is in the search box',
      expect.objectContaining({
        action: expect.objectContaining({ label: 'Undo' }),
        cancel: expect.objectContaining({ label: "Don't show again" }),
      }),
    );

    fireEvent.pointerDown(page, { button: 0, pointerType: 'mouse' });
    expect(onSearch).toHaveBeenCalledWith('captain');
  });

  it('cancels the armed search when the clear control is clicked', async () => {
    const onSearch = vi.fn();
    const view = render(<Harness onSearch={onSearch} />);
    const page = view.getByRole('button', { name: 'page' });

    await armClipboard(page);
    fireEvent.pointerDown(view.getByRole('button', { name: 'clear' }), {
      button: 0,
      pointerType: 'mouse',
    });
    fireEvent.pointerDown(page, { button: 0, pointerType: 'mouse' });

    expect(onSearch).not.toHaveBeenCalled();
  });

  it('undo restores the previous search text', async () => {
    const onSearch = vi.fn();
    const view = render(<Harness onSearch={onSearch} />);
    await armClipboard(view.getByRole('button', { name: 'page' }));

    const options = vi.mocked(toast).mock.calls[0]?.[1] as {
      action: { onClick: () => void };
    };
    options.action.onClick();

    expect(readSearchQuery()).toBe('previous');
    fireEvent.pointerDown(view.getByRole('button', { name: 'page' }), {
      button: 0,
      pointerType: 'mouse',
    });
    expect(onSearch).not.toHaveBeenCalled();
  });

  it('does not search when the notice itself is clicked', async () => {
    const onSearch = vi.fn();
    const view = render(<Harness onSearch={onSearch} />);
    const page = view.getByRole('button', { name: 'page' });
    await armClipboard(page);

    fireEvent.pointerDown(view.getByRole('button', { name: 'notice' }), {
      button: 0,
      pointerType: 'mouse',
    });
    expect(onSearch).not.toHaveBeenCalled();

    fireEvent.pointerDown(page, { button: 0, pointerType: 'mouse' });
    expect(onSearch).toHaveBeenCalledWith('captain');
  });

  it('skips the notice after it has been closed for good', async () => {
    dismissClipboardSearchHint();
    const onSearch = vi.fn();
    const view = render(<Harness onSearch={onSearch} />);
    await armClipboard(view.getByRole('button', { name: 'page' }));
    expect(toast).not.toHaveBeenCalled();
  });

  it('ignores a clear click that arrives before the clipboard is read', async () => {
    let resolveRead: (value: string) => void = () => undefined;
    vi.mocked(readText).mockReturnValue(
      new Promise((resolve) => {
        resolveRead = resolve;
      }),
    );
    const onSearch = vi.fn();
    const view = render(<Harness onSearch={onSearch} />);
    const page = view.getByRole('button', { name: 'page' });

    rightClick(page);
    rightClick(page);
    fireEvent.pointerDown(view.getByRole('button', { name: 'clear' }), {
      button: 0,
      pointerType: 'mouse',
    });
    resolveRead('captain');
    await Promise.resolve();

    expect(readSearchQuery()).toBe('previous');
    fireEvent.pointerDown(page, { button: 0, pointerType: 'mouse' });
    expect(onSearch).not.toHaveBeenCalled();
  });
});
