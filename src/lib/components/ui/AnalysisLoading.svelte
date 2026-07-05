<script lang="ts">
  import { EngineService } from '$lib/services/engineService';
  import { isAnalyzing } from '$lib/stores/reviewStore';
  import Loader2 from 'lucide-svelte/icons/loader-2';
  import X from 'lucide-svelte/icons/x';

  export let progress: number = 0;

  async function handleCancel() {
    try {
      await EngineService.cancelAnalysis();
      isAnalyzing.set(false);
    } catch (e) {
      console.error('Failed to cancel analysis:', e);
    }
  }
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

  <button class="cancel-btn" on:click={handleCancel} title="Cancel Analysis">
    <X size={24} strokeWidth={2.5} />
  </button>
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
    color: var(--accent-primary, #8be1b4);
    margin-bottom: 2rem;
  }

  .loading-overlay__title {
    font-family: 'Bebas Neue', sans-serif;
    font-size: 1.5rem;
    letter-spacing: 1px;
    color: var(--text-primary, #ececec);
    margin: 0 0 0.5rem 0;
  }

  .loading-overlay__desc {
    font-size: 0.9rem;
    color: var(--text-secondary, #888);
    margin: 0 0 2rem 0;
  }

  .progress-track {
    width: 80%;
    height: 6px;
    background: var(--bg-element, #111);
    border: 1px solid var(--border-strong, #333);
    border-radius: 4px;
    overflow: hidden;
    margin-bottom: 0.5rem;
  }

  .progress-fill {
    height: 100%;
    background: var(--accent-primary, #8be1b4);
    transition: width 0.15s ease-out;
  }

  .progress-text {
    font-family: 'Bebas Neue', sans-serif;
    font-size: 1.2rem;
    color: var(--accent-primary, #8be1b4);
    letter-spacing: 1px;
  }

  .cancel-btn {
    margin-top: 2rem;
    background: #2a1a1a;
    border: 1px solid #4a2a2a;
    color: var(--accent-danger, #e07070);
    border-radius: 50%;
    width: 48px;
    height: 48px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .cancel-btn:hover {
    background: #3a1a1a;
    border-color: var(--accent-danger, #e07070);
    transform: scale(1.05);
  }
</style>
