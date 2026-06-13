import { writable, get } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import type {
  FetchResult,
  GameSummary,
  ChessComCursor,
  Platform
} from '../types/fetch';

export const usernameInput = writable<string>('');
export const selectedPlatform = writable<Platform>('chesscom');

export const fetchedProfile = writable<FetchResult['profile'] | null>(null);
export const fetchedGames = writable<GameSummary[]>([]);
export const selectedGame = writable<GameSummary | null>(null);
export const isFetching = writable<boolean>(false);
export const fetchError = writable<string | null>(null);
export const fetchCursor = writable<ChessComCursor | null>(null);
export const hasMore = writable<boolean>(false);
export const processedGameId = writable<string | null>(null);

export async function fetchGames(
  username: string,
  platform: Platform,
  cursor?: ChessComCursor
) {
  isFetching.set(true);
  fetchError.set(null);

  try {
    const result = await invoke<FetchResult>('fetch_games', {
      username,
      platform,
      cursor: cursor ?? null
    });

    if (cursor) {
      fetchedGames.update((g) => [...g, ...result.games]);
    } else {
      fetchedGames.set(result.games);
      fetchedProfile.set(result.profile);
      selectedGame.set(null);
    }

    fetchCursor.set(result.cursor);
    hasMore.set(result.cursor !== null);
  } catch (e) {
    fetchError.set(String(e));
  } finally {
    isFetching.set(false);
  }
}

export function loadMore(username: string, platform: Platform) {
  const cursor = get(fetchCursor);
  if (cursor) fetchGames(username, platform, cursor);
}
