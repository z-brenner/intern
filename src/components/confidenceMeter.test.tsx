import { render } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { ConfidenceMeter } from './ConfidenceMeter';
import type { QueueStatus } from '../types';

const meter = (value: number | undefined, status: QueueStatus) => {
  const { container } = render(<ConfidenceMeter value={value} status={status} variant="panel" />);
  return container;
};

describe('the confidence meter', () => {
  // The meter used to take its color from whatever it was put in, and showed
  // the model's own 100% for a document in review. Its color is now its
  // row's status, carried by the meter itself, whatever the figure says.
  it('is green for a ready or renamed row and amber for one in review, whatever the figure', () => {
    for (const [status, tone] of [['ready', 'ready'], ['completed', 'ready'], ['review', 'review']] as const) {
      for (const value of [0.12, 0.45, 1]) {
        const shown = meter(value, status).querySelector('.confidence-meter');
        expect(shown, `${status} ${value}`).toHaveClass('confidence', tone);
        expect(shown, `${status} ${value}`).not.toHaveClass(tone === 'ready' ? 'review' : 'ready');
      }
    }
  });

  it('takes no tone of its own for a row that is neither ready nor in review', () => {
    for (const status of ['waiting', 'processing', 'failed'] as const) {
      const shown = meter(0.8, status).querySelector('.confidence-meter');
      expect(shown, status).not.toHaveClass('confidence');
      expect(shown, status).not.toHaveClass('ready');
      expect(shown, status).not.toHaveClass('review');
    }
  });

  it('shows the figure and a track as long as it, and a dash where there is none', () => {
    const shown = meter(0.45, 'review');
    expect(shown).toHaveTextContent('45%');
    expect(shown.querySelector('.meter-fill')).toHaveStyle({ width: '45%' });
    expect(meter(undefined, 'review')).toHaveTextContent('—');
    expect(meter(Number.NaN, 'ready')).toHaveTextContent('—');
    expect(meter(undefined, 'review').querySelector('.meter')).toBeNull();
  });
});
