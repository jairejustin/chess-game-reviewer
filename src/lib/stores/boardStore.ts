import { writable, derived } from 'svelte/store';
import type { MoveNode } from '../types/game';

export const moves = writable<MoveNode[]>([]);
export const activePly = writable<number>(0);
export const isFlipped = writable<boolean>(false);

export const currentFen = derived(
  [moves, activePly],
  ([$moves, $activePly]) => $moves[$activePly]?.fen ?? 'start'
);

export const appendMove = (move: MoveNode) => {
  moves.update((m) => {
    if (m.some((existing) => existing.ply === move.ply)) return m;
    return [...m, move];
  });
};

export const resetBoard = () => {
  moves.set([]);
  activePly.set(0);
};
