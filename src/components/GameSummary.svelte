<script lang="ts">
  // Added activePly and sidebarView imports here
  import { analysisSummary, activePly, sidebarView } from '../store/gameStore';
  import { tallyLabels, formatAccuracy, activeTallyRows } from '../utils/ui';
  import Badge from './Badge.svelte';
  import EvalGraph from './EvalGraph.svelte';

  function startReview() {
    activePly.set(0);
    sidebarView.set('game');
  }
</script>

<div class="summary">
  {#if !$analysisSummary}
    <div class="summary__empty">
      <p>Run analysis to see a summary.</p>
    </div>
  {:else}
    {@const s = $analysisSummary}

    <div class="summary__section">
      <h3 class="summary__section-title">Evaluation</h3>
      <EvalGraph />
    </div>

    <div class="summary__section">
      <h3 class="summary__section-title">Accuracy</h3>
      <div class="summary__accuracy-row">
        <div class="summary__player">
          <span class="summary__player-dot summary__player-dot--white"></span>
          <span class="summary__player-name">{s.metadata.white}</span>
        </div>
        <span class="summary__accuracy-score"
          >{formatAccuracy(s.whiteAccuracy)}</span
        >
      </div>
      <div class="summary__accuracy-row">
        <div class="summary__player">
          <span class="summary__player-dot summary__player-dot--black"></span>
          <span class="summary__player-name">{s.metadata.black}</span>
        </div>
        <span class="summary__accuracy-score"
          >{formatAccuracy(s.blackAccuracy)}</span
        >
      </div>
    </div>

    <div class="summary__section">
      <h3 class="summary__section-title">Move Tally</h3>
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
              <div title={classification}>
                <Badge {classification} size={28} />
              </div>
              <span class="tally__label">{tallyLabels[classification]}</span>
            </div>
            <div class="tally__counts">
              <span class="tally__count tally__count--white"
                >{wCount > 0 ? wCount : '—'}</span
              >
              <span class="tally__count tally__count--black"
                >{bCount > 0 ? bCount : '—'}</span
              >
            </div>
          </div>
        {/each}
      </div>
    </div>

    <button class="review-btn" on:click={startReview}>
      Start Review
    </button>
  {/if}
</div>

<style>
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
  .summary__accuracy-row:last-child {
    border-bottom: none;
  }
  .summary__player {
    display: flex;
    align-items: center;
    gap: 0.4rem;
  }
  .summary__player-dot {
    width: 9px;
    height: 9px;
    border-radius: 50%;
    flex-shrink: 0;
  }
  .summary__player-dot--white {
    background: #e8e6e0;
    border: 1px solid #555;
  }
  .summary__player-dot--black {
    background: #3a3a3a;
    border: 1px solid #555;
  }
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
  .tally__header {
    display: flex;
    align-items: center;
    padding-bottom: 0.4rem;
    margin-bottom: 0.1rem;
  }
  .tally__header-spacer {
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
  .tally__header-player--white {
    justify-content: flex-end;
  }
  .tally__header-player--black {
    justify-content: flex-end;
  }
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
  .tally__row:last-child {
    border-bottom: none;
  }
  .tally__identity {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 11px;
    min-width: 0;
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
  .tally__count--white {
    color: #ccc;
  }
  .tally__count--black {
    color: #888;
  }

  /* NEW: Review Button Styles */
  .review-btn {
    background: #1b382b;
    border: 1px solid #2b5743;
    color: #8be1b4;
    padding: 0.8rem 1.2rem;
    border-radius: 8px;
    cursor: pointer;
    font-family: 'Outfit', sans-serif;
    font-weight: 600;
    font-size: 1rem;
    width: 100%;
    margin-top: 1rem;
    transition: all 0.2s ease;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  
  .review-btn:hover {
    background: #234737;
    border-color: #3b7359;
  }
</style>