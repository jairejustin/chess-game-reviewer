<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import Cpu from 'lucide-svelte/icons/cpu';

  import {
    selectedGame,
    fetchedProfile,
    processedGameId
  } from '$lib/stores/fetchStore';
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

  import AppLayout from '$lib/components/ui/AppLayout.svelte';
  import TabbedSidebar from '$lib/components/ui/TabbedSidebar.svelte';
  import ChessBoard from '$lib/components/board/ChessBoard.svelte';
  import NavigationControls from '$lib/components/ui/NavigationControls.svelte';
  import FetchGames from '$lib/components/import/FetchGames.svelte';
  import EngineFeedback from '$lib/components/analysis/EngineFeedback.svelte';
  import MoveList from '$lib/components/ui/MoveList.svelte';
  import GameSummary from '$lib/components/analysis/GameSummary.svelte';
  import AnalysisLoading from '$lib/components/ui/AnalysisLoading.svelte';
  import EngineSettings from '$lib/components/ui/EngineSettings.svelte';

  const reviewTabs = [
    { id: 'import', label: 'Import' },
    { id: 'game', label: 'Game' },
    { id: 'summary', label: 'Summary' },
    { id: 'engine', label: 'Engine' }
  ];

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

  $: if ($selectedGame && $selectedGame.id !== $processedGameId) {
    $processedGameId = $selectedGame.id;

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

<AppLayout>
  <svelte:fragment slot="board">
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
      fen={$moves[$activePly]?.fen ?? 'start'}
      currentMove={$moves[$activePly]}
    />
  </svelte:fragment>

  <svelte:fragment slot="sidebar">
    <TabbedSidebar
      title="Game Analysis"
      tabs={reviewTabs}
      bind:activeTab={$sidebarView}
    >
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

          <MoveList
            moves={$moves}
            activeIndex={$activePly}
            onSelect={(i) => activePly.set(i)}
            showBadges={true}
            emptyMessage="No moves analyzed yet."
          />

          <div class="sidebar__controls">
            <NavigationControls
              canGoBack={$activePly > 0}
              canGoForward={$activePly < $moves.length - 1}
              onBack={() => activePly.update((p) => p - 1)}
              onForward={() => activePly.update((p) => p + 1)}
            />
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
      {:else if $sidebarView === 'engine'}
        <EngineSettings />
      {/if}
    </TabbedSidebar>
  </svelte:fragment>
</AppLayout>

<style>
  .sidebar__controls {
    padding: 0.75rem 1rem;
    background: var(--bg-surface, #1c1c1f);
    border-top: 1px solid var(--border-subtle, #2a2a2e);
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    margin-top: auto;
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
  }
  .analyze-preview-btn:hover {
    background: #234737;
    border-color: #3b7359;
  }
</style>
