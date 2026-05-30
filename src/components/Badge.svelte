<script lang="ts">
  import type { MoveBadge } from '../types/game';

  import Star from 'lucide-svelte/icons/star';
  import ThumbsUp from 'lucide-svelte/icons/thumbs-up';
  import BookOpen from 'lucide-svelte/icons/book-open';
  import Check from 'lucide-svelte/icons/check';
  import ArrowRight from 'lucide-svelte/icons/arrow-right';
  import X from 'lucide-svelte/icons/x';

  export let classification: MoveBadge;
  export let size: number = 24;

  const badgeColors: Record<MoveBadge, string> = {
    brilliant: '#1baca6',
    great: '#5c8bb0',
    best: '#95bb4a',
    excellent: '#96bc4b',
    good: '#96ba4b',
    inaccuracy: '#f6b236',
    mistake: '#e28c28',
    blunder: '#ca3431',
    miss: '#ff7769',
    book: '#a88764',
    forced: '#96ba4b'
  };

  const badgeIcons: Record<MoveBadge, any> = {
    brilliant: '!!',
    great: '!',
    best: Star,
    excellent: ThumbsUp,
    good: Check,
    inaccuracy: '?!',
    mistake: '?',
    blunder: '??',
    miss: X,
    book: BookOpen,
    forced: ArrowRight
  };

  const strokeIcons = [Check, ArrowRight, X];

  $: color = badgeColors[classification];
  $: content = badgeIcons[classification];
  $: isText = typeof content === 'string';

  $: textSize = size * 0.65;
  $: iconSize = size * 0.65;

  $: isStrokeIcon = strokeIcons.includes(content);

  $: iconStrokeWidth = isStrokeIcon ? 5 : 0;
  $: iconFill = isStrokeIcon ? 'none' : '#fff';
</script>

<div
  class="badge"
  style="width: {size}px; height: {size}px; background-color: {color};"
>
  {#if isText}
    <span class="badge__text" style="font-size: {textSize}px;">
      {content}
    </span>
  {:else}
    <svelte:component
      this={content}
      size={iconSize}
      strokeWidth={iconStrokeWidth}
      fill={iconFill}
      color="#fff"
    />
  {/if}
</div>

<style>
  .badge {
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #fff;
    flex-shrink: 0;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.4);
  }

  .badge__text {
    font-family: 'Outfit', sans-serif;
    font-weight: bolder;
    line-height: 1;
    margin-top: 1px;
  }
</style>
