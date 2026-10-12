import { fireEvent, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { WhatsNewDialog } from '../WhatsNewDialog';
import { renderWithProviders } from '../../test/test-utils';

describe('WhatsNewDialog', () => {
  it('lists the current version notes', () => {
    const onDismiss = vi.fn();
    renderWithProviders(
      <WhatsNewDialog open version="0.12.0" onDismiss={onDismiss} />
    );
    expect(screen.getByText("What's new in v0.12.0")).toBeInTheDocument();
    expect(screen.getByText(/search box to translate/i)).toBeInTheDocument();
    fireEvent.click(screen.getByRole('button', { name: 'Got it' }));
    expect(onDismiss).toHaveBeenCalled();
  });
});
