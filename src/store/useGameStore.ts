import { create } from "zustand";
import { AnalyzedMove, MoveBadge } from "../types/game";

interface GameState {
    moves: AnalyzedMove[];
    activePly: number;
    isAnalyzing: boolean;
}

interface GameActions {
    setActivePly: (ply: number) => void;
    appendMove: (move: AnalyzedMove) => void;
    setIsAnalyzing: (to: boolean) => void;
    reset: () => void;
}

type GameStore = GameState & GameActions;

export const useGameStore = create<GameStore>((set) => ({
    moves: [],
    activePly: 0,
    isAnalyzing: false,
    setActivePly: (ply: number): void => set({ activePly: ply }),
    appendMove: (move: AnalyzedMove): void => set((state) => ({
        moves: [ ...state.moves, move]
    })),
    setIsAnalyzing: (to: boolean): void => set({ isAnalyzing: to }),
    reset: () => set({ moves: [], activePly: 0, isAnalyzing: false })

}))