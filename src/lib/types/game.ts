export interface MoveNode {
  ply: number;
  san: string;
  fen: string;
  uci: string;

  source: 'game' | 'variation';

  prevBestEval?: number;
  playedEval?: number;
  bestMoveSan?: string;
  classification?: MoveBadge;
  principalVariation?: string[];
  mateIn?: number | null;
  bestMateIn?: number | null;
}

export interface EngineLine {
  orig: string;
  dest: string;
  rank?: number;
}

export interface AnalysisProgress {
  currentPly: number;
  totalPlies: number;
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
  moves: MoveNode[];
}

export interface PVLine {
  index: number;
  evaluation: string;
  evalCp: number;
  mateIn: number | null;
  sanMoves: string[];
  uciMoves: string[];
}
