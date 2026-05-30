import type { MoveBadge } from '../types/game';

export const badgeColors: Record<MoveBadge, string> = {
  brilliant: '#1baca6',
  great: '#5c8bb0',
  best: '#95bb4a',
  excellent: '#96bc4b',
  good: '#96ba4b',
  inaccuracy: '#f6b236',
  mistake: '#e28c28',
  blunder: '#ca3431',
  miss: '#ff7769',
  book: '#a88764',
  forced: '#96ba4b'
};

const svgPaths = {
  best: '<polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2" />', // Star
  excellent:
    '<path d="M7 10v12" /><path d="M15 5.88 14 10h5.83a2 2 0 0 1 1.92 2.56l-2.33 8A2 2 0 0 1 17.5 22H4a2 2 0 0 1-2-2v-8a2 2 0 0 1 2-2h2.76a2 2 0 0 0 1.79-1.11L12 2h0a3.13 3.13 0 0 1 3 3.88Z" />', // ThumbsUp
  good: '<polyline points="20 6 9 17 4 12" />', // Check
  miss: '<path d="M18 6 6 18" /><path d="m6 6 12 12" />', // X
  book: '<path d="M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z" /><path d="M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z" />', // BookOpen
  forced: '<path d="M5 12h14" /><path d="m12 5 7 7-7 7" />' // ArrowRight
};

export function getInBoardBadge(classification: MoveBadge): string {
  const color = badgeColors[classification];
  let innerMarkup = '';

  const textBadges = {
    brilliant: '!!',
    great: '!',
    inaccuracy: '?!',
    mistake: '?',
    blunder: '??'
  };

  if (classification in textBadges) {
    innerMarkup = `
      <text x="17" y="19" fill="#fff" font-family="Outfit, sans-serif" font-weight="900" font-size="21" text-anchor="middle" dominant-baseline="middle" letter-spacing="-1">
        ${textBadges[classification as keyof typeof textBadges]}
      </text>`;
  } else {
    const strokeOnly = ['good', 'miss', 'forced'].includes(classification);
    const fill = strokeOnly ? 'none' : '#fff';
    const strokeWidth = strokeOnly ? '4' : '0';

    innerMarkup = `
      <svg x="7" y="7" width="20" height="20" viewBox="0 0 24 24" fill="${fill}" stroke="#fff" stroke-width="${strokeWidth}" stroke-linecap="round" stroke-linejoin="round">
        ${svgPaths[classification as keyof typeof svgPaths]}
      </svg>`;
  }

  return `
    <g transform="translate(70, -5)">
      <g class="badge-anim" style="transform-origin: 17px 17px;">
        <circle cx="17" cy="17" r="17" fill="${color}" stroke="#1e1e1e" stroke-width="1" />
        ${innerMarkup}
      </g>
    </g>
  `;
}
