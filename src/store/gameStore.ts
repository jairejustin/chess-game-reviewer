import { writable, derived } from 'svelte/store';
import type {
  AnalyzedMove,
  AnalysisSummary,
  AnalysisProgress
} from '../types/game';
import { listen } from '@tauri-apps/api/event';

export type SidebarView = 'import' | 'game' | 'summary';

export const moves = writable<AnalyzedMove[]>([]);
export const activePly = writable<number>(0);
export const isAnalyzing = writable<boolean>(false);
export const analysisSummary = writable<AnalysisSummary | null>(null);
export const loadingProgress = writable<number>(0);
export const sidebarView = writable<SidebarView>('import');

export const currentFen = derived(
  [moves, activePly],
  ([$moves, $activePly]) => $moves[$activePly]?.fen ?? 'start'
);

export const currentEval = derived(
  [moves, activePly],
  ([$moves, $activePly]) => $moves[$activePly]?.playedEval ?? 0
);

export const currentMateIn = derived(
  [moves, activePly],
  ([$moves, $activePly]) => $moves[$activePly]?.mateIn ?? null
);

export const appendMove = (move: AnalyzedMove) => {
  moves.update((m) => {
    if (m.some((existing) => existing.ply === move.ply)) return m;
    return [...m, move];
  });
};

export const resetGame = () => {
  moves.set([]);
  activePly.set(0);
  isAnalyzing.set(false);
  analysisSummary.set(null);
  loadingProgress.set(0);
};

export async function initTauriListeners() {
  await listen('analysis-started', () => {
    resetGame();
    isAnalyzing.set(true);
  });

  await listen<AnalysisProgress>('analysis-progress', (event) => {
    const { currentPly, totalPlies } = event.payload;
    if (totalPlies > 0) {
      loadingProgress.set(currentPly / totalPlies);
    }
  });

  await listen<AnalysisSummary>('analysis-complete', (event) => {
    isAnalyzing.set(false);
    analysisSummary.set(event.payload);
    moves.set(event.payload.moves);

    if (event.payload.moves.length > 0) {
      activePly.set(event.payload.moves.length - 1);
    }

    loadingProgress.set(1);
    sidebarView.set('summary');
  });

  await listen<string>('analysis-error', (event) => {
    console.error('[Theoria engine error]', event.payload);
    isAnalyzing.set(false);
  });
}
