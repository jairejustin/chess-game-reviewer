<script lang="ts">
  import {
    currentLine,
    explorerIndex,
    setExplorerIndex
  } from '$lib/stores/explorerStore';
  import Figurine from '$lib/components/ui/Figurine.svelte';

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
  {#if $currentLine.length <= 1}
    <div class="move-list__empty">
      <p>Make a move on the board</p>
    </div>
  {:else}
    {#each $currentLine as move, index}
      {#if index > 0}
        <button
          class="move-list__row"
          class:move-list__row--active={$explorerIndex === index}
          on:click={() => setExplorerIndex(index)}
          use:autoScroll={$explorerIndex === index}
        >
          <div class="move-list__info">
            <span class="move-list__ply">
              {Math.ceil(move.ply / 2)}{move.ply % 2 !== 0 ? '.' : '...'}
            </span>
            <span class="move-list__san">
              <Figurine san={move.san} />
            </span>
          </div>

          {#if move.source === 'variation'}
            <div class="move-list__meta move-list__meta--variation">
              <span class="variation-indicator">~</span>
            </div>
          {/if}
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
    flex-direction: column;
    align-items: center;
    justify-content: center;
    color: #555;
    font-size: 0.9rem;
    text-align: center;
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

  .move-list__meta--variation {
    opacity: 0.3;
  }

  .variation-indicator {
    font-family: 'Bebas Neue', sans-serif;
    font-size: 1rem;
    color: #8be1b4;
  }
</style>
