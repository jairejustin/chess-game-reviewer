<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import Save from 'lucide-svelte/icons/save';
  
  let threads = 4;
  let hashMb = 128;
  let multiPv = 3;
  let analysisTimeMs = 1500;

  let isApplying = false;
  let isLoading = true;

  onMount(async () => {
    try {
      const config: any = await invoke('get_engine_config');
      
      if (config.threads !== null) threads = config.threads;
      if (config.hashMb !== null) hashMb = config.hashMb;
      if (config.multiPv !== null) multiPv = config.multiPv;
      if (config.analysisTimeMs !== null) analysisTimeMs = config.analysisTimeMs;
    } catch (e) {
      console.error('Failed to load initial engine config:', e);
    } finally {
      isLoading = false;
    }
  });

  async function applySettings() {
    isApplying = true;
    try {
      await invoke('configure_engine', { 
        config: {
          threads,
          hashMb,
          multiPv,
          analysisTimeMs
        }
      });
    } catch (e) {
      console.error('Failed to configure engine:', e);
    } finally {
      setTimeout(() => (isApplying = false), 300); 
    }
  }
</script>

<div class="engine-settings" class:loading={isLoading}>
  <div class="setting-group">
    <div class="setting-header">
      <label for="threads">Threads</label>
      <span class="setting-value">{threads}</span>
    </div>
    <input 
      type="range" 
      id="threads" 
      min="1" max="32" 
      bind:value={threads} 
      class="slider" 
      disabled={isLoading} 
      style="--progress: {((threads - 1) / 31) * 100}%"
    />
    <p class="setting-desc">Number of CPU threads Stockfish can use.</p>
  </div>

  <div class="setting-group">
    <div class="setting-header">
      <label for="hash">Hash Size (MB)</label>
      <span class="setting-value">{hashMb} MB</span>
    </div>
    <input 
      type="range" 
      id="hash" 
      min="16" max="8192" step="16" 
      bind:value={hashMb} 
      class="slider" 
      disabled={isLoading}
      style="--progress: {((hashMb - 16) / 8176) * 100}%"
    />
    <p class="setting-desc">Memory allocated for transposition tables.</p>
  </div>

  <div class="setting-row">
    <div class="setting-group half">
      <label for="multipv">MultiPV</label>
      <input 
        type="number" 
        id="multipv" 
        min="1" max="5" 
        bind:value={multiPv} 
        class="number-input" 
        disabled={isLoading} 
      />
    </div>
    <div class="setting-group half">
      <label for="time">Time per Move (ms)</label>
      <input 
        type="number" 
        id="time" 
        min="100" max="10000" step="100" 
        bind:value={analysisTimeMs} 
        class="number-input" 
        disabled={isLoading} 
      />
    </div>
  </div>

  <button class="apply-btn" on:click={applySettings} disabled={isApplying || isLoading}>
    <Save size={18} strokeWidth={2.5} />
    {isApplying ? 'Applying...' : 'Apply Configuration'}
  </button>
</div>

<style>
  .engine-settings {
    padding: 1.25rem;
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
    flex: 1;
    overflow-y: auto;
    transition: opacity 0.2s ease;
  }
  
  .engine-settings.loading {
    opacity: 0.5;
    pointer-events: none;
  }

  .setting-group {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .setting-row {
    display: flex;
    gap: 1rem;
  }

  .half {
    flex: 1;
  }

  .setting-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  label {
    font-family: 'Bebas Neue', sans-serif;
    font-size: 1.1rem;
    color: #ececec;
    letter-spacing: 0.5px;
  }

  .setting-value {
    font-family: 'Outfit', sans-serif;
    font-size: 0.9rem;
    color: #8be1b4;
    font-weight: 600;
  }

  .setting-desc {
    font-size: 0.75rem;
    color: #777;
    margin: 0;
    line-height: 1.3;
  }

  .slider {
    -webkit-appearance: none;
    appearance: none;
    width: 100%;
    height: 6px;
    background: linear-gradient(
      to right, 
      #8be1b4 0%, 
      #8be1b4 var(--progress), 
      #111 var(--progress), 
      #111 100%
    );
    border: 1px solid #333;
    border-radius: 4px;
    outline: none;
  }

  .slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: #8be1b4;
    cursor: pointer;
    transition: transform 0.1s;
  }

  .slider::-moz-range-thumb {
    appearance: none;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: #8be1b4;
    border: none;
    cursor: pointer;
    transition: transform 0.1s;
  }

  .slider::-webkit-slider-thumb:hover,
  .slider::-moz-range-thumb:hover {
    transform: scale(1.2);
  }

  .slider:disabled::-webkit-slider-thumb,
  .slider:disabled::-moz-range-thumb {
    background: #555;
    cursor: not-allowed;
  }

  .number-input {
    -moz-appearance: textfield;
    appearance: textfield;
    background: #111;
    border: 1px solid #333;
    color: #ececec;
    font-family: 'Outfit', sans-serif;
    font-size: 1rem;
    padding: 0.5rem;
    border-radius: 6px;
    width: 100%;
    box-sizing: border-box;
    outline: none;
    transition: border-color 0.2s;
  }

  .number-input::-webkit-outer-spin-button,
  .number-input::-webkit-inner-spin-button {
    -webkit-appearance: none;
    appearance: none;
    margin: 0;
  }

  .number-input:focus {
    border-color: #555;
  }

  .number-input:disabled {
    color: #555;
    background: #0a0a0a;
    cursor: not-allowed;
  }

  .apply-btn {
    background: #232326;
    border: 1px solid #333;
    color: #ececec;
    padding: 0.8rem 1rem;
    border-radius: 8px;
    cursor: pointer;
    font-family: 'Outfit', sans-serif;
    font-weight: 600;
    font-size: 1rem;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    transition: all 0.2s ease;
    margin-top: auto;
  }

  .apply-btn:hover:not(:disabled) {
    background: #1b382b;
    border-color: #2b5743;
    color: #8be1b4;
  }

  .apply-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
</style>