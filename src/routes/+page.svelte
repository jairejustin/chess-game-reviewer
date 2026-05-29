<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from "@tauri-apps/api/core";
  import { chessground } from "../actions/chessground";
  import {
    moves,
    activePly,
    currentFen,
    currentEval,
    isAnalyzing,
    analysisSummary,
    initTauriListeners,
    currentMateIn
  } from "../store/gameStore";
  import type { MoveBadge, MoveCounts } from '../types/game';

  import "@lichess-org/chessground/assets/chessground.base.css";
  import "@lichess-org/chessground/assets/chessground.brown.css";
  import "@lichess-org/chessground/assets/chessground.cburnett.css";
  // @ts-ignore
  import "@fontsource/bebas-neue";
  // @ts-ignore
  import "@fontsource-variable/outfit";

  import {
    StarIcon,
    ThumbsUpIcon,
    BookBookmarkIcon,
    CaretLeftIcon,
    CaretRightIcon,
    CpuIcon,
    CircleNotchIcon
  } from 'phosphor-svelte';

  // ---------------------------------------------------------------------------
  // Types
  // ---------------------------------------------------------------------------

  type SidebarView = 'game' | 'summary';

  // ---------------------------------------------------------------------------
  // Lookup maps  (keyed on MoveBadge from types/game.ts)
  // ---------------------------------------------------------------------------

  const badgeColors: Record<MoveBadge, string> = {
    brilliant:  '#1baca6',
    great:      '#5c8bb0',
    best:       '#95bb4a',
    excellent:  '#96bc4b',
    good:       '#96bc4b',
    inaccuracy: '#f4c153',
    mistake:    '#e6912c',
    blunder:    '#b33430',
    miss:       '#ff7769',
    book:       '#a88865',
    forced:     '#8da6b6',
  };

  const badgeIcons: Record<MoveBadge, any> = {
    brilliant:  '!!',
    great:      '!',
    best:       StarIcon,
    excellent:  ThumbsUpIcon,
    good:       '✓',
    inaccuracy: '?!',
    mistake:    '?',
    blunder:    '??',
    miss:       '✗',
    book:       BookBookmarkIcon,
    forced:     '→',
  };

  const tallyOrder: MoveBadge[] = [
    'brilliant', 'great', 'best', 'excellent', 'good',
    'book', 'forced', 'inaccuracy', 'mistake', 'miss', 'blunder',
  ];

  const tallyLabels: Record<MoveBadge, string> = {
    brilliant:  'Brilliant',
    great:      'Great',
    best:       'Best',
    excellent:  'Excellent',
    good:       'Good',
    book:       'Book',
    forced:     'Forced',
    inaccuracy: 'Inaccuracy',
    mistake:    'Mistake',
    miss:       'Miss',
    blunder:    'Blunder',
  };

  // ---------------------------------------------------------------------------
  // State
  // ---------------------------------------------------------------------------

  let sidebarView: SidebarView = 'game';

  // ---------------------------------------------------------------------------
  // Reactive helpers
  // ---------------------------------------------------------------------------

  $: canGoBack    = $activePly > 0;
  $: canGoForward = $activePly < $moves.length - 1;

  $: currentMove = $moves[$activePly];

  function goBack()    { if (canGoBack)    activePly.update(p => p - 1); }
  function goForward() { if (canGoForward) activePly.update(p => p + 1); }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'ArrowLeft')  goBack();
    if (e.key === 'ArrowRight') goForward();
  }

  function winPercent(cp: number): number {
    return 50 + 50 * (2 / (1 + Math.exp(-0.00368 * cp)) - 1);
  }

  $: whitePercent = winPercent($currentEval);
  $: blackPercent = 100 - whitePercent;

  function formatEval(cp: number, mateIn?: number | null): string {
    if (mateIn != null) {
      if (mateIn === 0 && cp >= 10000) return '+M';
      if (mateIn === 0 && cp <= -10000) return '+M';
      return mateIn > 0 ? `+M${mateIn}` : `-M${Math.abs(mateIn)}`;
    }
    if (cp >= 10000)  return '+M';
    if (cp <= -10000) return '-M';
    const abs = Math.abs(cp / 100).toFixed(2);
    return cp >= 0 ? `+${abs}` : `-${abs}`;
  }
  function formatAccuracy(score: number): string {
    return score.toFixed(1) + '%';
  }

  // Returns the classifications that appear in either player's counts
  function activeTallyRows(w: MoveCounts, b: MoveCounts): MoveBadge[] {
    return tallyOrder.filter(c => (w[c] ?? 0) > 0 || (b[c] ?? 0) > 0);
  }

  // ---------------------------------------------------------------------------
  // Lifecycle
  // ---------------------------------------------------------------------------

  onMount(() => {
    initTauriListeners().catch(console.error);
  });

  async function runAnalysis() {
    const pgn = `[Event "Test"]\n[White "Player1"]\n[Black "Player2"]\n[Result "1-0"]\n\n1. e4 e5 2. Nf3 Nc6 3. Bc4 Bc5 4. b4 Bxb4 5. c3 Ba5 6. d4 exd4 7. O-O d3 8. Qb3 Qf6 9. e5 Qg6 10. Re1 Nge7 11. Ba3 b5 12. Qxb5 Rb8 13. Qa4 Bb6 14. Nbd2 Bb7 15. Ne4 Qf5 16. Bxd3 Qh5 17. Nf6+ gxf6 18. exf6 Rg8 19. Rad1 Qxf3 20. Rxe7+ Nxe7 21. Qxd7+ Kxd7 22. Bf5+ Ke8 23. Bd7+ Kf8 24. Bxe7# 1-0`;
    try {
      await invoke("analyze_game", { pgn });
    } catch (e) {
      console.error("Analysis failed:", e);
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<main class="layout">

  <!-- ── Board column ────────────────────────────────────────────── -->
  <section class="layout__board">

    <div class="board-row">

      <!-- Eval bar -->
      <div class="eval-bar" aria-label="Evaluation bar, {formatEval($currentEval, $currentMateIn)}">
        <div class="eval-bar__track">
          <div class="eval-bar__segment eval-bar__segment--black" style="flex: {blackPercent}"></div>
          <div class="eval-bar__divider"></div>
          <div class="eval-bar__segment eval-bar__segment--white" style="flex: {whitePercent}"></div>
        </div>
        <span
          class="eval-bar__label"
          class:eval-bar__label--winning={$currentEval > 0}
          class:eval-bar__label--losing={$currentEval < 0}
        >
          {formatEval($currentEval, $currentMateIn)}
        </span>
      </div>

      <!-- Chessboard -->
      <div class="board" use:chessground={{ fen: $currentFen, viewOnly: true }}></div>

    </div>

    <!-- Controls -->
    <div class="controls">
      <button class="controls__btn controls__btn--icon" on:click={goBack} disabled={!canGoBack} title="Previous move">
        <CaretLeftIcon size={20} weight="bold" />
      </button>
      <button class="controls__btn controls__btn--icon" on:click={goForward} disabled={!canGoForward} title="Next move">
        <CaretRightIcon size={20} weight="bold" />
      </button>
      <button class="controls__btn controls__btn--analyze" on:click={runAnalysis} disabled={$isAnalyzing}>
        {#if $isAnalyzing}
          <CircleNotchIcon size={20} class="spin" weight="bold" /> Analyzing…
        {:else}
          <CpuIcon size={20} weight="bold" /> Run Analysis
        {/if}
      </button>
    </div>

  </section>

  <!-- ── Sidebar ──────────────────────────────────────────────────── -->
  <aside class="sidebar">

    <!-- Header -->
    <div class="sidebar__header">
      <h2 class="sidebar__title">Game Analysis</h2>
      <span class="sidebar__ply-count">{$moves.length} plies</span>
    </div>

    <!-- View switcher -->
    <div class="sidebar__nav">
      <button
        class="sidebar__nav-btn"
        class:sidebar__nav-btn--active={sidebarView === 'game'}
        on:click={() => sidebarView = 'game'}
      >
        Game
      </button>
      <button
        class="sidebar__nav-btn"
        class:sidebar__nav-btn--active={sidebarView === 'summary'}
        on:click={() => sidebarView = 'summary'}
      >
        Summary
      </button>
    </div>

    <!-- ── Game view ─────────────────────────────────────────────── -->
    {#if sidebarView === 'game'}
      
      {#if currentMove && currentMove.ply > 0}
        <div class="engine-feedback">
          
          <div class="engine-feedback__header">
            <div 
              class="engine-feedback__badge" 
              style="background-color: {badgeColors[currentMove.classification]}"
            >
              {#if typeof badgeIcons[currentMove.classification] === 'string'}
                <span class="engine-feedback__badge-text">{badgeIcons[currentMove.classification]}</span>
              {:else}
                <svelte:component this={badgeIcons[currentMove.classification]} size={16} weight="fill" />
              {/if}
            </div>
            <span class="engine-feedback__title">
              {tallyLabels[currentMove.classification]}
            </span>
          </div>

          <div class="engine-feedback__comparison">
            <div class="engine-line">
              <span class="engine-line__label">Played:</span>
              <span class="engine-line__move">{currentMove.san}</span>
              <span class="engine-line__eval">{formatEval(currentMove.playedEval, currentMove.mateIn)}</span>
            </div>
            
            {#if !['best', 'book', 'forced', 'great', 'brilliant'].includes(currentMove.classification)}
              <div class="engine-line engine-line--best">
                <span class="engine-line__label">Best:</span>
                <span class="engine-line__move">{currentMove.bestMoveSan}</span>
                <span class="engine-line__eval">{formatEval(currentMove.prevBestEval)}</span>
              </div>
            {/if}
          </div>

        </div>
      {/if}

      <div class="move-list">
        {#if $moves.length === 0}
          <div class="move-list__empty">
            {#if $isAnalyzing}
              <CircleNotchIcon size={32} class="spin" weight="bold" color="#5c8bb0" />
              <p>Engine is calculating...</p>
            {:else}
              <p>No moves analyzed yet.</p>
            {/if}
          </div>
        {:else}
          {#each $moves as move, index}
            <button
              class="move-list__row"
              class:move-list__row--active={$activePly === index}
              on:click={() => activePly.set(index)}
            >
              <div class="move-list__info">
                <span class="move-list__ply">
                  {Math.ceil(move.ply / 2)}{move.ply % 2 !== 0 ? '.' : '...'}
                </span>
                <span class="move-list__san">{move.san}</span>
              </div>
              <div class="move-list__meta">
                <span class="move-list__eval">{formatEval(move.playedEval, move.mateIn)}</span>
                <span
                  class="move-list__badge"
                  style="background-color: {badgeColors[move.classification]}"
                  title={move.classification}
                >
                  {#if typeof badgeIcons[move.classification] === 'string'}
                    <span class="move-list__badge-text">{badgeIcons[move.classification]}</span>
                  {:else if badgeIcons[move.classification]}
                    <svelte:component this={badgeIcons[move.classification]} size={14} weight="fill" />
                  {/if}
                </span>
              </div>
            </button>
          {/each}
        {/if}
      </div>

    <!-- ── Summary view ──────────────────────────────────────────── -->
    {:else if sidebarView === 'summary'}
      <div class="summary">

        {#if !$analysisSummary}
          <div class="summary__empty">
            <p>Run analysis to see a summary.</p>
          </div>
        {:else}
          {@const s = $analysisSummary}

          <!-- Accuracy -->
          <div class="summary__section">
            <h3 class="summary__section-title">Accuracy</h3>
            <div class="summary__accuracy-row">
              <div class="summary__player">
                <span class="summary__player-dot summary__player-dot--white"></span>
                <span class="summary__player-name">{s.metadata.white}</span>
              </div>
              <span class="summary__accuracy-score">{formatAccuracy(s.whiteAccuracy)}</span>
            </div>
            <div class="summary__accuracy-row">
              <div class="summary__player">
                <span class="summary__player-dot summary__player-dot--black"></span>
                <span class="summary__player-name">{s.metadata.black}</span>
              </div>
              <span class="summary__accuracy-score">{formatAccuracy(s.blackAccuracy)}</span>
            </div>
          </div>

          <!-- Move tally — white vs black columns -->
          <div class="summary__section">
            <h3 class="summary__section-title">Move Tally</h3>

            <!-- Column headers -->
            <div class="tally__header">
              <span class="tally__header-spacer"></span>
              <div class="tally__header-players">
                <span class="tally__header-player tally__header-player--white">
                  <span class="summary__player-dot summary__player-dot--white"></span>
                </span>
                <span class="tally__header-player tally__header-player--black">
                  <span class="summary__player-dot summary__player-dot--black"></span>
                </span>
              </div>
            </div>

            <div class="tally">
              {#each activeTallyRows(s.moveCountsWhite, s.moveCountsBlack) as classification}
                {@const wCount = s.moveCountsWhite[classification] ?? 0}
                {@const bCount = s.moveCountsBlack[classification] ?? 0}
                <div class="tally__row">
                  <div class="tally__identity">
                    <span
                      class="tally__badge"
                      style="background-color: {badgeColors[classification]}"
                      title={classification}
                    >
                      {#if typeof badgeIcons[classification] === 'string'}
                        <span class="tally__badge-text">{badgeIcons[classification]}</span>
                      {:else}
                        <svelte:component this={badgeIcons[classification]} size={13} weight="fill" />
                      {/if}
                    </span>
                    <span class="tally__label">{tallyLabels[classification]}</span>
                  </div>
                  <div class="tally__counts">
                    <span class="tally__count tally__count--white">{wCount > 0 ? wCount : '—'}</span>
                    <span class="tally__count tally__count--black">{bCount > 0 ? bCount : '—'}</span>
                  </div>
                </div>
              {/each}
            </div>
          </div>

        {/if}
      </div>
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

  /* ── Layout ──────────────────────────────────────────────────────── */
  .layout {
    display: flex;
    height: 100vh;
    max-width: 1200px;
    margin: 0 auto;
    padding: 2rem;
    gap: 2rem;
    box-sizing: border-box;
  }

  .layout__board {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 1.5rem;
    min-width: 0;
  }

  /* ── Board row ───────────────────────────────────────────────────── */
  .board-row {
    display: flex;
    flex-direction: row;
    align-items: stretch;
    gap: 12px;
    width: 100%;
    max-width: 680px;
  }

  .board {
    flex: 1;
    aspect-ratio: 1 / 1;
    border-radius: 6px;
    box-shadow: 0 12px 40px rgba(0, 0, 0, 0.6);
    min-width: 0;
  }

  /* ── Eval bar ────────────────────────────────────────────────────── */
  .eval-bar {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
  }

  .eval-bar__track {
    width: 24px;
    flex: 1;
    border-radius: 8px;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    background: #1a1a1a;
    box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.06);
  }

  .eval-bar__segment {
    transition: flex 0.4s cubic-bezier(0.25, 1, 0.5, 1);
    min-height: 0;
  }

  .eval-bar__segment--black { background: #1e1e1e; }
  .eval-bar__segment--white { background: #f0ede8; }

  .eval-bar__divider {
    height: 3px;
    flex-shrink: 0;
    background: rgba(255, 255, 255, 0.2);
  }

  .eval-bar__label {
    font-family: 'Bebas Neue', sans-serif;
    font-size: 1.35rem;
    color: #888;
    letter-spacing: 0.5px;
  }

  .eval-bar__label--winning { color: #95bb4a; }
  .eval-bar__label--losing  { color: #e06060; }

  /* ── Controls ────────────────────────────────────────────────────── */
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

  .controls__btn--icon { padding: 0.6rem; }

  .controls__btn--analyze {
    background: #1b382b;
    border-color: #2b5743;
    color: #8be1b4;
  }

  .controls__btn--analyze:hover:not(:disabled) {
    background: #234737;
    border-color: #3b7359;
  }

  :global(.spin) { animation: spin 1s linear infinite; }
  @keyframes spin { to { transform: rotate(360deg); } }

  /* ── Sidebar ─────────────────────────────────────────────────────── */
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

  .sidebar__ply-count {
    font-size: 0.8rem;
    color: #555;
    font-weight: 600;
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
    transition: color 0.15s ease, border-color 0.15s ease;
    margin-bottom: -1px;
  }

  .sidebar__nav-btn:hover:not(.sidebar__nav-btn--active) {
    color: #888;
  }

  .sidebar__nav-btn--active {
    color: #ececec;
    border-bottom-color: #ececec;
  }

  /* ── Move list ───────────────────────────────────────────────────── */
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

  .move-list__row:hover { background: #232326; }

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

  .move-list__badge {
    width: 26px;
    height: 26px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #fff;
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.3);
  }

  .move-list__badge-text {
    font-family: 'Outfit', sans-serif;
    font-weight: 800;
    font-size: 0.65rem;
    line-height: 1;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  /* ── Summary view ────────────────────────────────────────────────── */
  .summary {
    flex: 1;
    overflow-y: auto;
    padding: 1.5rem 1.25rem;
    display: flex;
    flex-direction: column;
    gap: 2rem;
    scrollbar-width: thin;
    scrollbar-color: #333 transparent;
  }

  .summary__empty {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #555;
    font-weight: 500;
  }

  .summary__section {
    display: flex;
    flex-direction: column;
    gap: 0.6rem;
  }

  .summary__section-title {
    font-family: 'Bebas Neue', sans-serif;
    font-size: 0.9rem;
    font-weight: 400;
    letter-spacing: 2px;
    text-transform: uppercase;
    color: #f0ede8;
    margin: 0;
    padding-bottom: 0.6rem;
    border-bottom: 1px solid #2a2a2e;
  }

  .summary__accuracy-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.2rem 0;
    border-bottom: 1px solid #1e1e21;
  }

  .summary__accuracy-row:last-child { border-bottom: none; }

  .summary__player {
    display: flex;
    align-items: center;
    gap: 0.2rem;
  }

  .summary__player-dot {
    width: 9px;
    height: 9px;
    border-radius: 50%;
    flex-shrink: 0;
  }

  .summary__player-dot--white { background: #e8e6e0; border: 1px solid #555; }
  .summary__player-dot--black { background: #3a3a3a; border: 1px solid #555; }

  .summary__player-name {
    font-size: 0.88rem;
    font-weight: 600;
    color: #bbb;
  }

  .summary__accuracy-score {
    font-family: 'Bebas Neue', sans-serif;
    font-size: 1.6rem;
    letter-spacing: 1px;
    color: #ececec;
  }

  /* ── Tally ───────────────────────────────────────────────────────── */
  .tally__header {
    display: flex;
    align-items: center;
    padding-bottom: 0.4rem;
    margin-bottom: 0.1rem;
  }

  .tally__header-spacer {
    /* badge + label column — keep aligned with rows below */
    flex: 1;
  }

  .tally__header-players {
    display: flex;
    gap: 0;
    width: 72px;
    flex-shrink: 0;
  }

  .tally__header-player {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 5px;
    font-size: 0.72rem;
    font-weight: 600;
    color: #555;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .tally__header-player--white { justify-content: flex-end; }
  .tally__header-player--black { justify-content: flex-end; }

  .tally {
    display: flex;
    flex-direction: column;
    gap: 0;
  }

  .tally__row {
    display: flex;
    align-items: center;
    padding: 0.42rem 0;
    border-bottom: 1px solid #1e1e21;
  }

  .tally__row:last-child { border-bottom: none; }

  .tally__identity {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 9px;
    min-width: 0;
  }

  .tally__badge {
    width: 22px;
    height: 22px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #fff;
    flex-shrink: 0;
    box-shadow: 0 2px 5px rgba(0, 0, 0, 0.25);
  }

  .tally__badge-text {
    font-family: 'Outfit', sans-serif;
    font-weight: bolder;
    font-size: 0.62rem;
    line-height: 1;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .tally__label {
    font-size: 0.8rem;
    font-weight: 800;
    color: #999;
    white-space: nowrap;
  }

  .tally__counts {
    display: flex;
    width: 72px;
    flex-shrink: 0;
  }

  .tally__count {
    flex: 1;
    font-family: 'Bebas Neue', sans-serif;
    font-size: 1.15rem;
    letter-spacing: 0.5px;
    text-align: right;
  }

  .tally__count--white { color: #ccc; }
  .tally__count--black { color: #888; }

  /* ── Engine Feedback Panel ───────────────────────────────────────── */
  .engine-feedback {
    background: #1c1c1f;
    padding: 1rem 1.25rem;
    display: flex;
    flex-direction: column;
    gap: 0.8rem;
    flex-shrink: 0;
    border-bottom: 1px solid #2a2a2e;
  }

  .engine-feedback__header {
    display: flex;
    align-items: center;
    gap: 0.6rem;
  }

  .engine-feedback__badge {
    width: 24px;
    height: 24px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #fff;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.4);
  }

  .engine-feedback__badge-text {
    font-family: 'Outfit', sans-serif;
    font-weight: 800;
    font-size: 0.75rem;
    line-height: 1;
  }

  .engine-feedback__title {
    font-family: 'Bebas Neue', sans-serif;
    font-size: 1.3rem;
    letter-spacing: 1px;
    color: #ececec;
  }

  .engine-feedback__comparison {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    background: #161618;
    padding: 0.6rem 0.8rem;
    border-radius: 6px;
    border: 1px solid #2a2a2e;
  }

  .engine-line {
    display: flex;
    align-items: center;
    gap: 0.6rem;
    font-size: 0.95rem;
  }

  .engine-line__label {
    color: #666;
    font-weight: 600;
    width: 60px;
    font-size: 0.8rem;
    text-transform: uppercase;
  }

  .engine-line__move {
    font-weight: 600;
    color: #ececec;
    flex: 1;
  }

  .engine-line__eval {
    font-family: 'Bebas Neue', sans-serif;
    color: #888;
    font-size: 1.1rem;
    letter-spacing: 0.5px;
  }

  .engine-line--best .engine-line__move {
    color: #95bb4a;
  }

</style>