<script lang="ts">
  import Loader2 from 'lucide-svelte/icons/loader-2';
  export let progress: number = 0; // 0 to 1 value
</script>

<div class="loading-overlay">
  <div class="loading-overlay__icon-wrapper">
    <Loader2 size={48} class="spin" strokeWidth={2} />
  </div>
  <h3 class="loading-overlay__title">Analyzing...</h3>
  <p class="loading-overlay__desc">The engine is evaluating</p>
  <div class="progress-track">
    <div class="progress-fill" style="width: {progress * 100}%"></div>
  </div>
  <span class="progress-text">{Math.round(progress * 100)}%</span>
</div>

<style>
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
