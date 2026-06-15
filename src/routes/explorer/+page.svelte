<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { get } from 'svelte/store';
  import { Chess } from 'chess.js';

  // Stores
  import { selectedGame } from '$lib/stores/fetchStore';
  import { moves, activePly } from '$lib/stores/boardStore';
  import {
    currentLine,
    explorerIndex,
    activeExplorerFen,
    liveEval,
    liveMateIn,
    engineOn,
    mountExplorer,
    unmountExplorer,
    livePVLines,
    enterVariationFromMove
  } from '$lib/stores/explorerStore';

  // Components
  import ChessBoard from '$lib/components/board/ChessBoard.svelte';
  import ExplorerSidebar from '$lib/components/explorer/ExplorerSidebar.svelte';
  import ActionStrip from '$lib/components/ui/ActionStrip.svelte';
  import type { EngineLine } from '$lib/types/game';

  $: whiteName = $selectedGame?.white.username ?? 'White';
  $: blackName = $selectedGame?.black.username ?? 'Black';

  $: legalDests = computeLegalDests($activeExplorerFen);

  $: boardEngineLines = $livePVLines
    .filter(line => line.uciMoves && line.uciMoves.length > 0)
    .map((line, i) => {
      const firstUci = line.uciMoves[0];
      return {
        orig: firstUci.substring(0, 2),
        dest: firstUci.substring(2, 4),
        rank: i + 1,
      };
    });

  function computeLegalDests(fen: string): Map<string, string[]> {
    const dests = new Map<string, string[]>();
    try {
      const chess = new Chess(fen === 'start' ? undefined : fen);
      const chessMoves = chess.moves({ verbose: true });
      for (const move of chessMoves) {
        const existing = dests.get(move.from) ?? [];
        existing.push(move.to);
        dests.set(move.from, existing);
      }
    } catch {}
    return dests;
  }

  function handleBoardMove(orig: string, dest: string) {
    const fen = get(activeExplorerFen);
    try {
      const chess = new Chess(fen === 'start' ? undefined : fen);
      const result = chess.move({ from: orig, to: dest, promotion: 'q' });
      if (!result) return;

      enterVariationFromMove(
        result.san,
        chess.fen(),
        orig + dest + (result.promotion ?? '')
      );
    } catch {}
  }

  // Lifecycle
  onMount(async () => {
    await mountExplorer(get(moves), get(activePly));
  });

  onDestroy(async () => {
    await unmountExplorer();
  });
</script>

<main class="layout">
  <section class="layout__board">
    <ChessBoard
      {whiteName}
      {blackName}
      whiteRating={$selectedGame?.white.rating ?? null}
      blackRating={$selectedGame?.black.rating ?? null}
      evalCp={$liveEval}
      evalMateIn={$liveMateIn}
      evalActive={$engineOn}
      fen={$activeExplorerFen}
      currentMove={$currentLine[$explorerIndex]}
      viewOnly={false}
      {legalDests}
      onMove={handleBoardMove}
      engineLines={boardEngineLines}
    />
  </section>

  <ActionStrip />
  <ExplorerSidebar />
</main>

<style>
  .layout {
    display: flex;
    height: 100vh;
    width: 100vw;
    max-width: 100%;
    margin: 0;
    padding: 1rem;
    gap: 0;
    box-sizing: border-box;
    overflow: hidden;
    align-items: flex-start;
  }
  .layout__board {
    flex: 1;
    display: flex;
    justify-content: center;
    align-items: center;
    height: 100%;
    min-height: 0;
    padding: 4rem 1rem 4rem 4rem;
    box-sizing: border-box;
  }
</style>
