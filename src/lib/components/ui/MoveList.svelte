<script lang="ts">
  import { autoScroll } from '$lib/actions/scroll';
  import Figurine from '$lib/components/ui/Figurine.svelte';
  import Badge from '$lib/components/ui/Badge.svelte';
  import { formatEval } from '$lib/utils/ui';
  import type { MoveNode } from '$lib/types/game';

  export let moves: MoveNode[];
  export let activeIndex: number;
  export let onSelect: (index: number) => void;

  export let showBadges: boolean = false;
  export let emptyMessage: string = 'No moves yet.';
</script>

<div class="move-list">
  {#if moves.length <= 1}
    <div class="move-list__empty">
      <p>{emptyMessage}</p>
    </div>
  {:else}
    {#each moves as move, index}
      {#if index > 0}
        <button
          class="move-list__row"
          class:move-list__row--active={activeIndex === index}
          on:click={() => onSelect(index)}
          use:autoScroll={activeIndex === index}
        >
          <div class="move-list__info">
            <span class="move-list__ply">
              {Math.ceil(move.ply / 2)}{move.ply % 2 !== 0 ? '.' : '...'}
            </span>
            <span class="move-list__san">
              <Figurine san={move.san} />
            </span>
          </div>

          <div class="move-list__meta">
            {#if showBadges && move.classification}
              <span class="move-list__eval">
                {formatEval(move.playedEval ?? 0, move.mateIn)}
              </span>
              <div title={move.classification}>
                <Badge classification={move.classification} size={26} />
              </div>
            {/if}

            {#if !showBadges && move.source === 'variation'}
              <div class="move-list__meta--variation">
                <span class="variation-indicator">~</span>
              </div>
            {/if}
          </div>
        </button>
      {/if}
    {/each}
  {/if}
</div>

<style>
  .move-list {
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    padding: 0.5rem;
    scrollbar-width: thin;
    scrollbar-color: #333 transparent;
  }
  .move-list__empty {
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #555;
  }

  .move-list__row {
    appearance: none;
    background: transparent;
    border: 1px solid transparent;
    text-align: left;
    font-family: inherit;
    color: inherit;
    width: 100%;
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.4rem 1rem;
    margin-bottom: 4px;
    border-radius: 8px;
    transition: all 0.15s ease;
    cursor: pointer;
  }
  .move-list__row:hover {
    background: #232326;
  }
  .move-list__row--active {
    background: #252a30;
    border-color: #34404a;
  }

  .move-list__info {
    display: flex;
    gap: 0.8rem;
    align-items: center;
  }
  .move-list__ply {
    color: #777;
    font-weight: 500;
    font-size: 1rem;
    width: 42px;
    flex-shrink: 0;
  }
  .move-list__san {
    font-weight: 400;
    font-size: 1rem;
  }

  .move-list__meta {
    display: flex;
    align-items: center;
    gap: 0.6rem;
  }
  .move-list__eval {
    font-family: 'Bebas Neue', sans-serif;
    color: #888;
    font-size: 1.15rem;
    width: 52px;
    text-align: right;
    letter-spacing: 0.5px;
  }
  .move-list__meta--variation {
    opacity: 0.3;
  }
  .variation-indicator {
    font-family: 'Bebas Neue', sans-serif;
    font-size: 1rem;
    color: #8be1b4;
  }
</style>
