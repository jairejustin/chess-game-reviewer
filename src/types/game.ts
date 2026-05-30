export interface AnalyzedMove {
  ply: number;
  san: string;
  fen: string;
  prevBestEval: number;
  playedEval: number;
  bestMoveSan: string;
  classification: MoveBadge;
  principalVariation: string[];
  mateIn: number | null;
}

export type MoveBadge =
  | 'book'
  | 'brilliant'
  | 'great'
  | 'best'
  | 'excellent'
  | 'good'
  | 'inaccuracy'
  | 'mistake'
  | 'blunder'
  | 'miss'
  | 'forced';

export interface MoveCounts {
  brilliant: number;
  great: number;
  best: number;
  excellent: number;
  good: number;
  inaccuracy: number;
  mistake: number;
  blunder: number;
  miss: number;
  book: number;
  forced: number;
}

export interface GameMetadata {
  white: string;
  black: string;
  date: string;
  result: string;
  event?: string;
}

export interface AnalysisSummary {
  whiteAccuracy: number;
  blackAccuracy: number;
  moveCountsWhite: MoveCounts;
  moveCountsBlack: MoveCounts;
  metadata: GameMetadata;
}
