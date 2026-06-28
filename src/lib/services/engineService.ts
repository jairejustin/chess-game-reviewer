import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { AnalysisSummary, AnalysisProgress } from '../types/game';

export const EngineService = {
  async toggleLiveEngine(start: boolean): Promise<void> {
    await invoke('toggle_live_engine', { start });
  },

  async analyzeLivePosition(fen: string, multipv: number = 3): Promise<void> {
    await invoke('analyze_live_position', { fen, multipv });
  },

  async stopLiveAnalysis(): Promise<void> {
    await invoke('stop_live_analysis');
  },

  async listenToLiveEngine(
    callback: (payload: any) => void
  ): Promise<UnlistenFn> {
    return await listen<any>('live-engine-info', (event) => {
      callback(event.payload);
    });
  },

  async analyzeFullGame(pgn: string): Promise<void> {
    await invoke('analyze_game', { pgn });
  },

  async listenToAnalysisEvents(callbacks: {
    onStart: () => void;
    onProgress: (payload: AnalysisProgress) => void;
    onComplete: (payload: AnalysisSummary) => void;
    onError: (error: string) => void;
  }) {
    const unlisteners = await Promise.all([
      listen('analysis-started', callbacks.onStart),
      listen<AnalysisProgress>('analysis-progress', (e) =>
        callbacks.onProgress(e.payload)
      ),
      listen<AnalysisSummary>('analysis-complete', (e) =>
        callbacks.onComplete(e.payload)
      ),
      listen<string>('analysis-error', (e) => callbacks.onError(e.payload))
    ]);

    return () => unlisteners.forEach((unlisten) => unlisten());
  }
};
