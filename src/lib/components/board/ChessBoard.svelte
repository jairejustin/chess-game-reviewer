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

  export let whiteName: string = 'White';
  export let blackName: string = 'Black';
  export let whiteTitle: string | null = null;
  export let blackTitle: string | null = null;
  export let whiteRating: number | null = null;
  export let blackRating: number | null = null;
  export let whiteAvatar: string | null = null;
  export let blackAvatar: string | null = null;

  export let evalCp: number = 0;
  export let evalMateIn: number | null = null;
  export let evalActive: boolean = false;

  $: material = calculateMaterial($currentFen || 'start');

  let destHighlight = 'rgba(155, 199, 0, 0.41)';
  let cgConfig: any = { fen: 'start', viewOnly: true };

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

<div class="board-anchor">
  <div class="anchor-top">
    <PlayerProfile
      name={topName}
      title={topTitle}
      rating={topRating}
      avatarUrl={topAvatar}
      capturedPieces={topCaptured}
      advantage={topAdvantage}
    />
  </div>

  <div class="anchor-left">
    <EvalBar eval_cp={evalCp} mateIn={evalMateIn} active={evalActive} />
  </div>

  <div class="board-frame">
    <div
      class="board"
      style="--move-highlight: {destHighlight};"
      use:chessground={cgConfig}
    ></div>
  </div>

  <div class="anchor-bottom">
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
