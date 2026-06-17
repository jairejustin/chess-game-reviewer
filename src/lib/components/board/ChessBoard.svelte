<script lang="ts">
  import { chessground } from '../../actions/chessground';
  import { isFlipped } from '../../stores/boardStore';
  import { getInBoardBadge, badgeColors } from '../../utils/boardBadges';
  import PlayerProfile from './PlayerProfile.svelte';
  import EvalBar from '../analysis/EvalBar.svelte';
  import { calculateMaterial } from '../../utils/material';
  import type { MoveNode, EngineLine } from '../../types/game';
  import { playBoardSound } from '$lib/utils/audio';

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

  export let fen: string = 'start';
  export let currentMove: MoveNode | null | undefined = null;
  export let viewOnly: boolean = true;
  export let legalDests: Map<string, string[]> | undefined = undefined;
  export let engineLines: EngineLine[] = [];
  export let onMove: ((orig: string, dest: string) => void) | undefined =
    undefined;

  $: material = calculateMaterial(fen || 'start');

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

  let lastSoundPly = -1;

  $: if (currentMove && currentMove.ply !== lastSoundPly) {
    lastSoundPly = currentMove.ply;

    if (currentMove.ply > 0) {
      const san = currentMove.san || '';
      const isCapture = san.includes('x');

      if (isCapture) {
        playBoardSound('capture');
      } else {
        playBoardSound('move');
      }
    }
  }

  $: {
    let autoShapes: any[] = [];
    let lastMove: string[] = [];
    destHighlight = 'rgba(155, 199, 0, 0.41)';

    if (
      currentMove &&
      currentMove.ply > 0 &&
      typeof currentMove.uci === 'string' &&
      currentMove.uci.length >= 4
    ) {
      const orig = currentMove.uci.substring(0, 2);
      const dest = currentMove.uci.substring(2, 4);
      lastMove = [orig, dest];

      if (currentMove.classification) {
        destHighlight = badgeColors[currentMove.classification] + '66';
        autoShapes.push({
          orig: dest,
          brush: 'invisible',
          customSvg: { html: getInBoardBadge(currentMove.classification) }
        });
      }
    }

    for (let line of engineLines) {
      const brushName = line.rank === 3 ? 'pv3' : line.rank === 2 ? 'pv2' : 'pv1';
      autoShapes.push({
        orig: line.orig,
        dest: line.dest,
        brush: brushName,
      });
    }

    const config: any = {
      fen: fen || 'start',
      orientation: $isFlipped ? 'black' : 'white',
      viewOnly,
      lastMove,
      drawable: {
        brushes: {
          invisible: {
            key: 'i',
            color: 'transparent',
            opacity: 0,
            lineWidth: 1
          },
          pv1: {
            key: '1',
            color: '#3a7d44',
            opacity: 0.9,
            lineWidth: 8
          },
          pv2: {
            key: '2',
            color: '#7a8f35',
            opacity: 0.9,
            lineWidth: 8
          },
          pv3: {
            key: '3',
            color: '#b8992b',
            opacity: 0.9,
            lineWidth: 8
          },
        },
        autoShapes,
        visible: true
      }
    };

    if (!viewOnly) {
      config.movable = {
        free: false,
        color: 'both',
        dests: legalDests,
        showDests: false,
        events: {
          after: onMove
        }
      };
    }

    cgConfig = config;
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
    overflow: visible;
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
