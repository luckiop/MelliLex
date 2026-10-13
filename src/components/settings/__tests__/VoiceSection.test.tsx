import { screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { VoiceSection } from '../VoiceSection';
import { renderWithProviders } from '../../../test/test-utils';

describe('VoiceSection', () => {
  it('shows what each control does and starts louder and fuller', () => {
    renderWithProviders(<VoiceSection />);

    expect(screen.getByRole('slider', { name: 'Volume' })).toHaveValue('1.7');
    expect(screen.getByText('Higher is louder. Too high turns hard and strained.')).toBeInTheDocument();
    expect(screen.getByRole('slider', { name: 'Speed' })).toHaveValue('0.9');
    expect(screen.getByText('Lower is slower and easier to catch. Higher is quicker and tighter.')).toBeInTheDocument();
    expect(screen.getByRole('slider', { name: 'Tone' })).toHaveValue('5');
    expect(screen.getByText(/Sarah · fuller and clearer/)).toBeInTheDocument();
    expect(screen.getByText(/Right is fuller and more solid/)).toBeInTheDocument();
  });
});
