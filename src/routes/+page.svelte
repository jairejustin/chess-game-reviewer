<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import Cpu from 'lucide-svelte/icons/cpu';

  import { chessground } from '../actions/chessground';
  import {
    moves,
    activePly,
    currentFen,
    initTauriListeners,
    isAnalyzing,
    loadingProgress,
    sidebarView,
    analysisSummary,
    isFlipped
  } from '../store/gameStore';
  import { selectedGame, fetchedProfile } from '../store/fetchStore';
  import { getInBoardBadge, badgeColors } from '../utils/boardBadges';

  import EvalBar from '../components/EvalBar.svelte';
  import BoardControls from '../components/BoardControls.svelte';
  import EngineFeedback from '../components/EngineFeedback.svelte';
  import MoveList from '../components/MoveList.svelte';
  import GameSummary from '../components/GameSummary.svelte';
  import FetchGames from '../components/FetchGames.svelte';
  import PlayerProfile from '../components/PlayerProfile.svelte';
  import AnalysisLoading from '../components/AnalysisLoading.svelte';

  import '@lichess-org/chessground/assets/chessground.base.css';
  import '@lichess-org/chessground/assets/chessground.brown.css';
  import '@lichess-org/chessground/assets/chessground.cburnett.css';

  // @ts-ignore
  import '@fontsource/bebas-neue';
  // @ts-ignore
  import '@fontsource-variable/outfit';

  import { calculateMaterial } from '../utils/material';
  $: material = calculateMaterial($currentFen || 'start');

  let destHighlight = 'rgba(155, 199, 0, 0.41)';
  let cgConfig: any = { fen: 'start', viewOnly: true };
  let opponentProfile: any = null;

  // Helper to load preview moves sequence
  async function loadPreview(pgn: string) {
    try {
      const previewMoves: any[] = await invoke('parse_pgn', { pgn });

      $analysisSummary = null; // Flush out older engine state reports

      $moves = [
        {
          ply: 0,
          san: '',
          fen: 'start',
          uci: ''
        },
        ...previewMoves
      ];

      $activePly = $moves.length - 1;
      $sidebarView = 'game';
    } catch (err) {
      console.error('Failed to parse PGN payload from backend:', err);
    }
  }

  // Handle dynamic side re-orientations and trigger background preview fetching
  let processedGameId: string | null = null;
  $: if ($selectedGame && $selectedGame.id !== processedGameId) {
    processedGameId = $selectedGame.id;

    if ($fetchedProfile) {
      const userLower = $fetchedProfile.username.toLowerCase();
      const blackLower = $selectedGame.black.username.toLowerCase();
      $isFlipped = blackLower === userLower;

      // Lazy load opponent data
      const opponentName = $isFlipped
        ? $selectedGame.white.username
        : $selectedGame.black.username;
      invoke('get_player_profile', { username: opponentName })
        .then((profile) => {
          opponentProfile = profile;
        })
        .catch((err) => {
          console.error('Failed to lazy load opponent context profile:', err);
          opponentProfile = null;
        });
    } else {
      $isFlipped = false;
      opponentProfile = null;
    }

    loadPreview($selectedGame.pgn);
  }

  // Core Profile Base Metadata Properties Setup
  $: blackName =
    $analysisSummary?.metadata.black ??
    $selectedGame?.black.username ??
    'Opponent';
  $: whiteName =
    $analysisSummary?.metadata.white ??
    $selectedGame?.white.username ??
    'Player';

  $: blackTitle = ($fetchedProfile && blackName.toLowerCase() === $fetchedProfile.username.toLowerCase()) 
    ? $fetchedProfile.title 
    : opponentProfile?.title;

  $: whiteTitle = ($fetchedProfile && whiteName.toLowerCase() === $fetchedProfile.username.toLowerCase()) 
    ? $fetchedProfile.title 
    : opponentProfile?.title;

  $: blackRating = $selectedGame?.black.rating ?? null;
  $: whiteRating = $selectedGame?.white.rating ?? null;

  $: blackAvatar =
    $fetchedProfile &&
    blackName.toLowerCase() === $fetchedProfile.username.toLowerCase()
      ? $fetchedProfile.avatarUrl
      : opponentProfile?.avatarUrl;

  $: whiteAvatar =
    $fetchedProfile &&
    whiteName.toLowerCase() === $fetchedProfile.username.toLowerCase()
      ? $fetchedProfile.avatarUrl
      : opponentProfile?.avatarUrl;

  // Map values across layout rows dynamically tracking active $isFlipped configuration
  $: topName = $isFlipped ? whiteName : blackName;
  $: bottomName = $isFlipped ? blackName : whiteName;

  $: topRating = $isFlipped ? whiteRating : blackRating;
  $: bottomRating = $isFlipped ? blackRating : whiteRating;

  $: topAvatar = $isFlipped ? whiteAvatar : blackAvatar;
  $: bottomAvatar = $isFlipped ? blackAvatar : whiteAvatar;

  $: topCaptured = $isFlipped ? material.whiteCaptured : material.blackCaptured;
  $: bottomCaptured = $isFlipped
    ? material.blackCaptured
    : material.whiteCaptured;

  $: topAdvantage = $isFlipped
    ? material.whiteAdvantage
    : material.blackAdvantage;
  $: bottomAdvantage = $isFlipped
    ? material.blackAdvantage
    : material.whiteAdvantage;

  $: topTitle = $isFlipped ? whiteTitle : blackTitle;
  $: bottomTitle = $isFlipped ? blackTitle : whiteTitle;

  // Update Chessground layout configuration settings engine parameters
  $: {
    const move = $moves[$activePly];
    let autoShapes: any[] = [];
    let lastMove: string[] = [];
    destHighlight = 'rgba(155, 199, 0, 0.41)';

    if (
      move &&
      move.ply > 0 &&
      typeof move.uci === 'string' &&
      move.uci.length >= 4
    ) {
      const orig = move.uci.substring(0, 2);
      const dest = move.uci.substring(2, 4);
      lastMove = [orig, dest];

      if (move.classification) {
        destHighlight = badgeColors[move.classification] + '66';
        autoShapes.push({
          orig: dest,
          brush: 'invisible',
          customSvg: { html: getInBoardBadge(move.classification) }
        });
      }
    }

    cgConfig = {
      fen: $currentFen || 'start',
      orientation: $isFlipped ? 'black' : 'white',
      viewOnly: true,
      lastMove,
      drawable: {
        brushes: {
          invisible: {
            key: 'i',
            color: 'transparent',
            opacity: 0,
            lineWidth: 1
          }
        },
        autoShapes,
        visible: true
      }
    };
  }

  async function runAnalysis(pgn: string) {
    try {
      await invoke('analyze_game', { pgn });
      $sidebarView = 'game';
    } catch (e) {
      console.error('Analysis runtime tracking breakdown error:', e);
    }
  }

  onMount(() => {
    initTauriListeners().catch(console.error);
  });
</script>

<main class="layout">
  <section class="layout__board">
    <div class="board-layout-grid">
      <div class="grid-top-profile">
        <PlayerProfile
          name={topName}
          title={topTitle}
          rating={topRating}
          avatarUrl={topAvatar}
          capturedPieces={topCaptured}
          advantage={topAdvantage}
        />
      </div>

      <div class="grid-eval"><EvalBar /></div>

      <div class="grid-board">
        <div class="board-frame">
          <div
            class="board"
            style="--move-highlight: {destHighlight};"
            use:chessground={cgConfig}
          ></div>
        </div>
      </div>

      <div class="grid-bottom-profile">
        <PlayerProfile
          name={bottomName}
          title={bottomTitle}
          rating={bottomRating}
          avatarUrl={bottomAvatar}
          capturedPieces={bottomCaptured}
          advantage={bottomAdvantage}
        />
      </div>
    </div>
  </section>

  <aside class="sidebar">
    <div class="sidebar__header">
      <h2 class="sidebar__title">Game Analysis</h2>
    </div>

    <div class="sidebar__nav">
      <button
        class="sidebar__nav-btn"
        class:sidebar__nav-btn--active={$sidebarView === 'import'}
        on:click={() => ($sidebarView = 'import')}>Import</button
      >
      <button
        class="sidebar__nav-btn"
        class:sidebar__nav-btn--active={$sidebarView === 'game'}
        on:click={() => ($sidebarView = 'game')}>Game</button
      >
      <button
        class="sidebar__nav-btn"
        class:sidebar__nav-btn--active={$sidebarView === 'summary'}
        on:click={() => ($sidebarView = 'summary')}>Summary</button
      >
    </div>

    {#if $sidebarView === 'import'}
      <FetchGames />
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
  /* ── Global Styles ────────────────────────────────────────────────── */
  :global(body) {
    background-color: #0f0f11;
    color: #ececec;
    font-family: 'Outfit', system-ui, sans-serif;
    margin: 0;
  }

  :global(.cg-wrap square.last-move) {
    background-color: var(--move-highlight) !important;
  }

  :global(.badge-anim) {
    animation: badge-pop-in 0.1s cubic-bezier(0.175, 0.885, 0.32, 1.275) 0.1s
      both;
  }

  :global(.spin) {
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  @keyframes badge-pop-in {
    0% {
      opacity: 0;
      transform: scale(0.5);
    }
    100% {
      opacity: 1;
      transform: scale(1);
    }
  }

  /* ── Layout Framework ────────────────────────────────────────────── */
  .layout {
    display: flex;
    height: 100vh;
    width: 100vw;
    max-width: 100%;
    margin: 0;
    padding: 1rem;
    gap: 1.5rem;
    box-sizing: border-box;
    overflow: hidden;
  }

  .layout__board {
    flex: 1;
    display: flex;
    justify-content: center;
    align-items: flex-start;
    height: 100%;
    min-height: 0;
  }

  /* ── Board Layout Grid Structural Alignment ─────────────────────── */
  .board-layout-grid {
    display: grid;
    grid-template-columns: max-content max-content;
    grid-template-rows: max-content minmax(0, 1fr) max-content;
    gap: 0 16px;
    height: 100%;
    max-height: 100%;
  }

  .grid-top-profile {
    grid-column: 2;
    grid-row: 1;
    margin-bottom: 8px;
    width: 100%;
  }

  .grid-eval {
    grid-column: 1;
    grid-row: 2;
    height: 100%;
  }

  .grid-board {
    grid-column: 2;
    grid-row: 2;
    height: 100%;
    display: flex;
  }

  .board-frame {
    height: 100%;
    aspect-ratio: 1 / 1;
    position: relative;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.5);
    border-radius: 4px;
    overflow: hidden;
    flex-shrink: 0;
  }

  .board {
    width: 100%;
    height: 100%;
    position: relative;
    user-select: none;
  }

  .grid-bottom-profile {
    grid-column: 2;
    grid-row: 3;
    margin-top: 8px;
    width: 100%;
  }

  /* ── Sidebar Component Layout ────────────────────────────────────── */
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
    padding: 0.2rem;
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
