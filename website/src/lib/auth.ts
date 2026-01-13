import { writable } from 'svelte/store';

export interface User {
    discord_id: string;
    username: string;
    global_name: string | null;
    nickname: string | null;
    avatar: string | null;
    highest_role: string | null;
    is_admin: boolean;
    linked_players: string[]; // CoC linked players
    linked_cr_players: string[]; // CR linked players
}

export const user = writable<User | null>(null);
export const loading = writable<boolean>(true);

const apiBaseUrl =
    import.meta.env.VITE_API_BASE_URL !== undefined
        ? import.meta.env.VITE_API_BASE_URL
        : 'http://localhost:8888';

export async function fetchUser() {
    loading.set(true);
    try {
        const response = await fetch(`${apiBaseUrl}/auth/me`, {
            credentials: 'include',
        });
        if (response.ok) {
            const userData = await response.json();
            user.set(userData);
        } else {
            user.set(null);
        }
    } catch (error) {
        console.error('Failed to fetch user:', error);
        user.set(null);
    } finally {
        loading.set(false);
    }
}

export function login() {
    window.location.href = `${apiBaseUrl}/auth/discord/login`;
}

export async function logout() {
    try {
        await fetch(`${apiBaseUrl}/auth/logout`, {
            method: 'POST',
            credentials: 'include',
        });
        user.set(null);
    } catch (error) {
        console.error('Logout failed:', error);
    }
}

// API helper functions
export type GameType = 'coc' | 'cr';

export function getApiPrefix(game: GameType): string {
    return `/api/${game}`;
}
