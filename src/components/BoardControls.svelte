<script lang="ts">
  import { moves, activePly } from '../store/gameStore';
  import ChevronLeft from 'lucide-svelte/icons/chevron-left';
  import ChevronRight from 'lucide-svelte/icons/chevron-right';

  $: canGoBack = $activePly > 0;
  $: canGoForward = $activePly < $moves.length - 1;

  function goBack() {
    if (canGoBack) activePly.update((p) => p - 1);
  }
  function goForward() {
    if (canGoForward) activePly.update((p) => p + 1);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'ArrowLeft') goBack();
    if (e.key === 'ArrowRight') goForward();
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<div class="controls">
  <button
    class="controls__btn controls__btn--icon"
    on:click={goBack}
    disabled={!canGoBack}
    title="Previous move"
  >
    <ChevronLeft size={22} strokeWidth={3} />
  </button>
  <button
    class="controls__btn controls__btn--icon"
    on:click={goForward}
    disabled={!canGoForward}
    title="Next move"
  >
    <ChevronRight size={22} strokeWidth={3} />
  </button>
</div>

<style>
  .controls {
    display: flex;
    gap: 0.75rem;
    align-items: center;
    width: 100%;
  }
  .controls__btn {
    background: #232326;
    color: #fff;
    border: 1px solid #333;
    border-radius: 8px;
    cursor: pointer;
    font-family: 'Outfit', sans-serif;
    transition: all 0.2s ease;
    display: flex;
    align-items: center;
    justify-content: center;
    flex: 1;
  }
  .controls__btn:hover:not(:disabled) {
    background: #2e2e33;
    border-color: #444;
  }
  .controls__btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
</style>
