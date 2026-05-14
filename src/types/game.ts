export interface AnalyzedMove {
  ply: number;
  san: string;
  fen: string;
  bestMoveSan: string;
  playedEval: number;
  bestMoveEval: number;

  classification: MoveBadge;
  principalVariation: string[];
}

export type MoveBadge = 
  | 'book' 
  | 'brilliant' 
  | 'great' 
  | 'best' 
  | 'excellent' 
  | 'inaccuracy' 
  | 'mistake' 
  | 'blunder';