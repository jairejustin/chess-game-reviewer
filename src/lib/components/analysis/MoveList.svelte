<script lang="ts">
  import { moves, activePly } from '$lib/stores/boardStore';
  import { isAnalyzing } from '$lib/stores/reviewStore';
  import { formatEval } from '$lib/utils/ui';
  import Badge from '$lib/components/ui/Badge.svelte';
  import Figurine from '$lib/components/ui/Figurine.svelte';
  import Loader2 from 'lucide-svelte/icons/loader-2';

  function autoScroll(node: HTMLElement, isActive: boolean) {
    if (isActive) {
      node.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
    }
    return {
      update(newIsActive: boolean) {
        if (newIsActive) {
          node.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
        }
      }
    };
  }
</script>

<div class="move-list">
  {#if $moves.length === 0}
    <div class="move-list__empty">
      {#if $isAnalyzing}
        <Loader2 size={32} class="spin" strokeWidth={3} color="#5c8bb0" />
        <p>Engine is calculating...</p>
      {:else}
        <p>No moves analyzed yet.</p>
      {/if}
    </div>
  {:else}
    {#each $moves as move, index}
      <button
        id="ply-{index}"
        class="move-list__row"
        class:move-list__row--active={$activePly === index}
        on:click={() => activePly.set(index)}
        use:autoScroll={$activePly === index}
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
          {#if move.classification}
            <span class="move-list__eval">
              {formatEval(move.playedEval ?? 0, move.mateIn)}
            </span>
            <div title={move.classification}>
              <Badge classification={move.classification} size={26} />
            </div>
          {/if}
        </div>
      </button>
    {/each}
  {/if}
</div>

<style>
  .move-list {
    flex: 1;
    overflow-y: auto;
    padding: 0.5rem;
    scrollbar-width: thin;
    scrollbar-color: #333 transparent;
  }
  .move-list__empty {
    height: 100%;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 1rem;
    color: #666;
    font-weight: 500;
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
</style>
