import { writable, derived } from 'svelte/store';
import { EngineService } from '../services/engineService';
import { moves, activePly, resetBoard } from './boardStore';
import type { AnalysisSummary } from '../types/game';

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
  await EngineService.listenToAnalysisEvents({
    onStart: () => {
      resetAnalysis();
      isAnalyzing.set(true);
    },
    onProgress: (payload) => {
      if (payload.totalPlies > 0) {
        loadingProgress.set(payload.currentPly / payload.totalPlies);
      }
    },
    onComplete: (payload) => {
      isAnalyzing.set(false);
      analysisSummary.set(payload);
      moves.set(payload.moves.map((m) => ({ ...m, source: 'game' as const })));
      if (payload.moves.length > 0) {
        activePly.set(payload.moves.length - 1);
      }
      loadingProgress.set(1);
      sidebarView.set('summary');
    },
    onError: (error) => {
      console.error('[Engine error]', error);
      isAnalyzing.set(false);
    }
  });
}
