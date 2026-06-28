import { writable, derived, get } from 'svelte/store';
import { Chess } from 'chess.js';
import { EngineService } from '../services/engineService';
import type { MoveNode, PVLine } from '../types/game';

const STABLE_DEPTH = 8;

/** The immutable copy of the main game */
export const snapshotMoves = writable<MoveNode[]>([]);

/** The mutable working array representing what the UI is currently displaying */
export const currentLine = writable<MoveNode[]>([]);

/** The cursor pointing to the active move within currentLine */
export const explorerIndex = writable<number>(0);

/** Derived active FEN based strictly on the cursor */
export const activeExplorerFen = derived(
  [currentLine, explorerIndex],
  ([$line, $idx]) => $line[$idx]?.fen ?? 'start'
);

export const engineOn = writable<boolean>(false);
export const engineStatus = writable<'thinking' | 'paused' | 'starting'>(
  'paused'
);
export const liveEval = writable<number>(0);
export const liveMateIn = writable<number | null>(null);
export const livePVLines = writable<PVLine[]>([]);
export const currentDepth = writable<number>(0);

let analyzeTimeout: ReturnType<typeof setTimeout>;
let engineHeartbeat: ReturnType<typeof setTimeout>;
let activeSearchFen = '';
let mateAlreadyHandled = false;

function handleTerminalState(chess: Chess) {
  if (chess.isCheckmate()) {
    const whiteIsMated = chess.turn() === 'w';
    liveMateIn.set(0);
    liveEval.set(whiteIsMated ? -10000 : 10000);
  } else {
    liveMateIn.set(null);
    liveEval.set(0);
  }
  engineStatus.set('paused');
  if (flushTimer) {
    clearTimeout(flushTimer);
    flushTimer = null;
  }
  pendingLines.clear();
  livePVLines.set([]);
}

const engineWatcher = derived(
  [activeExplorerFen, engineOn],
  ([$fen, $engineOn]) => ({ fen: $fen, on: $engineOn })
);

engineWatcher.subscribe(({ fen, on }) => {
  clearTimeout(analyzeTimeout);
  clearTimeout(engineHeartbeat);
  activeSearchFen = '';
  mateAlreadyHandled = false;

  if (!on) {
    EngineService.stopLiveAnalysis().catch(console.error);
    engineStatus.set('paused');
    livePVLines.set([]);
    return;
  }

  livePVLines.set([]);

  try {
    const chess = new Chess(fen === 'start' ? undefined : fen);
    if (chess.isGameOver()) {
      handleTerminalState(chess);
      return;
    }
  } catch (e) {
    console.error('FEN parse error:', e);
    engineStatus.set('paused');
    return;
  }

  engineStatus.set('starting');

  analyzeTimeout = setTimeout(() => {
    const uciFen =
      fen === 'start'
        ? 'rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1'
        : fen;

    activeSearchFen = uciFen;
    pendingLines.clear();
    mateAlreadyHandled = false;

    EngineService.analyzeLivePosition(uciFen, 3).catch((err) => {
      console.error('Engine invoke error:', err);
      if (activeSearchFen === uciFen) {
        engineStatus.set('paused');
      }
    });

    engineHeartbeat = setTimeout(() => {
      if (activeSearchFen !== uciFen) return;
      try {
        const chess = new Chess(fen === 'start' ? undefined : fen);
        if (chess.isGameOver()) {
          handleTerminalState(chess);
        }
      } catch (e) {
        console.error('Heartbeat FEN error:', e);
      }
    }, 500);
  }, 150);
});

export function setExplorerIndex(index: number) {
  const line = get(currentLine);
  const snapshot = get(snapshotMoves);

  if (!line[index]) return;

  if (line[index].source === 'game') {
    currentLine.set([...snapshot]);
  }

  explorerIndex.set(index);
}

export function enterVariationFromPV(line: PVLine) {
  if (line.uciMoves.length === 0 || line.sanMoves.length === 0) return;

  const cursor = get(explorerIndex);
  const cl = get(currentLine);
  const startFen = cl[cursor]?.fen ?? 'start';

  const chess = new Chess(startFen === 'start' ? undefined : startFen);

  const uci = line.uciMoves[0];
  const san = line.sanMoves[0];
  const from = uci.slice(0, 2);
  const to = uci.slice(2, 4);
  const promotion = uci.length === 5 ? uci[4] : undefined;

  try {
    const result = chess.move({ from, to, promotion });
    if (result) {
      enterVariationFromMove(san, chess.fen(), uci);
    }
  } catch (e) {
    console.error('Failed to parse PV variation move', e);
  }
}

export function enterVariationFromMove(san: string, fen: string, uci: string) {
  const cursor = get(explorerIndex);
  const cl = get(currentLine);
  const snapshot = get(snapshotMoves);

  const isOnMainLine = cl[cursor]?.fen === snapshot[cursor]?.fen;
  const nextSnapshotMove = snapshot[cursor + 1];

  if (isOnMainLine && nextSnapshotMove && nextSnapshotMove.uci === uci) {
    currentLine.set([...snapshot]);
    explorerIndex.set(cursor + 1);
    return;
  }

  const nextCurrentMove = cl[cursor + 1];
  if (nextCurrentMove && nextCurrentMove.uci === uci) {
    explorerIndex.set(cursor + 1);
    return;
  }

  const base = cl.slice(0, cursor + 1);
  const lastPly = base.length > 0 ? base[base.length - 1].ply : 0;

  const newNode: MoveNode = {
    ply: lastPly + 1,
    san,
    fen,
    uci,
    source: 'variation'
  };

  currentLine.set([...base, newNode]);
  explorerIndex.set(cursor + 1);
}

let unlistenEngine: (() => void) | null = null;

const pendingLines = new Map<number, PVLine>();
let flushTimer: ReturnType<typeof setTimeout> | null = null;

export async function mountExplorer(gameMoves: MoveNode[], startIndex: number) {
  await EngineService.toggleLiveEngine(true).catch(console.error);

  const initialMoves: MoveNode[] =
    gameMoves.length > 0
      ? gameMoves
      : [
          {
            ply: 0,
            san: '',
            fen: 'start',
            uci: '',
            source: 'game'
          }
        ];

  snapshotMoves.set([...initialMoves]);
  currentLine.set([...initialMoves]);
  explorerIndex.set(startIndex);

  unlistenEngine = await EngineService.listenToLiveEngine((payload) => {
    const { fen: payloadFen, depth, multipv, evaluation, pv } = payload;

    if (payloadFen !== activeSearchFen) return;
    if (mateAlreadyHandled) return;

    clearTimeout(engineHeartbeat);

    const { cp, mateIn, formatted } = parseRawEngineEval(evaluation);
    const isMate = mateIn !== null;

    if (isMate && multipv === 1) {
      mateAlreadyHandled = true;

      if (flushTimer) {
        clearTimeout(flushTimer);
        flushTimer = null;
      }
      pendingLines.clear();

      currentDepth.set(99);
      engineStatus.set('paused');
      EngineService.stopLiveAnalysis().catch(console.error);

      const line = uciLineToPVLine(
        payloadFen,
        pv,
        formatted,
        cp,
        mateIn,
        multipv
      );
      if (line) {
        liveEval.set(cp);
        liveMateIn.set(mateIn);
        livePVLines.set([line]);
      }
      return;
    }

    currentDepth.set(depth);

    const isStable = depth >= STABLE_DEPTH;

    if (isStable) {
      const line = uciLineToPVLine(
        payloadFen,
        pv,
        formatted,
        cp,
        mateIn,
        multipv
      );
      if (!line) return;
      pendingLines.set(multipv, line);

      if (!flushTimer) {
        flushTimer = setTimeout(() => {
          const sorted = Array.from(pendingLines.values()).sort(
            (a, b) => a.index - b.index
          );
          livePVLines.set(sorted);

          const best = pendingLines.get(1);
          if (best) {
            liveEval.set(best.evalCp);
            liveMateIn.set(best.mateIn);
          }

          engineStatus.set('thinking');
          flushTimer = null;
        }, 60);
      }
    }
  });

  engineOn.set(true);
}

export async function unmountExplorer() {
  if (unlistenEngine) {
    unlistenEngine();
    unlistenEngine = null;
  }

  engineOn.set(false);

  snapshotMoves.set([]);
  currentLine.set([]);
  explorerIndex.set(0);
}

export function toggleEngine(on: boolean) {
  engineOn.set(on);
}

function uciLineToPVLine(
  startFen: string,
  uciMoves: string[],
  evaluation: string,
  evalCp: number,
  mateIn: number | null,
  index: number
): PVLine | null {
  const chess = new Chess(startFen === 'start' ? undefined : startFen);
  const sanMoves: string[] = [];

  for (let i = 0; i < uciMoves.length; i++) {
    const uci = uciMoves[i];
    const from = uci.slice(0, 2);
    const to = uci.slice(2, 4);
    const promotion = uci.length === 5 ? uci[4] : undefined;

    try {
      const result = chess.move({ from, to, promotion });
      if (!result) {
        if (i === 0) return null;
        break;
      }
      sanMoves.push(result.san);
    } catch {
      if (i === 0) return null;
      break;
    }
  }

  return { index, evaluation, evalCp, mateIn, sanMoves, uciMoves };
}

function parseRawEngineEval(evaluation: string): {
  cp: number;
  mateIn: number | null;
  formatted: string;
} {
  if (evaluation.includes('M')) {
    const mateIn = parseInt(evaluation.replace('+', '').replace('M', ''));
    const cp = mateIn > 0 ? 10000 : -10000;
    const formatted = mateIn > 0 ? `+M${mateIn}` : `-M${Math.abs(mateIn)}`;
    return { cp, mateIn, formatted };
  }

  const cp = Math.round(parseFloat(evaluation) * 100);
  const formatted =
    cp === 0 ? '0.00' : (cp > 0 ? '+' : '') + (cp / 100).toFixed(2);
  return { cp: isNaN(cp) ? 0 : cp, mateIn: null, formatted };
}
