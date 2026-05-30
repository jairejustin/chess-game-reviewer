<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { moves, activePly, isAnalyzing } from '../store/gameStore';

  import ChevronLeft from 'lucide-svelte/icons/chevron-left';
  import ChevronRight from 'lucide-svelte/icons/chevron-right';
  import Cpu from 'lucide-svelte/icons/cpu';
  import Loader2 from 'lucide-svelte/icons/loader-2';

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

  async function runAnalysis() {
    const pgn = `[Event "Test"]\n[White "Player1"]\n[Black "Player2"]\n[Result "1-0"]\n\n1. e4 e5 2. Nf3 Nc6 3. Bc4 Bc5 4. b4 Bxb4 5. c3 Ba5 6. d4 exd4 7. O-O d3 8. Qb3 Qf6 9. e5 Qg6 10. Re1 Nge7 11. Ba3 b5 12. Qxb5 Rb8 13. Qa4 Bb6 14. Nbd2 Bb7 15. Ne4 Qf5 16. Bxd3 Qh5 17. Nf6+ gxf6 18. exf6 Rg8 19. Rad1 Qxf3 20. Rxe7+ Nxe7 21. Qxd7+ Kxd7 22. Bf5+ Ke8 23. Bd7+ Kf8 24. Bxe7# 1-0`;
    try {
      await invoke('analyze_game', { pgn });
    } catch (e) {
      console.error('Analysis failed:', e);
    }
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
    <ChevronLeft size={20} strokeWidth={3} />
  </button>
  <button
    class="controls__btn controls__btn--icon"
    on:click={goForward}
    disabled={!canGoForward}
    title="Next move"
  >
    <ChevronRight size={20} strokeWidth={3} />
  </button>
  <button
    class="controls__btn controls__btn--analyze"
    on:click={runAnalysis}
    disabled={$isAnalyzing}
  >
    {#if $isAnalyzing}
      <Loader2 size={20} class="spin" strokeWidth={3} /> Analyzing…
    {:else}
      <Cpu size={20} strokeWidth={3} /> Run Analysis
    {/if}
  </button>
</div>

<style>
  .controls {
    display: flex;
    gap: 0.75rem;
    align-items: center;
  }
  .controls__btn {
    background: #232326;
    color: #fff;
    border: 1px solid #333;
    padding: 0.6rem 1.2rem;
    border-radius: 8px;
    cursor: pointer;
    font-family: 'Outfit', sans-serif;
    font-weight: 600;
    font-size: 1rem;
    transition: all 0.2s ease;
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  .controls__btn:hover:not(:disabled) {
    background: #2e2e33;
    border-color: #444;
  }
  .controls__btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  .controls__btn--icon {
    padding: 0.6rem;
  }
  .controls__btn--analyze {
    background: #1b382b;
    border-color: #2b5743;
    color: #8be1b4;
  }
  .controls__btn--analyze:hover:not(:disabled) {
    background: #234737;
    border-color: #3b7359;
  }
  :global(.spin) {
    animation: spin 1s linear infinite;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
