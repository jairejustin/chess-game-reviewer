<script lang="ts">
  import { moves, activePly } from '../store/gameStore';
  import { tallyLabels, formatEval } from '../utils/ui';
  import Figurine from './Figurine.svelte';
  import Badge from './Badge.svelte';

  $: currentMove = $moves[$activePly];
</script>

{#if currentMove && currentMove.ply > 0}
  <div class="engine-feedback">
    <div class="engine-feedback__header">
      <Badge classification={currentMove.classification} size={28} />
      <span class="engine-feedback__title">
        {tallyLabels[currentMove.classification]}
      </span>
    </div>

    <div class="engine-feedback__comparison">
      <div class="engine-line">
        <span class="engine-line__label">Played:</span>
        <span class="engine-line__move">
          <Figurine san={currentMove.san} />
        </span>
        <span class="engine-line__eval"
          >{formatEval(currentMove.playedEval, currentMove.mateIn)}
        </span>
      </div>

      {#if !['best', 'book', 'forced', 'great', 'brilliant'].includes(currentMove.classification)}
        <div class="engine-line engine-line--best">
          <span class="engine-line__label">Best:</span>
          <span class="engine-line__move">
            <Figurine san={currentMove.bestMoveSan} />
          </span>
          <span class="engine-line__eval"
            >{formatEval(currentMove.prevBestEval)}</span
          >
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .engine-feedback {
    background: #1c1c1f;
    padding: 1rem 1.25rem;
    display: flex;
    flex-direction: column;
    gap: 0.8rem;
    flex-shrink: 0;
    border-bottom: 1px solid #2a2a2e;
  }
  .engine-feedback__header {
    display: flex;
    align-items: center;
    gap: 0.6rem;
  }
  .engine-feedback__title {
    font-family: 'Bebas Neue', sans-serif;
    font-size: 1.5rem;
    letter-spacing: 1px;
    color: #ececec;
  }
  .engine-feedback__comparison {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    background: #161618;
    padding: 0.6rem 0.8rem;
    border-radius: 6px;
    border: 1px solid #2a2a2e;
  }
  .engine-line {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    font-size: 0.95rem;
  }
  .engine-line__label {
    color: #666;
    font-weight: 600;
    width: 60px;
    font-size: 0.8rem;
    text-transform: uppercase;
  }
  .engine-line__move {
    font-weight: 600;
    color: #ececec;
    flex: 1;
  }
  .engine-line__eval {
    font-family: 'Bebas Neue', sans-serif;
    color: #888;
    font-size: 1.1rem;
    letter-spacing: 0.5px;
  }
  .engine-line--best .engine-line__move {
    color: #95bb4a;
  }
</style>
