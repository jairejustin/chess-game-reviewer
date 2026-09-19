<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import Save from 'lucide-svelte/icons/save';
  import Info from 'lucide-svelte/icons/info';
  import RotateCcw from 'lucide-svelte/icons/rotate-ccw';

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
      console.error(e);
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
      console.error(e);
    } finally {
      setTimeout(() => (isApplying = false), 300);
    }
  }

  function resetToDefaults() {
    threads = 4;
    hashMb = 128;
    multiPv = 3;
    analysisTimeMs = 1500;
  }
</script>

<div class="engine-settings" class:loading={isLoading}>
  <div class="setting-row">
    <div class="setting-group half">
      <div class="setting-header">
        <label for="threads">Threads</label>
        <div class="setting-value-group">
          <span class="setting-value">{threads}</span>
          <div class="info-container">
            <Info size={14} strokeWidth={2.5} />
            <div class="tooltip">Number of CPU threads Stockfish can use.</div>
          </div>
        </div>
      </div>
      <input
        type="range"
        id="threads"
        min="1"
        max="32"
        bind:value={threads}
        class="slider"
        disabled={isLoading}
        style="--progress: {((threads - 1) / 31) * 100}%"
      />
    </div>

    <div class="setting-group half">
      <div class="setting-header">
        <label for="hash">Hash Size</label>
        <div class="setting-value-group">
          <span class="setting-value">{hashMb} MB</span>
          <div class="info-container">
            <Info size={14} strokeWidth={2.5} />
            <div class="tooltip">Memory allocated for transposition tables.</div>
          </div>
        </div>
      </div>
      <input
        type="range"
        id="hash"
        min="16"
        max="8192"
        step="16"
        bind:value={hashMb}
        class="slider"
        disabled={isLoading}
        style="--progress: {((hashMb - 16) / 8176) * 100}%"
      />
    </div>
  </div>

  <div class="setting-row">
    <div class="setting-group half">
      <div class="setting-header">
        <label for="multipv">MultiPV</label>
        <div class="setting-value-group">
          <span class="setting-value">{multiPv}</span>
          <div class="info-container">
            <Info size={14} strokeWidth={2.5} />
            <div class="tooltip">Number of best lines (Principal Variations) to calculate.</div>
          </div>
        </div>
      </div>
      <input
        type="range"
        id="multipv"
        min="1"
        max="5"
        step="1"
        bind:value={multiPv}
        class="slider"
        disabled={isLoading}
        style="--progress: {((multiPv - 1) / 4) * 100}%"
      />
    </div>

    <div class="setting-group half">
      <div class="setting-header">
        <label for="time">Time / Move</label>
        <div class="setting-value-group">
          <span class="setting-value">{analysisTimeMs}ms</span>
          <div class="info-container">
            <Info size={14} strokeWidth={2.5} />
            <div class="tooltip">Maximum time the engine spends thinking per move.</div>
          </div>
        </div>
      </div>
      <input
        type="range"
        id="time"
        min="100"
        max="10000"
        step="500"
        bind:value={analysisTimeMs}
        class="slider"
        disabled={isLoading}
        style="--progress: {((analysisTimeMs - 100) / 9900) * 100}%"
      />
    </div>
  </div>

  <div class="settings-actions">
    <button
      class="action-btn reset-btn"
      on:click={resetToDefaults}
      disabled={isLoading || isApplying}
      title="Reset to Defaults"
    >
      <RotateCcw size={20} strokeWidth={2.5} />
    </button>
    
    <button
      class="action-btn apply-btn"
      on:click={applySettings}
      disabled={isApplying || isLoading}
      title={isApplying ? 'Applying...' : 'Apply Configuration'}
    >
      <Save size={20} strokeWidth={2.5} />
    </button>
  </div>
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

  .setting-value-group {
    display: flex;
    align-items: center;
    gap: 0.5rem;
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

  .info-container {
    position: relative;
    display: flex;
    align-items: center;
    color: #777;
    cursor: help;
    transition: color 0.2s ease;
  }

  .info-container:hover {
    color: #8be1b4;
    z-index: 50;
  }

  .tooltip {
    position: absolute;
    top: 100%;
    right: 0;
    margin-top: 8px;
    width: max-content;
    max-width: 180px;
    background: #111;
    color: #ececec;
    padding: 0.5rem 0.6rem;
    border-radius: 6px;
    font-size: 0.75rem;
    font-family: 'Outfit', sans-serif;
    line-height: 1.3;
    border: 1px solid #333;
    box-shadow: 0 4px 6px rgba(0,0,0,0.4);
    opacity: 0;
    visibility: hidden;
    pointer-events: none;
    transition: opacity 0.2s ease, transform 0.2s ease;
    z-index: 50;
    text-align: left;
    transform: translateY(-4px);
  }

  .info-container:hover .tooltip {
    opacity: 1;
    visibility: visible;
    transform: translateY(0);
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

  .settings-actions {
    display: flex;
    gap: 0.5rem;
    margin-top: 0.5rem;
    align-self: flex-end;
    flex-shrink: 0;
  }

  .action-btn {
    background: #232326;
    border: 1px solid #333;
    color: #ececec;
    
    width: 42px !important;
    min-width: 42px !important;
    height: 42px !important;
    padding: 0 !important;
    
    border-radius: 8px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.2s ease;
    flex-shrink: 0;
  }

  .action-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .apply-btn:hover:not(:disabled),
  .reset-btn:hover:not(:disabled) {
    background: #1b382b;
    border-color: #2b5743;
    color: #8be1b4;
  }

</style>