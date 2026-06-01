<script lang="ts">
  import MaterialStrip from './MaterialStrip.svelte';

  export let name: string = 'Player';
  export let rating: number | null = null;
  export let avatarUrl: string | null = null;
  export let capturedPieces: string[] = [];
  export let advantage: number = 0;

  $: fallbackAvatar = `https://ui-avatars.com/api/?name=${name}&background=232326&color=ececec`;
</script>

<div class="player-profile">
  <div class="player-profile__avatar">
    <img src={avatarUrl ?? fallbackAvatar} alt={name} />
  </div>
  <div class="player-profile__info">
    <span class="player-profile__name">
      {name}{rating ? ` (${rating})` : ''}
    </span>
    <MaterialStrip {capturedPieces} {advantage} />
  </div>
</div>

<style>
  .player-profile {
    display: flex;
    align-items: center;
    gap: 12px;
    height: 40px;
    padding: 0 2px;
    width: 100%;
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
</style>
