<script lang="ts">
  import { chessground } from '../../actions/chessground';
  import {
    moves,
    activePly,
    currentFen,
    isFlipped
  } from '../../stores/boardStore';
  import { getInBoardBadge, badgeColors } from '../../utils/boardBadges';
  import PlayerProfile from './PlayerProfile.svelte';
  import EvalBar from '../analysis/EvalBar.svelte';
  import { calculateMaterial } from '../../utils/material';

  // Player Data Props
  export let whiteName: string = 'White';
  export let blackName: string = 'Black';
  export let whiteTitle: string | null = null;
  export let blackTitle: string | null = null;
  export let whiteRating: number | null = null;
  export let blackRating: number | null = null;
  export let whiteAvatar: string | null = null;
  export let blackAvatar: string | null = null;

  $: material = calculateMaterial($currentFen || 'start');
  let destHighlight = 'rgba(155, 199, 0, 0.41)';
  let cgConfig: any = { fen: 'start', viewOnly: true };

  // Flipped Board Logic Mapping
  $: topName = $isFlipped ? whiteName : blackName;
  $: bottomName = $isFlipped ? blackName : whiteName;
  $: topRating = $isFlipped ? whiteRating : blackRating;
  $: bottomRating = $isFlipped ? blackRating : whiteRating;
  $: topAvatar = $isFlipped ? whiteAvatar : blackAvatar;
  $: bottomAvatar = $isFlipped ? blackAvatar : whiteAvatar;
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
  $: topTitle = $isFlipped ? whiteTitle : blackTitle;
  $: bottomTitle = $isFlipped ? blackTitle : whiteTitle;

  // Chessground configuration
  $: {
    const move = $moves[$activePly];
    let autoShapes: any[] = [];
    let lastMove: string[] = [];
    destHighlight = 'rgba(155, 199, 0, 0.41)';

    if (
      move &&
      move.ply > 0 &&
      typeof move.uci === 'string' &&
      move.uci.length >= 4
    ) {
      const orig = move.uci.substring(0, 2);
      const dest = move.uci.substring(2, 4);
      lastMove = [orig, dest];

      if (move.classification) {
        destHighlight = badgeColors[move.classification] + '66';
        autoShapes.push({
          orig: dest,
          brush: 'invisible',
          customSvg: { html: getInBoardBadge(move.classification) }
        });
      }
    }

    cgConfig = {
      fen: $currentFen || 'start',
      orientation: $isFlipped ? 'black' : 'white',
      viewOnly: true,
      lastMove,
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
</script>

<div class="board-layout-grid">
  <div class="grid-top-profile">
    <PlayerProfile
      name={topName}
      title={topTitle}
      rating={topRating}
      avatarUrl={topAvatar}
      capturedPieces={topCaptured}
      advantage={topAdvantage}
    />
  </div>
  <div class="grid-eval">
    <EvalBar />
  </div>
  <div class="grid-board">
    <div class="board-frame">
      <div
        class="board"
        style="--move-highlight: {destHighlight};"
        use:chessground={cgConfig}
      ></div>
    </div>
  </div>
  <div class="grid-bottom-profile">
    <PlayerProfile
      name={bottomName}
      title={bottomTitle}
      rating={bottomRating}
      avatarUrl={bottomAvatar}
      capturedPieces={bottomCaptured}
      advantage={bottomAdvantage}
    />
  </div>
</div>

<style>
  /* Base Board Layout CSS from +page.svelte goes here */
  .board-layout-grid {
    display: grid;
    grid-template-columns: max-content max-content;
    grid-template-rows: max-content minmax(0, 1fr) max-content;
    gap: 0 16px;
    height: 100%;
    max-height: 100%;
  }
  .grid-top-profile {
    grid-column: 2;
    grid-row: 1;
    margin-bottom: 8px;
    width: 100%;
  }
  .grid-eval {
    grid-column: 1;
    grid-row: 2;
    height: 100%;
  }
  .grid-board {
    grid-column: 2;
    grid-row: 2;
    height: 100%;
    display: flex;
  }
  .board-frame {
    height: 100%;
    aspect-ratio: 1 / 1;
    position: relative;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
    border-radius: 4px;
    overflow: hidden;
    flex-shrink: 0;
  }
  .board {
    width: 100%;
    height: 100%;
    position: relative;
    user-select: none;
  }
  .grid-bottom-profile {
    grid-column: 2;
    grid-row: 3;
    margin-top: 8px;
    width: 100%;
  }
</style>
