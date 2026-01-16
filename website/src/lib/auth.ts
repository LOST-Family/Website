import { writable, derived } from 'svelte/store';

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

const internalUser = writable<User | null>(null);
// Export read-only version of the real user for UI checks
export const realUser = { subscribe: internalUser.subscribe };
export const userOverride = writable<{
    is_admin?: boolean;
    highest_role?: string | null;
} | null>(null);

export const user = derived(
    [internalUser, userOverride],
    ([$internalUser, $userOverride]) => {
        if (!$internalUser) return null;
        // Only allow overrides if the actual user is an admin
        if (!$internalUser.is_admin) return $internalUser;
        if (!$userOverride) return $internalUser;

        return {
            ...$internalUser,
            is_admin: $userOverride.is_admin ?? $internalUser.is_admin,
            highest_role:
                $userOverride.highest_role !== undefined
                    ? $userOverride.highest_role
                    : $internalUser.highest_role,
            // If an override is active, we also strip linked players to simulate a "clean" account
            // unless we want to keep them. For testing "MEMBER" view, we usually want these gone.
            linked_players: $userOverride ? [] : $internalUser.linked_players,
            linked_cr_players: $userOverride
                ? []
                : $internalUser.linked_cr_players,
        };
    }
);

export function getRolePriority(role: string | null | undefined): number {
    if (!role) return 0;
    switch (role.toUpperCase()) {
        case 'ADMIN':
            return 1000;
        case 'LEADER':
            return 100;
        case 'COLEADER':
            return 80;
        case 'ELDER':
            return 50;
        case 'MEMBER':
            return 10;
        case 'NOTINCLAN':
        case 'NOTMEMBER':
            return 0;
        default:
            return 0;
    }
}

export function hasRequiredRole(
    userRole: string | null | undefined,
    requiredRole: string
): boolean {
    return getRolePriority(userRole) >= getRolePriority(requiredRole);
}

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
            internalUser.set(userData);
        } else {
            internalUser.set(null);
        }
    } catch (error) {
        console.error('Failed to fetch user:', error);
        internalUser.set(null);
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
        internalUser.set(null);
    } catch (error) {
        console.error('Logout failed:', error);
    }
}

// API helper functions
export type GameType = 'coc' | 'cr';

export function getApiPrefix(game: GameType): string {
    return `/api/${game}`;
}
