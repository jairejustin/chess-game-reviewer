import type { MoveBadge, MoveCounts } from '../types/game';

export const tallyOrder: MoveBadge[] = [
  'brilliant',
  'great',
  'best',
  'excellent',
  'good',
  'book',
  'forced',
  'inaccuracy',
  'mistake',
  'miss',
  'blunder'
];

export const tallyLabels: Record<MoveBadge, string> = {
  brilliant: 'Brilliant',
  great: 'Great',
  best: 'Best',
  excellent: 'Excellent',
  good: 'Good',
  book: 'Book',
  forced: 'Forced',
  inaccuracy: 'Inaccuracy',
  mistake: 'Mistake',
  miss: 'Miss',
  blunder: 'Blunder'
};

export function formatEval(cp: number, mateIn?: number | null): string {
  if (mateIn != null) {
    if (mateIn === 0 && cp >= 10000) return '+M';
    if (mateIn === 0 && cp <= -10000) return '-M';
    return mateIn > 0 ? `+M${mateIn}` : `-M${Math.abs(mateIn)}`;
  }
  if (cp >= 10000) return '+M';
  if (cp <= -10000) return '-M';
  const abs = Math.abs(cp / 100).toFixed(2);
  return cp >= 0 ? `+${abs}` : `-${abs}`;
}

export function formatAccuracy(score: number): string {
  return score.toFixed(1) + '%';
}

export function activeTallyRows(w: MoveCounts, b: MoveCounts): MoveBadge[] {
  return tallyOrder.filter((c) => (w[c] ?? 0) > 0 || (b[c] ?? 0) > 0);
}

export function getPerspectiveEvalColor(
  cp: number,
  mateIn: number | null | undefined,
  isFlipped: boolean
): string {
  if (mateIn != null && mateIn !== 0) {
    const isWhiteMating = mateIn > 0;
    const isWinning = isFlipped ? !isWhiteMating : isWhiteMating;
    return isWinning ? 'mate-winning' : 'mate-losing';
  }

  if (cp === 0) return 'equal';

  const isWhiteWinning = cp > 0;
  const isWinning = isFlipped ? !isWhiteWinning : isWhiteWinning;
  return isWinning ? 'winning' : 'losing';
}
