<script lang="ts">
  import { currentEval, currentMateIn } from '../store/gameStore';
  import { formatEval } from '../utils/ui';

  function winPercent(cp: number): number {
    return 50 + 50 * (2 / (1 + Math.exp(-0.00368 * cp)) - 1);
  }

  $: whitePercent = winPercent($currentEval);
  $: blackPercent = 100 - whitePercent;
</script>

<div
  class="eval-bar"
  aria-label="Evaluation bar, {formatEval($currentEval, $currentMateIn)}"
>
  <div class="eval-bar__track">
    <div
      class="eval-bar__segment eval-bar__segment--black"
      style="flex: {blackPercent}"
    ></div>
    <div class="eval-bar__divider"></div>
    <div
      class="eval-bar__segment eval-bar__segment--white"
      style="flex: {whitePercent}"
    ></div>
  </div>

  <span
    class="eval-bar__label"
    class:eval-bar__label--winning={$currentEval > 0}
    class:eval-bar__label--losing={$currentEval < 0}
  >
    {formatEval($currentEval, $currentMateIn)}
  </span>
</div>

<style>
  .eval-bar {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 10px;
    width: 56px;
    flex-shrink: 0;
    height: 100%;
  }

  .eval-bar__track {
    width: 24px;
    flex: 1;
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
  .eval-bar__segment--black {
    background: #1e1e1e;
  }
  .eval-bar__segment--white {
    background: #f0ede8;
  }
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
    width: 56px;
    text-align: center;
    display: inline-block;
  }
  .eval-bar__label--winning {
    color: #95bb4a;
  }
  .eval-bar__label--losing {
    color: #e06060;
  }
</style>
