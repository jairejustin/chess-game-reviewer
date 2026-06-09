<script context="module" lang="ts">
  // Hoisting this here keeps it alive across page navigations.
  // This prevents loadPreview from wiping the analysis when returning from Explorer!
  let processedGameId: string | null = null;
</script>

<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import Cpu from 'lucide-svelte/icons/cpu';

  // Stores
  import { selectedGame, fetchedProfile } from '$lib/stores/fetchStore';
  import { moves, activePly, isFlipped } from '$lib/stores/boardStore';
  import {
    sidebarView,
    isAnalyzing,
    loadingProgress,
    analysisSummary,
    currentEval,
    currentMateIn,
    initAnalysisListeners
  } from '$lib/stores/reviewStore';

  // Components
  import ChessBoard from '$lib/components/board/ChessBoard.svelte';
  import BoardControls from '$lib/components/board/BoardControls.svelte';
  import FetchGames from '$lib/components/import/FetchGames.svelte';
  import EngineFeedback from '$lib/components/analysis/EngineFeedback.svelte';
  import MoveList from '$lib/components/analysis/MoveList.svelte';
  import GameSummary from '$lib/components/analysis/GameSummary.svelte';
  import AnalysisLoading from '$lib/components/ui/AnalysisLoading.svelte';
  import ActionStrip from '$lib/components/ui/ActionStrip.svelte';

  let opponentProfile: any = null;

  async function loadPreview(pgn: string) {
    try {
      const previewMoves: any[] = await invoke('parse_pgn', { pgn });
      $analysisSummary = null;
      $moves = [
        { ply: 0, san: '', fen: 'start', uci: '', source: 'game' },
        ...previewMoves.map((m) => ({ ...m, source: 'game' }))
      ];
      $activePly = $moves.length - 1;
      $sidebarView = 'game';
    } catch (err) {
      console.error('Failed to parse PGN payload from backend:', err);
    }
  }

  $: if ($selectedGame && $selectedGame.id !== processedGameId) {
    processedGameId = $selectedGame.id;

    if ($fetchedProfile) {
      const userLower = $fetchedProfile.username.toLowerCase();
      const blackLower = $selectedGame.black.username.toLowerCase();
      $isFlipped = blackLower === userLower;

      const opponentName = $isFlipped
        ? $selectedGame.white.username
        : $selectedGame.black.username;

      invoke('get_player_profile', { username: opponentName })
        .then((profile) => {
          opponentProfile = profile;
        })
        .catch((err) => {
          console.error('Failed to load opponent context profile:', err);
          opponentProfile = null;
        });
    } else {
      $isFlipped = false;
      opponentProfile = null;
    }

    loadPreview($selectedGame.pgn);
  }

  $: whiteName =
    $analysisSummary?.metadata.white ??
    $selectedGame?.white.username ??
    'White';
  $: blackName =
    $analysisSummary?.metadata.black ??
    $selectedGame?.black.username ??
    'Black';

  $: whiteAvatar =
    $fetchedProfile &&
    whiteName.toLowerCase() === $fetchedProfile.username.toLowerCase()
      ? $fetchedProfile.avatarUrl
      : opponentProfile?.avatarUrl;

  $: blackAvatar =
    $fetchedProfile &&
    blackName.toLowerCase() === $fetchedProfile.username.toLowerCase()
      ? $fetchedProfile.avatarUrl
      : opponentProfile?.avatarUrl;

  $: whiteTitle =
    $fetchedProfile &&
    whiteName.toLowerCase() === $fetchedProfile.username.toLowerCase()
      ? $fetchedProfile.title
      : opponentProfile?.title;

  $: blackTitle =
    $fetchedProfile &&
    blackName.toLowerCase() === $fetchedProfile.username.toLowerCase()
      ? $fetchedProfile.title
      : opponentProfile?.title;

  async function runAnalysis(pgn: string) {
    try {
      await invoke('analyze_game', { pgn });
      $sidebarView = 'game';
    } catch (e) {
      console.error('Analysis runtime tracking breakdown error:', e);
    }
  }

  onMount(() => {
    initAnalysisListeners().catch(console.error);
  });
</script>

<main class="layout">
  <section class="layout__board">
    <ChessBoard
      {whiteName}
      {blackName}
      whiteRating={$selectedGame?.white.rating ?? null}
      blackRating={$selectedGame?.black.rating ?? null}
      {whiteTitle}
      {blackTitle}
      {whiteAvatar}
      {blackAvatar}
      evalCp={$currentEval}
      evalMateIn={$currentMateIn}
      evalActive={!!$analysisSummary}
    />
  </section>

  <ActionStrip />

  <aside class="sidebar">
    <div class="sidebar__header">
      <h2 class="sidebar__title">Game Analysis</h2>
    </div>

    <div class="sidebar__nav">
      <button
        class="sidebar__nav-btn"
        class:sidebar__nav-btn--active={$sidebarView === 'import'}
        on:click={() => ($sidebarView = 'import')}
      >
        Import
      </button>
      <button
        class="sidebar__nav-btn"
        class:sidebar__nav-btn--active={$sidebarView === 'game'}
        on:click={() => ($sidebarView = 'game')}
      >
        Game
      </button>
      <button
        class="sidebar__nav-btn"
        class:sidebar__nav-btn--active={$sidebarView === 'summary'}
        on:click={() => ($sidebarView = 'summary')}
      >
        Summary
      </button>
    </div>

    {#if $sidebarView === 'import'}
      <FetchGames />
      {#if !$analysisSummary && $selectedGame}
        <div class="sidebar__controls">
          <button
            class="analyze-preview-btn"
            on:click={() => runAnalysis($selectedGame!.pgn)}
          >
            <Cpu size={18} strokeWidth={3} />
            Analyze Game
          </button>
        </div>
      {/if}
    {:else if $sidebarView === 'game'}
      {#if $isAnalyzing}
        <AnalysisLoading progress={$loadingProgress} />
      {:else}
        <EngineFeedback />
        <MoveList />
        <div class="sidebar__controls">
          <BoardControls />
          {#if !$analysisSummary && $selectedGame}
            <button
              class="analyze-preview-btn"
              on:click={() => runAnalysis($selectedGame!.pgn)}
            >
              <Cpu size={18} strokeWidth={3} />
              Analyze Game
            </button>
          {/if}
        </div>
      {/if}
    {:else if $sidebarView === 'summary'}
      <GameSummary />
    {/if}
  </aside>
</main>

<style>
  /* All your existing CSS goes here unaltered... */
  .layout {
    display: flex;
    height: 100vh;
    width: 100vw;
    max-width: 100%;
    margin: 0;
    padding: 1rem;
    gap: 0;
    box-sizing: border-box;
    overflow: hidden;
    align-items: flex-start;
  }

  .layout__board {
    flex: 1;
    display: flex;
    justify-content: center;
    align-items: center;
    height: 100%;
    min-height: 0;
    padding: 4rem 1rem 4rem 4rem;
    box-sizing: border-box;
  }

  .sidebar {
    width: 360px;
    height: 100%;
    max-height: 100%;
    flex-shrink: 0;
    background: #161618;
    border: 1px solid #2a2a2e;
    border-radius: 12px;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    box-shadow: 0 10px 30px rgba(0, 0, 0, 0.3);
  }

  .sidebar__header {
    padding: 1.1rem 1.25rem 1rem;
    background: #1c1c1f;
    border-bottom: 1px solid #2a2a2e;
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    flex-shrink: 0;
  }

  .sidebar__title {
    font-family: 'Bebas Neue', sans-serif;
    font-size: 1.8rem;
    font-weight: 400;
    margin: 0;
    letter-spacing: 1px;
    color: #fff;
  }

  .sidebar__nav {
    display: flex;
    flex-shrink: 0;
    border-bottom: 1px solid #2a2a2e;
    background: #1c1c1f;
  }

  .sidebar__nav-btn {
    flex: 1;
    background: transparent;
    border: none;
    border-bottom: 2px solid transparent;
    color: #555;
    font-family: 'Bebas Neue', sans-serif;
    font-size: 1rem;
    font-weight: 400;
    letter-spacing: 2px;
    text-transform: uppercase;
    padding: 0.4rem 1rem;
    cursor: pointer;
    transition:
      color 0.15s ease,
      border-color 0.15s ease;
    margin-bottom: -1px;
  }

  .sidebar__nav-btn:hover:not(.sidebar__nav-btn--active) {
    color: #888;
  }

  .sidebar__nav-btn--active {
    color: #ececec;
    border-bottom-color: #ececec;
  }

  .sidebar__controls {
    padding: 0.75rem 1rem;
    background: #1c1c1f;
    border-top: 1px solid #2a2a2e;
    flex-shrink: 0;
  }

  .analyze-preview-btn {
    background: #1b382b;
    border: 1px solid #2b5743;
    color: #8be1b4;
    padding: 0.8rem 1.2rem;
    border-radius: 8px;
    cursor: pointer;
    font-family: 'Outfit', sans-serif;
    font-weight: 600;
    font-size: 1rem;
    transition: all 0.2s ease;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    width: 100%;
    margin-top: 0.75rem;
  }

  .analyze-preview-btn:hover {
    background: #234737;
    border-color: #3b7359;
  }
</style>
