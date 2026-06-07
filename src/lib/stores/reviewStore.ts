import { writable, derived } from 'svelte/store';
import { listen } from '@tauri-apps/api/event';
import { moves, activePly, resetBoard } from './boardStore';
import type { AnalysisSummary, AnalysisProgress } from '../types/game';

export type SidebarView = 'import' | 'game' | 'summary';

export const isAnalyzing = writable<boolean>(false);
export const analysisSummary = writable<AnalysisSummary | null>(null);
export const loadingProgress = writable<number>(0);
export const sidebarView = writable<SidebarView>('import');

export const currentEval = derived(
  [moves, activePly],
  ([$moves, $activePly]) => $moves[$activePly]?.playedEval ?? 0
);

export const currentMateIn = derived(
  [moves, activePly],
  ([$moves, $activePly]) => $moves[$activePly]?.mateIn ?? null
);

export const resetAnalysis = () => {
  resetBoard();
  isAnalyzing.set(false);
  analysisSummary.set(null);
  loadingProgress.set(0);
};

export async function initAnalysisListeners() {
  await listen('analysis-started', () => {
    resetAnalysis();
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
    console.error('[Engine error]', event.payload);
    isAnalyzing.set(false);
  });
}
