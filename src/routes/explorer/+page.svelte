<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { get } from 'svelte/store';
  import { Chess } from 'chess.js';

  // Stores
  import { selectedGame } from '$lib/stores/fetchStore';
  import { moves, activePly, isFlipped } from '$lib/stores/boardStore';
  import {
    currentLine,
    explorerIndex,
    activeExplorerFen,
    liveEval,
    liveMateIn,
    engineOn,
    mountExplorer,
    unmountExplorer,
    enterVariationFromMove
  } from '$lib/stores/explorerStore';

  // Components
  import { chessground } from '$lib/actions/chessground';
  import EvalBar from '$lib/components/analysis/EvalBar.svelte';
  import PlayerProfile from '$lib/components/board/PlayerProfile.svelte';
  import ExplorerSidebar from '$lib/components/explorer/ExplorerSidebar.svelte';
  import ActionStrip from '$lib/components/ui/ActionStrip.svelte';
  import { calculateMaterial } from '$lib/utils/material';
  import { getInBoardBadge, badgeColors } from '$lib/utils/boardBadges';

  $: whiteName = $selectedGame?.white.username ?? 'White';
  $: blackName = $selectedGame?.black.username ?? 'Black';

  $: material = calculateMaterial($activeExplorerFen || 'start');

  $: topName = $isFlipped ? whiteName : blackName;
  $: bottomName = $isFlipped ? blackName : whiteName;
  
  $: topRating = $isFlipped
    ? ($selectedGame?.white.rating ?? null)
    : ($selectedGame?.black.rating ?? null);
  $: bottomRating = $isFlipped
    ? ($selectedGame?.black.rating ?? null)
    : ($selectedGame?.white.rating ?? null);

  $: topCaptured = $isFlipped ? material.whiteCaptured : material.blackCaptured;
  $: bottomCaptured = $isFlipped
    ? material.blackCaptured
    : material.whiteCaptured;
    
  $: topAdvantage = $isFlipped
    ? material.whiteAdvantage
    : material.blackAdvantage;
  $: bottomAdvantage = $isFlipped
    ? material.blackAdvantage
    : material.whiteAdvantage;

  let cgConfig: any = { fen: 'start', viewOnly: false };

  // Chessground config derived purely from currentLine and explorerIndex
  $: {
    const activeMove = $currentLine[$explorerIndex];
    let lastMove: string[] = [];
    let autoShapes: any[] = [];
    let destHighlight = 'rgba(155, 199, 0, 0.41)';

    if (
      activeMove &&
      activeMove.ply > 0 &&
      typeof activeMove.uci === 'string' &&
      activeMove.uci.length >= 4
    ) {
      const orig = activeMove.uci.substring(0, 2);
      const dest = activeMove.uci.substring(2, 4);
      lastMove = [orig, dest];

      if (activeMove.source === 'game' && activeMove.classification) {
        destHighlight = badgeColors[activeMove.classification] + '66';
        autoShapes.push({
          orig: dest,
          brush: 'invisible',
          customSvg: { html: getInBoardBadge(activeMove.classification) }
        });
      }
    }

    const legalDests = computeLegalDests($activeExplorerFen);

    cgConfig = {
      fen: $activeExplorerFen || 'start',
      orientation: $isFlipped ? 'black' : 'white',
      viewOnly: false,
      lastMove,
      movable: {
        free: false,
        color: 'both',
        dests: legalDests,
        events: {
          after: handleBoardMove
        }
      },
      drawable: {
        brushes: {
          invisible: {
            key: 'i',
            color: 'transparent',
            opacity: 0,
            lineWidth: 1
          }
        },
        autoShapes,
        visible: true
      }
    };
  }

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
    <div class="board-anchor">
      <div class="anchor-top">
        <PlayerProfile
          name={topName}
          rating={topRating}
          capturedPieces={topCaptured}
          advantage={topAdvantage}
        />
      </div>

      <div class="anchor-left">
        <EvalBar eval_cp={$liveEval} mateIn={$liveMateIn} active={$engineOn} />
      </div>

      <div class="board-frame">
        <div class="board" use:chessground={cgConfig}></div>
      </div>

      <div class="anchor-bottom">
        <PlayerProfile
          name={bottomName}
          rating={bottomRating}
          capturedPieces={bottomCaptured}
          advantage={bottomAdvantage}
        />
      </div>
    </div>
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
  .board-anchor {
    position: relative;
    height: 100%;
    aspect-ratio: 1 / 1;
    flex-shrink: 1;
  }
  .board-frame {
    width: 100%;
    height: 100%;
    position: relative;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
    border-radius: 4px;
    overflow: hidden;
  }
  .board {
    width: 100%;
    height: 100%;
    position: relative;
    user-select: none;
  }
  .anchor-top {
    position: absolute;
    bottom: 100%;
    left: 0;
    width: 100%;
    margin-bottom: 8px;
  }
  .anchor-bottom {
    position: absolute;
    top: 100%;
    left: 0;
    width: 100%;
    margin-top: 8px;
  }
  .anchor-left {
    position: absolute;
    right: 100%;
    top: 0;
    height: 100%;
    margin-right: 16px;
  }
</style>