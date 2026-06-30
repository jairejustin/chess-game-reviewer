<script lang="ts">
  import { isFlipped } from '$lib/stores/boardStore';
  import ChevronLeft from 'lucide-svelte/icons/chevron-left';
  import ChevronRight from 'lucide-svelte/icons/chevron-right';

  export let canGoBack: boolean;
  export let canGoForward: boolean;
  export let onBack: () => void;
  export let onForward: () => void;

  function toggleFlip() {
    $isFlipped = !$isFlipped;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'ArrowLeft' && canGoBack) onBack();
    if (e.key === 'ArrowRight' && canGoForward) onForward();
    if (e.key.toLowerCase() === 'f') toggleFlip();
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<div class="controls">
  <button
    class="controls__btn controls__btn--icon"
    on:click={onBack}
    disabled={!canGoBack}
    title="Previous move"
  >
    <ChevronLeft size={22} strokeWidth={3} />
  </button>
  <button
    class="controls__btn controls__btn--icon"
    on:click={onForward}
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
    height: 38px;
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
