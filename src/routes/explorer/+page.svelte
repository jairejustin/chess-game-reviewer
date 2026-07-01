<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { get } from 'svelte/store';
  import { Chess } from 'chess.js';

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

  import AppLayout from '$lib/components/ui/AppLayout.svelte';
  import ChessBoard from '$lib/components/board/ChessBoard.svelte';
  import ExplorerSidebar from '$lib/components/explorer/ExplorerSidebar.svelte';

  $: whiteName = $selectedGame?.white.username ?? 'White';
  $: blackName = $selectedGame?.black.username ?? 'Black';

  $: legalDests = computeLegalDests($activeExplorerFen);

  $: boardEngineLines = $livePVLines
    .filter((line) => line.uciMoves && line.uciMoves.length > 0)
    .map((line, i) => {
      const firstUci = line.uciMoves[0];
      return {
        orig: firstUci.substring(0, 2),
        dest: firstUci.substring(2, 4),
        rank: i + 1
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

  onMount(async () => {
    await mountExplorer(get(moves), get(activePly));
  });

  onDestroy(async () => {
    await unmountExplorer();
  });
</script>

<AppLayout>
  <svelte:fragment slot="board">
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
  </svelte:fragment>

  <ExplorerSidebar slot="sidebar" />
</AppLayout>
