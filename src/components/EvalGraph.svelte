<script lang="ts">
  import { moves, activePly } from '../store/gameStore';

  function winPercent(cp: number, mateIn: number | null): number {
    if (mateIn !== null) {
      if (mateIn > 0) return 100;
      if (mateIn < 0) return 0;
      if (mateIn === 0 && cp >= 0) return 100;
      if (mateIn === 0 && cp < 0) return 0;
    }
    return 50 + 50 * (2 / (1 + Math.exp(-0.00368 * cp)) - 1);
  }

  $: points = $moves.map((m, i) => {
    const x = $moves.length > 1 ? (i / ($moves.length - 1)) * 100 : 0;
    const y = 100 - winPercent(m.playedEval, m.mateIn);
    return { x, y };
  });

  $: areaPathData =
    points.length > 0
      ? `M 0 50 ` +
        points.map((p) => `L ${p.x} ${p.y}`).join(' ') +
        ` L 100 50 Z`
      : '';

  $: linePathData =
    points.length > 0
      ? `M ` + points.map((p) => `${p.x} ${p.y}`).join(' L ')
      : '';

  $: activeX = $moves.length > 1 ? ($activePly / ($moves.length - 1)) * 100 : 0;

  let svgElement: SVGSVGElement;

  function handleGraphClick(e: MouseEvent) {
    if (!svgElement || $moves.length <= 1) return;
    const rect = svgElement.getBoundingClientRect();
    const percent = Math.max(
      0,
      Math.min(1, (e.clientX - rect.left) / rect.width)
    );
    const clickedPly = Math.round(percent * ($moves.length - 1));
    activePly.set(clickedPly);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'ArrowLeft' && $activePly > 0) {
      e.preventDefault();
      activePly.update((p) => p - 1);
    } else if (e.key === 'ArrowRight' && $activePly < $moves.length - 1) {
      e.preventDefault();
      activePly.update((p) => p + 1);
    }
  }
</script>

<div class="eval-graph">
  <svg
    bind:this={svgElement}
    viewBox="0 0 100 100"
    preserveAspectRatio="none"
    role="slider"
    tabindex="0"
    aria-label="Game evaluation over time"
    aria-valuemin={0}
    aria-valuemax={$moves.length > 0 ? $moves.length - 1 : 0}
    aria-valuenow={$activePly}
    on:click={handleGraphClick}
    on:keydown={handleKeydown}
  >
    <line x1="0" y1="50" x2="100" y2="50" class="graph-centerline" />

    <clipPath id="white-clip">
      <rect x="0" y="0" width="100" height="50" />
    </clipPath>
    <path
      d={areaPathData}
      class="graph-fill--white"
      clip-path="url(#white-clip)"
    />

    <clipPath id="black-clip">
      <rect x="0" y="50" width="100" height="50" />
    </clipPath>
    <path
      d={areaPathData}
      class="graph-fill--black"
      clip-path="url(#black-clip)"
    />

    <path
      d={linePathData}
      class="graph-line"
      vector-effect="non-scaling-stroke"
    />

    {#if $moves.length > 0}
      <line
        x1={activeX}
        y1="0"
        x2={activeX}
        y2="100"
        class="graph-indicator"
        vector-effect="non-scaling-stroke"
      />
    {/if}
  </svg>
</div>

<style>
  .eval-graph {
    width: 100%;
    height: 110px;
    background: #161618;
    border: 1px solid #2a2a2e;
    border-radius: 8px;
    overflow: hidden;
    position: relative;
    cursor: crosshair;
  }

  svg {
    width: 100%;
    height: 100%;
    display: block;
    outline: none;
  }

  svg:focus-visible {
    outline: 2px solid #5c8bb0;
    outline-offset: -2px;
  }

  .graph-centerline {
    stroke: rgba(255, 255, 255, 0.1);
    stroke-width: 0.5px;
  }

  .graph-fill--white {
    fill: #f0ede8;
    opacity: 0.7;
  }

  .graph-fill--black {
    fill: #3a3a3a;
    opacity: 0.8;
  }

  .graph-line {
    fill: none;
    stroke: #888;
    stroke-width: 1.5px;
  }

  .graph-indicator {
    stroke: #95bb4a;
    stroke-width: 1.5px;
  }
</style>
