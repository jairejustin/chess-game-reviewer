<script lang="ts">
  import {
    usernameInput,
    selectedPlatform,
    fetchedProfile,
    fetchedGames,
    selectedGame,
    isFetching,
    fetchError,
    hasMore,
    fetchGames,
    loadMore
  } from '$lib/stores/fetchStore';
  import Loader2 from 'lucide-svelte/icons/loader-2';
  import ClipboardPaste from 'lucide-svelte/icons/clipboard-paste';

  let currentTab: 'chesscom' | 'lichess' | 'paste' = $selectedPlatform;
  let manualPgnInput = '';

  function setTab(tab: 'chesscom' | 'lichess' | 'paste') {
    currentTab = tab;
    if (tab !== 'paste') {
      $selectedPlatform = tab;
    }
  }

  function handleFetch() {
    if (!$usernameInput.trim()) return;
    fetchGames($usernameInput.trim(), $selectedPlatform);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') handleFetch();
  }

  function formatResult(result: string): string {
    if (result === '1/2-1/2') return '½-½';
    if (result === '1-0') return '1-0';
    if (result === '0-1') return '0-1';
    return result;
  }

  function formatTimeClass(tc: string): string {
    return tc.charAt(0).toUpperCase() + tc.slice(1);
  }

  function handleManualImport() {
    if (!manualPgnInput.trim()) return;

    const whiteMatch = manualPgnInput.match(/\[White "(.*?)"\]/);
    const blackMatch = manualPgnInput.match(/\[Black "(.*?)"\]/);
    const resultMatch = manualPgnInput.match(/\[Result "(.*?)"\]/);
    const res = resultMatch ? resultMatch[1] : '*';

    const manualGame: any = {
      id: 'manual-' + Date.now(),
      timeClass: 'manual',
      pgn: manualPgnInput,
      white: {
        username: whiteMatch ? whiteMatch[1] : 'White',
        result: res
      },
      black: {
        username: blackMatch ? blackMatch[1] : 'Black',
        result: res
      }
    };

    selectedGame.set(manualGame);
    manualPgnInput = '';
  }
</script>

<div class="fetch-games">
  <div class="platform-toggle">
    <button
      class="platform-btn"
      class:platform-btn--active={currentTab === 'chesscom'}
      on:click={() => setTab('chesscom')}
    >
      <img
        src="/assets/chesscom_logo_pawn_negative.png"
        alt="Chess.com"
        width="18"
        height="20"
        class="platform-logo"
      />
      <span>Chess.com</span>
    </button>

    <button
      class="platform-btn"
      class:platform-btn--active={currentTab === 'lichess'}
      on:click={() => setTab('lichess')}
    >
      <img
        src="/assets/lichess_logo_white_transparent.png"
        alt="Lichess"
        width="20"
        height="20"
        class="platform-logo"
      />
      <span>Lichess</span>
    </button>

    <button
      class="platform-btn"
      class:platform-btn--active={currentTab === 'paste'}
      on:click={() => setTab('paste')}
    >
      <ClipboardPaste size={20} strokeWidth={1.5} class="platform-logo" />
      <span>PGN</span>
    </button>
  </div>

  {#if currentTab !== 'paste'}
    <div class="fetch-input-row">
      <input
        class="fetch-input"
        type="text"
        placeholder={'Username'}
        bind:value={$usernameInput}
        on:keydown={handleKeydown}
        disabled={$isFetching}
      />
      <button
        class="fetch-btn"
        on:click={handleFetch}
        disabled={$isFetching || !$usernameInput.trim()}
      >
        {#if $isFetching && !$fetchedGames.length}
          <Loader2 size={16} class="spin" strokeWidth={2.5} />
        {:else}
          Fetch
        {/if}
      </button>
    </div>

    {#if $fetchError}
      <p class="fetch-error">{$fetchError}</p>
    {/if}

    {#if $fetchedProfile}
      <div class="profile-strip">
        <div class="profile-strip__avatar">
          {#if $fetchedProfile.avatarUrl}
            <img
              src={$fetchedProfile.avatarUrl}
              alt={$fetchedProfile.username}
            />
          {:else}
            <div class="profile-strip__avatar-fallback">
              {$fetchedProfile.username.charAt(0).toUpperCase()}
            </div>
          {/if}
        </div>
        <div class="profile-strip__info">
          {#if $fetchedProfile.title}
            <span class="player-title">{$fetchedProfile.title}</span>
          {/if}
          <span class="profile-strip__name">{$fetchedProfile.username}</span>
          <div class="profile-strip__meta">
            {#if $fetchedProfile.countryCode}
              <span class="profile-strip__tag"
                >{$fetchedProfile.countryCode}</span
              >
            {/if}
          </div>
        </div>
      </div>
    {/if}

    {#if $fetchedGames.length > 0}
      <div class="game-list">
        {#each $fetchedGames as game}
          {@const isSelected = $selectedGame?.id === game.id}
          {@const canonicalUser = $fetchedProfile?.username || $usernameInput}
          {@const userIsWhite =
            game.white.username.toLowerCase() === canonicalUser.toLowerCase()}
          {@const opponent = userIsWhite ? game.black : game.white}
          {@const userSide = userIsWhite ? game.white : game.black}
          {@const res = userSide.result}

          {@const isWin =
            res === 'win' ||
            (userIsWhite && res === '1-0') ||
            (!userIsWhite && res === '0-1')}
          {@const isLoss =
            res === 'checkmated' ||
            res === 'timeout' ||
            res === 'resign' ||
            res === 'lose' ||
            res === 'abandoned' ||
            (userIsWhite && res === '0-1') ||
            (!userIsWhite && res === '1-0')}

          <button
            class="game-row"
            class:game-row--selected={isSelected}
            on:click={() => selectedGame.set(game)}
          >
            <div class="game-row__left">
              <span class="game-row__time-class"
                >{formatTimeClass(game.timeClass)}</span
              >
              <span class="game-row__opponent">
                vs {opponent.username}
                {#if opponent.rating}
                  <span class="game-row__rating">({opponent.rating})</span>
                {/if}
              </span>
            </div>
            <div class="game-row__right">
              <span
                class="game-row__result"
                class:game-row__result--win={isWin}
                class:game-row__result--loss={isLoss}
                class:game-row__result--draw={!isWin && !isLoss}
              >
                {formatResult(res)}
              </span>
            </div>
          </button>
        {/each}

        {#if $hasMore}
          <button
            class="load-more-btn"
            on:click={() => loadMore($usernameInput, $selectedPlatform)}
            disabled={$isFetching}
          >
            {#if $isFetching}
              <Loader2 size={16} class="spin" strokeWidth={2.5} /> Loading...
            {:else}
              Load More
            {/if}
          </button>
        {/if}
      </div>
    {/if}
  {:else}
    <div class="paste-section">
      <textarea
        class="pgn-textarea"
        placeholder="[Event &quot;Live Chess&quot;]&#10;[White &quot;Player1&quot;]&#10;[Black &quot;Player2&quot;]&#10;..."
        bind:value={manualPgnInput}
      ></textarea>
      <button
        class="fetch-btn load-more-btn"
        on:click={handleManualImport}
        disabled={!manualPgnInput.trim()}
      >
        Import Game
      </button>
    </div>
  {/if}
</div>

<style>
  .fetch-games {
    padding: 1.25rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
    flex: 1;
    overflow-y: auto;
    min-height: 0;
  }

  .platform-toggle {
    display: flex;
    background: #111;
    border: 1px solid #2a2a2e;
    border-radius: 8px;
    padding: 3px;
    gap: 3px;
  }

  .platform-btn {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    border-radius: 6px;
    color: #666;
    font-family: 'Outfit', sans-serif;
    font-weight: 600;
    font-size: 0.85rem;
    padding: 0.5rem 0.45rem;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .platform-btn--active {
    background: #232326;
    color: #ececec;
  }

  .platform-btn:hover:not(.platform-btn--active) {
    color: #999;
  }

  .platform-logo,
  :global(.platform-btn svg) {
    opacity: 0.8;
    transition: opacity 0.15s ease;
    flex-shrink: 0;
  }

  .platform-btn--active .platform-logo,
  :global(.platform-btn--active svg) {
    opacity: 1;
  }

  .fetch-input-row {
    display: flex;
    gap: 0.5rem;
  }

  .fetch-input {
    flex: 1;
    background: #111;
    border: 1px solid #333;
    padding: 0.7rem 0.8rem;
    border-radius: 6px;
    color: #ececec;
    font-family: 'Outfit', sans-serif;
    font-size: 0.9rem;
    outline: none;
    transition: border-color 0.2s;
  }

  .fetch-input:focus {
    border-color: #555;
  }

  .fetch-btn {
    background: #232326;
    border: 1px solid #333;
    color: #ececec;
    padding: 0.7rem 1rem;
    border-radius: 6px;
    cursor: pointer;
    font-family: 'Outfit', sans-serif;
    font-weight: 600;
    font-size: 0.9rem;
    transition: all 0.15s ease;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.4rem;
    white-space: nowrap;
  }

  .fetch-btn:hover:not(:disabled) {
    background: #2e2e32;
    border-color: #444;
  }

  .fetch-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .fetch-error {
    font-size: 0.85rem;
    color: #e07070;
    margin: 0;
    padding: 0.6rem 0.8rem;
    background: #2a1a1a;
    border: 1px solid #4a2a2a;
    border-radius: 6px;
  }

  .profile-strip {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem;
    background: #1c1c1f;
    border: 1px solid #2a2a2e;
    border-radius: 8px;
  }

  .profile-strip__avatar {
    width: 2.5rem;
    height: 2.5rem;
    border-radius: 6px;
    overflow: hidden;
    background: #232326;
    flex-shrink: 0;
  }

  .profile-strip__avatar img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }

  .profile-strip__avatar-fallback {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-weight: 700;
    font-size: 1.1rem;
    color: #888;
  }

  .profile-strip__info {
    display: flex;
    flex-direction: column;
    gap: 3px;
    min-width: 0;
  }

  .profile-strip__name {
    font-weight: 700;
    font-size: 0.95rem;
    color: #ececec;
  }

  .profile-strip__meta {
    display: flex;
    gap: 0.4rem;
    flex-wrap: wrap;
  }

  .profile-strip__tag {
    font-size: 0.75rem;
    color: #888;
    background: #232326;
    border: 1px solid #333;
    border-radius: 4px;
    padding: 1px 6px;
  }

  .game-list {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }

  .game-row {
    appearance: none;
    background: transparent;
    border: 1px solid transparent;
    border-radius: 8px;
    color: inherit;
    font-family: inherit;
    text-align: left;
    width: 100%;
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.5rem 0.75rem;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .game-row:hover {
    background: #1c1c1f;
  }

  .game-row--selected {
    background: #1c2535;
    border-color: #2a3f5a;
  }

  .game-row__left {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }

  .game-row__time-class {
    font-size: 0.75rem;
    color: #666;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  .game-row__opponent {
    font-size: 0.9rem;
    font-weight: 600;
    color: #ccc;
  }

  .game-row__rating {
    font-weight: 400;
    color: #777;
    font-size: 0.85rem;
  }

  .game-row__right {
    flex-shrink: 0;
  }

  .game-row__result {
    font-family: 'Bebas Neue', sans-serif;
    font-size: 1.1rem;
    letter-spacing: 0.5px;
    color: #888;
  }

  .game-row__result--win {
    color: #8be1b4;
  }

  .game-row__result--loss {
    color: #e07070;
  }

  .game-row__result--draw {
    color: #aaa;
  }

  .load-more-btn {
    appearance: none;
    background: transparent;
    border: 1px solid #2a2a2e;
    border-radius: 8px;
    color: #666;
    font-family: 'Outfit', sans-serif;
    font-weight: 600;
    font-size: 0.85rem;
    padding: 0.6rem;
    width: 100%;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.4rem;
    transition: all 0.15s ease;
    margin-top: 0.25rem;
  }

  .load-more-btn:hover:not(:disabled) {
    background: #1c1c1f;
    color: #aaa;
  }

  .load-more-btn:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }

  .player-title {
    background: #b33430;
    color: #fff;
    font-family: 'Outfit', sans-serif;
    font-weight: 800;
    font-size: 0.7rem;
    padding: 1px 4px;
    border-radius: 4px;
    line-height: 1;
    text-transform: uppercase;
  }

  .paste-section {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    height: 100%;
  }

  .pgn-textarea {
    flex: 1;
    min-height: 150px;
    background: #111;
    border: 1px solid #333;
    padding: 0.75rem;
    border-radius: 6px;
    color: #ececec;
    font-family: 'JetBrains Mono', 'Courier New', monospace;
    font-size: 0.85rem;
    resize: none;
    outline: none;
    transition: border-color 0.2s;
  }

  .pgn-textarea:focus {
    border-color: #555;
  }
</style>
