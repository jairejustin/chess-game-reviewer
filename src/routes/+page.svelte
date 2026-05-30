<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { chessground } from '../actions/chessground';
  import {
    moves,
    activePly,
    currentFen,
    initTauriListeners,
    isAnalyzing,
    loadingProgress,
    sidebarView,
  } from '../store/gameStore';
  import { getInBoardBadge, badgeColors } from '../utils/boardBadges';

  import EvalBar from '../components/EvalBar.svelte';
  import BoardControls from '../components/BoardControls.svelte';
  import EngineFeedback from '../components/EngineFeedback.svelte';
  import MoveList from '../components/MoveList.svelte';
  import GameSummary from '../components/GameSummary.svelte';

  import Cpu from 'lucide-svelte/icons/cpu';
  import Loader2 from 'lucide-svelte/icons/loader-2';

  import '@lichess-org/chessground/assets/chessground.base.css';
  import '@lichess-org/chessground/assets/chessground.brown.css';
  import '@lichess-org/chessground/assets/chessground.cburnett.css';
  // @ts-ignore
  import '@fontsource/bebas-neue';
  // @ts-ignore
  import '@fontsource-variable/outfit';

  import { calculateMaterial } from '../utils/material';
  import MaterialStrip from '../components/MaterialStrip.svelte';

  $: material = calculateMaterial($currentFen || 'start');

  let destHighlight = 'rgba(155, 199, 0, 0.41)';

  let cgConfig: any = { fen: 'start', viewOnly: true };

  $: {
    const move = $moves[$activePly];
    let autoShapes = [];
    let lastMove: string[] = [];
    destHighlight = 'rgba(155, 199, 0, 0.41)'; // Reset on ply change

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

  async function runAnalysis() {
    const pgn = `[Event "Test"]\n[White "Player1"]\n[Black "Player2"]\n[Result "1-0"]\n\n1. e4 e5 2. Nf3 Nc6 3. Bc4 Bc5 4. b4 Bxb4 5. c3 Ba5 6. d4 exd4 7. O-O d3 8. Qb3 Qf6 9. e5 Qg6 10. Re1 Nge7 11. Ba3 b5 12. Qxb5 Rb8 13. Qa4 Bb6 14. Nbd2 Bb7 15. Ne4 Qf5 16. Bxd3 Qh5 17. Nf6+ gxf6 18. exf6 Rg8 19. Rad1 Qxf3 20. Rxe7+ Nxe7 21. Qxd7+ Kxd7 22. Bf5+ Ke8 23. Bd7+ Kf8 24. Bxe7# 1-0`;
    try {
      await invoke('analyze_game', { pgn });
      $sidebarView = 'game'; // Switch to game view automatically when analysis starts
    } catch (e) {
      console.error('Analysis failed:', e);
    }
  }

  onMount(() => {
    initTauriListeners().catch(console.error);
  });
</script>

<main class="layout">
  <section class="layout__board">
    <div class="board-layout-grid">
      <div class="grid-top-profile player-profile">
        <div class="player-profile__avatar">
          <img
            src="https://ui-avatars.com/api/?name=Opponent&background=232326&color=ececec"
            alt="Opponent Avatar"
          />
        </div>
        <div class="player-profile__info">
          <span class="player-profile__name">Opponent</span>
          <MaterialStrip
            capturedPieces={material.blackCaptured}
            advantage={material.blackAdvantage}
          />
        </div>
      </div>

      <div class="grid-eval">
        <EvalBar />
      </div>

      <div class="grid-board">
        <div class="board-frame">
          <div
            class="board"
            style="--move-highlight: {destHighlight};"
            use:chessground={cgConfig}
          ></div>
        </div>
      </div>

      <div class="grid-bottom-profile player-profile">
        <div class="player-profile__avatar">
          <img
            src="https://ui-avatars.com/api/?name=Player&background=232326&color=ececec"
            alt="Player Avatar"
          />
        </div>
        <div class="player-profile__info">
          <span class="player-profile__name">Player</span>
          <MaterialStrip
            capturedPieces={material.whiteCaptured}
            advantage={material.whiteAdvantage}
          />
        </div>
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
      <div class="import-tab">
        <h3 class="import-tab__title">Fetch Online Games</h3>
        <p class="import-tab__desc">
          Connect your account to analyze recent games from Lichess or
          Chess.com.
        </p>

        <div class="import-tab__form">
          <input
            type="text"
            class="import-tab__input"
            placeholder="Lichess Username"
            disabled
          />
          <button class="import-tab__btn" disabled>Fetch Games</button>
        </div>

        <div class="import-tab__divider"><span>OR</span></div>

        <p class="import-tab__desc">
          Run an engine analysis on the hardcoded test PGN:
        </p>
        <button
          class="analyze-btn"
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
    {:else if $sidebarView === 'game'}
      {#if $isAnalyzing}
        <div class="loading-overlay">
          <div class="loading-overlay__icon-wrapper">
            <Loader2 size={48} class="spin" strokeWidth={2} />
          </div>
          <h3 class="loading-overlay__title">Analyzing...</h3>
          <p class="loading-overlay__desc">The engine is evaluating</p>
          
          <div class="progress-track">
            <div class="progress-fill" style="width: {$loadingProgress * 100}%"></div>
          </div>
          <span class="progress-text">{Math.round($loadingProgress * 100)}%</span>
        </div>
      {:else}
        <EngineFeedback />
        <MoveList />
        <div class="sidebar__controls">
          <BoardControls />
        </div>
      {/if}
    {:else if $sidebarView === 'summary'}
      <GameSummary />
    {/if}
  </aside>
</main>

<style>
  /* ── Global ──────────────────────────────────────────────────────── */
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

  /* ── Layout ──────────────────────────────────────────────────────── */
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
    align-items: flex-start; /* Aligns contents flush with the top */
    height: 100%;
    min-height: 0;
  }

  /* ── Board Grid Structural Framework ───────────────────────────── */
  .board-layout-grid {
    display: grid;
    grid-template-columns: max-content max-content;
    grid-template-rows: max-content minmax(0, 1fr) max-content;
    gap: 0 16px; /* 16px gap between eval bar and board */
    height: 100%;
    max-height: 100%;
  }

  .grid-top-profile {
    grid-column: 2;
    grid-row: 1;
    margin-bottom: 8px;
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
    aspect-ratio: 1 / 1; /* Retains perfect square based on height */
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

  /* ── Player Profiles ─────────────────────────────────────────────── */
  .grid-bottom-profile {
    grid-column: 2;
    grid-row: 3;
    margin-top: 8px;
  }

  .player-profile {
    display: flex;
    align-items: center;
    gap: 12px;
    height: 40px;
    padding: 0 2px;
    width: 100%; /* Will exactly match the dynamic board width */
    box-sizing: border-box;
    flex-shrink: 0;
  }

  .player-profile__avatar {
    width: 3rem;
    height: 3rem;
    border-radius: 6px;
    overflow: hidden;
    background: #232326;
    flex-shrink: 0;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
  }

  .player-profile__avatar img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }

  .player-profile__info {
    display: flex;
    flex-direction: column;
    justify-content: center;
    gap: 2px;
    min-width: 0;
  }

  .player-profile__name {
    font-weight: 700;
    font-size: 1rem;
    color: #ececec;
    line-height: 1;
  }

  /* ── Sidebar Framework ───────────────────────────────────────────── */
  .sidebar {
    width: 360px;
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

  /* ── Nav switcher ────────────────────────────────────────────────── */
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

  /* ── Game Tab specific ───────────────────────────────────────────── */
  .sidebar__controls {
    padding: 0.2rem;
    background: #1c1c1f;
    border-top: 1px solid #2a2a2e;
    flex-shrink: 0;
  }

  /* ── Import Tab specific ─────────────────────────────────────────── */
  .import-tab {
    padding: 1.5rem 1.25rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
    flex: 1;
    overflow-y: auto;
  }
  .import-tab__title {
    font-family: 'Bebas Neue', sans-serif;
    font-size: 1.3rem;
    letter-spacing: 1px;
    color: #ececec;
    margin: 0;
  }
  .import-tab__desc {
    font-size: 0.9rem;
    color: #888;
    margin: 0;
    line-height: 1.4;
  }
  .import-tab__form {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    margin-top: 0.5rem;
  }
  .import-tab__input {
    background: #111;
    border: 1px solid #333;
    padding: 0.8rem;
    border-radius: 6px;
    color: #ececec;
    font-family: 'Outfit', sans-serif;
    outline: none;
    transition: border-color 0.2s;
  }
  .import-tab__input:focus {
    border-color: #555;
  }
  .import-tab__btn {
    background: #232326;
    color: #fff;
    border: 1px solid #333;
    padding: 0.8rem;
    border-radius: 6px;
    cursor: not-allowed;
    font-family: 'Outfit', sans-serif;
    font-weight: 600;
    opacity: 0.5;
  }
  .import-tab__divider {
    display: flex;
    align-items: center;
    text-align: center;
    color: #555;
    font-size: 0.8rem;
    font-weight: 600;
    margin: 1rem 0;
  }
  .import-tab__divider::before,
  .import-tab__divider::after {
    content: '';
    flex: 1;
    border-bottom: 1px solid #2a2a2e;
  }
  .import-tab__divider span {
    padding: 0 10px;
  }
  .analyze-btn {
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
    margin-top: 0.5rem;
  }
  .analyze-btn:hover:not(:disabled) {
    background: #234737;
    border-color: #3b7359;
  }
  .analyze-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  /* ── Loading Overlay ─────────────────────────────────────────────── */
  .loading-overlay {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 2rem;
    text-align: center;
  }
  
  .loading-overlay__icon-wrapper {
    color: #8be1b4;
    margin-bottom: 2rem;
  }
  
  .loading-overlay__title {
    font-family: 'Bebas Neue', sans-serif;
    font-size: 1.5rem;
    letter-spacing: 1px;
    color: #ececec;
    margin: 0 0 0.5rem 0;
  }
  
  .loading-overlay__desc {
    font-size: 0.9rem;
    color: #888;
    margin: 0 0 2rem 0;
  }
  
  .progress-track {
    width: 80%;
    height: 6px;
    background: #111;
    border: 1px solid #333;
    border-radius: 4px;
    overflow: hidden;
    margin-bottom: 0.5rem;
  }
  
  .progress-fill {
    height: 100%;
    background: #8be1b4;
    transition: width 0.15s ease-out;
  }
  
  .progress-text {
    font-family: 'Bebas Neue', sans-serif;
    font-size: 1.2rem;
    color: #8be1b4;
    letter-spacing: 1px;
  }
</style>
