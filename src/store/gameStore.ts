import { writable, derived } from 'svelte/store';
import type { AnalyzedMove, AnalysisSummary } from '../types/game';
import { listen } from '@tauri-apps/api/event';


export const moves = writable<AnalyzedMove[]>([]);
export const activePly = writable<number>(0);
export const isAnalyzing = writable<boolean>(false);
export const analysisSummary = writable<AnalysisSummary | null>(null);


export const currentFen = derived(
    [moves, activePly],
    ([$moves, $activePly]) => $moves[$activePly]?.fen ?? 'start'
);

export const currentEval = derived(
    [moves, activePly],
    ([$moves, $activePly]) => $moves[$activePly]?.playedEval ?? 0
);


export const appendMove = (move: AnalyzedMove) => {
    moves.update(m => {
        if (m.some(existing => existing.ply === move.ply)) return m;
        return [...m, move];
    });
};

export const resetGame = () => {
    moves.set([]);
    activePly.set(0);
    isAnalyzing.set(false);
    analysisSummary.set(null);
};


export async function initTauriListeners() {
    await listen('analysis-started', () => {
        resetGame();
        isAnalyzing.set(true);
    });

    await listen<AnalyzedMove>('batch-tick', (event) => {
        appendMove(event.payload);

        moves.subscribe(currentMoves => {
            activePly.set(currentMoves.length - 1);
        })();
    });

    await listen<AnalysisSummary>('analysis-complete', (event) => {
        isAnalyzing.set(false);
        analysisSummary.set(event.payload);
    });

    await listen<string>('analysis-error', (event) => {
        console.error('[Theoria engine error]', event.payload);
        isAnalyzing.set(false);
    });
}