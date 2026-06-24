<script lang="ts">
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import { invoke } from '@tauri-apps/api/core';

  import {
    isAnalyzing,
    analysisSummary,
    sidebarView
  } from '$lib/stores/reviewStore';
  import { selectedGame } from '$lib/stores/fetchStore';
  import { activePly, isFlipped } from '$lib/stores/boardStore';

  import Microscope from 'lucide-svelte/icons/microscope';
  import LayoutDashboard from 'lucide-svelte/icons/layout-dashboard';
  import Settings from 'lucide-svelte/icons/settings';
  import Play from 'lucide-svelte/icons/play';
  import CircleStar from 'lucide-svelte/icons/circle-star';
  import ArrowUpDown from 'lucide-svelte/icons/arrow-up-down';

  $: isExplorer = page.url.pathname.startsWith('/explorer');

  function toggleExplorer() {
    if (isExplorer) {
      goto('/');
    } else {
      goto('/explorer');
    }
  }

  function toggleFlip() {
    $isFlipped = !$isFlipped;
  }

  $: explorerTitle = $isAnalyzing
    ? 'Cannot open Explorer while analyzing'
    : isExplorer
      ? 'Back to Game Review'
      : 'Open Live Explorer';

  async function runAnalysis() {
    if (!$selectedGame) return;
    try {
      await invoke('analyze_game', { pgn: $selectedGame.pgn });
      $sidebarView = 'game';
    } catch (e) {
      console.error('Analysis runtime tracking breakdown error:', e);
    }
  }

  function startReview() {
    $activePly = 0;
    $sidebarView = 'game';
  }
</script>

<div class="action-strip">
  <button
    class="action-strip__btn"
    class:action-strip__btn--active={isExplorer}
    on:click={toggleExplorer}
    disabled={$isAnalyzing && !isExplorer}
    title={explorerTitle}
  >
    {#if isExplorer}
      <LayoutDashboard size={30} strokeWidth={2} />
    {:else}
      <Microscope size={30} strokeWidth={2} />
    {/if}
  </button>

  <button
    class="action-strip__btn"
    class:action-strip__btn--active={$isAnalyzing}
    on:click={runAnalysis}
    disabled={$isAnalyzing ||
      !$selectedGame ||
      !!$analysisSummary ||
      isExplorer}
    title="Game Review"
  >
    <CircleStar size={30} strokeWidth={2} />
  </button>

  <div class="action-strip__divider"></div>

  <button
    class="action-strip__btn"
    on:click={startReview}
    disabled={!$analysisSummary || isExplorer}
    title="Start"
  >
    <Play size={30} strokeWidth={2} />
  </button>

  <button
    class="action-strip__btn"
    on:click={toggleFlip}
    title="Flip board (F)"
  >
    <ArrowUpDown size={28} strokeWidth={2} />
  </button>

  <div class="action-strip__divider"></div>

  <button class="action-strip__btn" title="Settings" disabled>
    <Settings size={30} strokeWidth={2} />
  </button>
</div>

<style>
  .action-strip {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
    padding: 8px 8px;
    flex-shrink: 0;
  }
  .action-strip__btn {
    width: 2.5rem;
    height: 2.5rem;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: 1px solid transparent;
    border-radius: 8px;
    color: #fff;
    cursor: pointer;
    transition:
      color 0.15s ease,
      background 0.15s ease,
      border-color 0.15s ease;
  }
  .action-strip__btn:hover:not(:disabled) {
    color: #ccc;
    background: #1e1e22;
    border-color: #2a2a2e;
  }
  .action-strip__btn--active {
    color: #8be1b4;
    background: #1b382b;
    border-color: #2b5743;
  }
  .action-strip__btn--active:hover {
    background: #234737 !important;
    border-color: #3b7359 !important;
  }
  .action-strip__btn:disabled {
    opacity: 0.3;
    cursor: not-allowed;
  }
  .action-strip__divider {
    width: 2.5rem;
    height: 2px;
    background: #2a2a2e;
    margin: 4px 0;
  }
</style>
