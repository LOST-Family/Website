<script lang="ts">
    import ThemeToggle from './ThemeToggle.svelte';
    import { createEventDispatcher, onMount, onDestroy } from 'svelte';
    import { user, loading, login, logout } from './auth';

    export let theme: 'dark' | 'light' = 'dark';
    export let logo: string;

    let mobileMenuOpen = false;
    let userClans: {
        tag: string;
        name: string;
        gameType: string;
        badgeUrl?: string;
    }[] = [];

    const apiBaseUrl =
        import.meta.env.VITE_API_BASE_URL !== undefined
            ? import.meta.env.VITE_API_BASE_URL
            : 'http://localhost:8888';

    const dispatch = createEventDispatcher<{
        themeToggle: { theme: 'dark' | 'light' };
        navigate: string;
    }>();

    function handleThemeToggle(
        event: CustomEvent<{ theme: 'dark' | 'light' }>,
    ) {
        dispatch('themeToggle', event.detail);
    }

    function navigate(page: string) {
        dispatch('navigate', page);
        mobileMenuOpen = false;
    }

    function toggleMobileMenu() {
        mobileMenuOpen = !mobileMenuOpen;
    }

    function handleBackdropKeydown(event: KeyboardEvent) {
        if (event.key === 'Enter' || event.key === ' ') {
            toggleMobileMenu();
        }
    }

    $: if (typeof document !== 'undefined' && document.body) {
        if (mobileMenuOpen) {
            document.body.classList.add('no-scroll');
        } else {
            document.body.classList.remove('no-scroll');
        }
    }

    onDestroy(() => {
        if (typeof document !== 'undefined') {
            document.body.classList.remove('no-scroll');
        }
    });

    async function fetchUserClans() {
        if (!$user) return;
        try {
            const [accountsRes, cocClansRes, crClansRes] = await Promise.all([
                fetch(`${apiBaseUrl}/api/me/accounts`, {
                    credentials: 'include',
                }),
                fetch(`${apiBaseUrl}/api/coc/clans`, {
                    credentials: 'include',
                }),
                fetch(`${apiBaseUrl}/api/cr/clans`, {
                    credentials: 'include',
                }),
            ]);

            const accounts = accountsRes.ok
                ? await accountsRes.json()
                : { coc: [], cr: [] };
            const cocClans = cocClansRes.ok ? await cocClansRes.json() : [];
            const crClans = crClansRes.ok ? await crClansRes.json() : [];

            const officialCocTags = new Set(
                cocClans.map((c: any) => c.tag.toUpperCase()),
            );
            const officialCrTags = new Set(
                crClans.map((c: any) => c.tag.toUpperCase()),
            );

            const badgeMap = new Map<string, string>();
            cocClans.forEach((c: any) => {
                if (c.badgeUrl) badgeMap.set(c.tag.toUpperCase(), c.badgeUrl);
            });
            crClans.forEach((c: any) => {
                if (c.badgeUrl) badgeMap.set(c.tag.toUpperCase(), c.badgeUrl);
            });

            const clansMap = new Map<
                string,
                { tag: string; name: string; gameType: string; index: number }
            >();

            // Process Clash of Clans accounts
            const cocAccounts =
                accounts.coc || (Array.isArray(accounts) ? accounts : []);
            cocAccounts.forEach((acc: any) => {
                if (acc.clan) {
                    const tag = acc.clan.tag.toUpperCase();
                    if (officialCocTags.has(tag)) {
                        const clanData = cocClans.find(
                            (c: any) => c.tag.toUpperCase() === tag,
                        );
                        clansMap.set(`coc-${tag}`, {
                            tag: acc.clan.tag,
                            name: acc.clan.name,
                            gameType: 'coc',
                            index: clanData?.index || 0,
                        });
                    }
                }
            });

            // Process Clash Royale accounts
            const crAccounts = accounts.cr || [];
            crAccounts.forEach((acc: any) => {
                if (acc.clan) {
                    const tag = acc.clan.tag.toUpperCase();
                    if (officialCrTags.has(tag)) {
                        const clanData = crClans.find(
                            (c: any) => c.tag.toUpperCase() === tag,
                        );
                        clansMap.set(`cr-${tag}`, {
                            tag: acc.clan.tag,
                            name: acc.clan.name,
                            gameType: 'cr',
                            index: clanData?.index || 0,
                        });
                    }
                }
            });

            userClans = Array.from(clansMap.values())
                .map((info) => ({
                    tag: info.tag,
                    name: info.name,
                    gameType: info.gameType,
                    badgeUrl: badgeMap.get(info.tag.toUpperCase()),
                    index: info.index,
                }))
                .sort((a, b) => {
                    const aIdx = a.index === 0 ? 1000 : a.index || 999;
                    const bIdx = b.index === 0 ? 1000 : b.index || 999;
                    return aIdx - bIdx;
                });
        } catch (error) {
            console.error('Failed to fetch user clans:', error);
        }
    }

    $: if ($user) {
        fetchUserClans();
    } else {
        userClans = [];
    }
</script>

<header class="header" class:light={theme === 'light'}>
    <div class="header-content">
        <button type="button" class="logo" on:click={() => navigate('home')}>
            <img src={logo} alt="LOST Family" class="logo-icon" />
            <span class="logo-text">LOST Family</span>
        </button>

        <nav
            class="nav"
            class:open={mobileMenuOpen}
            class:light={theme === 'light'}
        >
            <div class="nav-item dropdown">
                <button class="nav-link">
                    Clash of Clans
                    <svg
                        class="dropdown-arrow"
                        viewBox="0 0 20 20"
                        fill="currentColor"
                    >
                        <path
                            fill-rule="evenodd"
                            d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z"
                            clip-rule="evenodd"
                        />
                    </svg>
                </button>
                <div class="dropdown-menu">
                    <a
                        href="/coc/clans"
                        class="dropdown-item"
                        on:click|preventDefault={() => navigate('coc/clans')}
                        >Clans</a
                    >
                    <a
                        href="/coc/cwl"
                        class="dropdown-item"
                        on:click|preventDefault={() => navigate('coc/cwl')}
                        >Clankriegsliga</a
                    >
                </div>
            </div>

            <div class="nav-item dropdown">
                <button class="nav-link">
                    Clash Royale
                    <svg
                        class="dropdown-arrow"
                        viewBox="0 0 20 20"
                        fill="currentColor"
                    >
                        <path
                            fill-rule="evenodd"
                            d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z"
                            clip-rule="evenodd"
                        />
                    </svg>
                </button>
                <div class="dropdown-menu">
                    <a
                        href="/cr/clans"
                        class="dropdown-item"
                        on:click|preventDefault={() => navigate('cr/clans')}
                        >Clans</a
                    >
                </div>
            </div>
        </nav>

        <div class="header-actions">
            <ThemeToggle {theme} on:toggle={handleThemeToggle} />

            {#if $loading}
                <div class="auth-loading animate-pulse"></div>
            {:else if $user}
                <div class="user-profile dropdown pc-user">
                    <button class="user-btn">
                        <div class="avatar-wrapper">
                            {#if $user.avatar}
                                <img
                                    src={$user.avatar}
                                    alt={$user.nickname ||
                                        $user.global_name ||
                                        $user.username}
                                    class="user-avatar"
                                />
                            {:else}
                                <div class="user-avatar-placeholder">
                                    {(
                                        $user.nickname ||
                                        $user.global_name ||
                                        $user.username
                                    )
                                        .charAt(0)
                                        .toUpperCase()}
                                </div>
                            {/if}
                            <div class="status-indicator"></div>
                        </div>
                        <span class="username"
                            >{$user.nickname ||
                                $user.global_name ||
                                $user.username}</span
                        >
                        <svg
                            class="dropdown-arrow"
                            viewBox="0 0 20 20"
                            fill="currentColor"
                        >
                            <path
                                fill-rule="evenodd"
                                d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z"
                                clip-rule="evenodd"
                            />
                        </svg>
                    </button>

                    <div
                        class="dropdown-menu dropdown-menu-right user-dropdown"
                    >
                        <div class="user-dropdown-info">
                            <div class="user-dropdown-avatar">
                                {#if $user.avatar}
                                    <img
                                        src={$user.avatar}
                                        alt={$user.nickname ||
                                            $user.global_name ||
                                            $user.username}
                                    />
                                {:else}
                                    <div class="user-avatar-placeholder large">
                                        {(
                                            $user.nickname ||
                                            $user.global_name ||
                                            $user.username
                                        )
                                            .charAt(0)
                                            .toUpperCase()}
                                    </div>
                                {/if}
                            </div>
                            <div class="user-dropdown-details">
                                <div class="user-dropdown-name">
                                    {$user.nickname ||
                                        $user.global_name ||
                                        $user.username}
                                </div>
                                <div class="user-dropdown-id">
                                    @{$user.username}
                                </div>
                                <div class="user-dropdown-role-badge">
                                    {$user.highest_role || 'Member'}
                                </div>
                            </div>
                        </div>

                        <div class="dropdown-divider"></div>

                        <a
                            href="/account"
                            class="dropdown-item"
                            on:click|preventDefault={() => navigate('account')}
                        >
                            <svg
                                class="item-icon"
                                viewBox="0 0 24 24"
                                fill="none"
                                stroke="currentColor"
                                stroke-width="2"
                            >
                                <path
                                    d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"
                                /><circle cx="12" cy="7" r="4" />
                            </svg>
                            Verknüpfte Accounts
                        </a>

                        {#if userClans.length === 1}
                            <a
                                href="/{userClans[0]
                                    .gameType}/clan/{userClans[0].tag.replace(
                                    '#',
                                    '',
                                )}"
                                class="dropdown-item"
                                on:click|preventDefault={() =>
                                    navigate(
                                        `${userClans[0].gameType}/clan/${userClans[0].tag.replace(
                                            '#',
                                            '',
                                        )}`,
                                    )}
                            >
                                {#if userClans[0].badgeUrl}
                                    <img
                                        src={userClans[0].badgeUrl}
                                        alt={userClans[0].name}
                                        class="item-icon clan-badge-mini"
                                    />
                                {:else}
                                    <svg
                                        class="item-icon"
                                        viewBox="0 0 24 24"
                                        fill="none"
                                        stroke="currentColor"
                                        stroke-width="2"
                                    >
                                        <path
                                            d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"
                                        />
                                        <circle cx="9" cy="7" r="4" />
                                        <path d="M23 21v-2a4 4 0 0 0-3-3.87" />
                                        <path d="M16 3.13a4 4 0 0 1 0 7.75" />
                                    </svg>
                                {/if}
                                Dein Clan
                            </a>
                        {:else}
                            <a
                                href="/my-clans"
                                class="dropdown-item"
                                on:click|preventDefault={() =>
                                    navigate('my-clans')}
                            >
                                <svg
                                    class="item-icon"
                                    viewBox="0 0 24 24"
                                    fill="none"
                                    stroke="currentColor"
                                    stroke-width="2"
                                >
                                    <path
                                        d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"
                                    />
                                    <circle cx="9" cy="7" r="4" />
                                    <path d="M23 21v-2a4 4 0 0 0-3-3.87" />
                                    <path d="M16 3.13a4 4 0 0 1 0 7.75" />
                                </svg>
                                Deine Clans
                            </a>
                        {/if}

                        {#if $user.is_admin}
                            <a
                                href="/admin/clans"
                                class="dropdown-item admin-link"
                                on:click|preventDefault={() =>
                                    navigate('admin/clans')}
                            >
                                <svg
                                    class="item-icon"
                                    viewBox="0 0 24 24"
                                    fill="none"
                                    stroke="currentColor"
                                    stroke-width="2"
                                >
                                    <path
                                        d="M12 2L2 7l10 5 10-5-10-5zm0 9l2.5-1.25L12 8.5l-2.5 1.25L12 11zm0 2.5l-5-2.5-5 2.5L12 22l10-8.5-5-2.5-5 2.5z"
                                    />
                                </svg>
                                Alle Clans
                            </a>
                            <a
                                href="/admin"
                                class="dropdown-item admin-link"
                                on:click|preventDefault={() =>
                                    navigate('admin')}
                            >
                                <svg
                                    class="item-icon"
                                    viewBox="0 0 24 24"
                                    fill="none"
                                    stroke="currentColor"
                                    stroke-width="2"
                                >
                                    <path
                                        d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"
                                    />
                                </svg>
                                Admin Dashboard
                            </a>
                        {/if}

                        <div class="dropdown-divider"></div>

                        <button
                            on:click={logout}
                            class="dropdown-item logout-btn"
                        >
                            <svg
                                class="logout-icon"
                                viewBox="0 0 24 24"
                                fill="none"
                                stroke="currentColor"
                                stroke-width="2"
                            >
                                <path
                                    d="M9 21H5a2 2 0 01-2-2V5a2 2 0 012-2h4M16 17l5-5-5-5M21 12H9"
                                />
                            </svg>
                            Logout
                        </button>
                    </div>
                </div>
            {:else}
                <button class="login-btn pc-login" on:click={login}>
                    <svg
                        class="discord-icon"
                        viewBox="0 0 24 24"
                        fill="currentColor"
                    >
                        <path
                            d="M20.317 4.37a19.791 19.791 0 0 0-4.885-1.515.074.074 0 0 0-.079.037c-.21.375-.444.864-.608 1.25a18.27 18.27 0 0 0-5.487 0 12.64 12.64 0 0 0-.617-1.25.077.077 0 0 0-.079-.037A19.736 19.736 0 0 0 3.677 4.37a.07.07 0 0 0-.032.027C.533 9.046-.32 13.58.099 18.057a.082.082 0 0 0 .031.057 19.9 19.9 0 0 0 5.993 3.03.078.078 0 0 0 .084-.028 14.09 14.09 0 0 0 1.226-1.994.076.076 0 0 0-.041-.106 13.107 13.107 0 0 1-1.872-.892.077.077 0 0 1-.008-.128 10.2 10.2 0 0 0 .372-.292.074.074 0 0 1 .077-.01c3.928 1.793 8.18 1.793 12.062 0a.074.074 0 0 1 .078.01c.12.098.246.198.373.292a.077.077 0 0 1-.006.127 12.299 12.299 0 0 1-1.873.892.077.077 0 0 0-.041.107c.36.698.772 1.362 1.225 1.993a.076.076 0 0 0 .084.028 19.839 19.839 0 0 0 6.002-3.03.077.077 0 0 0 .032-.054c.5-5.177-.838-9.674-3.549-13.66a.061.061 0 0 0-.031-.03zM8.02 15.33c-1.183 0-2.157-1.085-2.157-2.419 0-1.333.956-2.419 2.157-2.419 1.21 0 2.176 1.096 2.157 2.42 0 1.333-.956 2.418-2.157 2.418zm7.975 0c-1.183 0-2.157-1.085-2.157-2.419 0-1.333.955-2.419 2.157-2.419 1.21 0 2.176 1.096 2.157 2.42 0 1.333-.946 2.418-2.157 2.418z"
                        />
                    </svg>
                    Login
                </button>
            {/if}

            <button
                class="mobile-menu-btn"
                on:click={toggleMobileMenu}
                aria-label="Toggle menu"
            >
                <svg
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                >
                    {#if mobileMenuOpen}
                        <path d="M6 18L18 6M6 6l12 12" />
                    {:else}
                        <path d="M4 6h16M4 12h16M4 18h16" />
                    {/if}
                </svg>
            </button>
        </div>
    </div>
</header>

<!-- Mobile Navigation Drawer -->
<div
    role="button"
    tabindex="0"
    class="mobile-backdrop"
    class:visible={mobileMenuOpen}
    on:click={toggleMobileMenu}
    on:keydown={handleBackdropKeydown}
    aria-label="Menü schließen"
></div>

<aside
    class="mobile-drawer"
    class:open={mobileMenuOpen}
    class:light={theme === 'light'}
>
    <div class="drawer-header">
        <button class="logo" on:click={() => navigate('home')}>
            <img src={logo} alt="LOST Family" class="logo-icon" />
            <span class="logo-text-drawer">LOST Family</span>
        </button>
        <button
            class="drawer-close"
            on:click={toggleMobileMenu}
            aria-label="Menü schließen"
        >
            <svg
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
            >
                <path d="M6 18L18 6M6 6l12 12" />
            </svg>
        </button>
    </div>

    <div class="drawer-content">
        {#if $user}
            <div class="drawer-user-section">
                <div class="drawer-user-info">
                    <div class="avatar-wrapper large">
                        {#if $user.avatar}
                            <img
                                src={$user.avatar}
                                alt={$user.username}
                                class="user-avatar-large"
                            />
                        {:else}
                            <div class="user-avatar-placeholder large">
                                {($user.nickname || $user.username)
                                    .charAt(0)
                                    .toUpperCase()}
                            </div>
                        {/if}
                        <div class="status-indicator"></div>
                    </div>
                    <div class="drawer-user-details">
                        <span class="drawer-username"
                            >{$user.nickname ||
                                $user.global_name ||
                                $user.username}</span
                        >
                        <span class="drawer-role"
                            >{$user.highest_role || 'Member'}</span
                        >
                    </div>
                </div>
            </div>
        {:else}
            <div class="drawer-login-section">
                <button class="login-btn full-width" on:click={login}>
                    <svg
                        class="discord-icon"
                        viewBox="0 0 24 24"
                        fill="currentColor"
                    >
                        <path
                            d="M20.317 4.37a19.791 19.791 0 0 0-4.885-1.515.074.074 0 0 0-.079.037c-.21.375-.444.864-.608 1.25a18.27 18.27 0 0 0-5.487 0 12.64 12.64 0 0 0-.617-1.25.077.077 0 0 0-.079-.037A19.736 19.736 0 0 0 3.677 4.37a.07.07 0 0 0-.032.027C.533 9.046-.32 13.58.099 18.057a.082.082 0 0 0 .031.057 19.9 19.9 0 0 0 5.993 3.03.078.078 0 0 0 .084-.028 14.09 14.09 0 0 0 1.226-1.994.076.076 0 0 0-.041-.106 13.107 13.107 0 0 1-1.872-.892.077.077 0 0 1-.008-.128 10.2 10.2 0 0 0 .372-.292.074.074 0 0 1 .077-.01c3.928 1.793 8.18 1.793 12.062 0a.074.074 0 0 1 .078.01c.12.098.246.198.373.292a.077.077 0 0 1-.006.127 12.299 12.299 0 0 1-1.873.892.077.077 0 0 0-.041.107c.36.698.772 1.362 1.225 1.993a.076.076 0 0 0 .084.028 19.839 19.839 0 0 0 6.002-3.03.077.077 0 0 0 .032-.054c.5-5.177-.838-9.674-3.549-13.66a.061.061 0 0 0-.031-.03zM8.02 15.33c-1.183 0-2.157-1.085-2.157-2.419 0-1.333.956-2.419 2.157-2.419 1.21 0 2.176 1.096 2.157 2.42 0 1.333-.956 2.418-2.157 2.418zm7.975 0c-1.183 0-2.157-1.085-2.157-2.419 0-1.333.955-2.419 2.157-2.419 1.21 0 2.176 1.096 2.157 2.42 0 1.333-.946 2.418-2.157 2.418z"
                        />
                    </svg>
                    Social Login
                </button>
            </div>
        {/if}

        <div class="drawer-section">
            <span class="section-title">Navigation</span>
            <div class="drawer-nav-item">
                <button
                    class="drawer-nav-link"
                    on:click={() => navigate('home')}>Home</button
                >
            </div>

            <div class="drawer-nav-group">
                <div class="group-header">Clash of Clans</div>
                <a
                    href="/coc/clans"
                    class="drawer-sub-link"
                    on:click|preventDefault={() => navigate('coc/clans')}
                    >Clans</a
                >
                <a
                    href="/coc/cwl"
                    class="drawer-sub-link"
                    on:click|preventDefault={() => navigate('coc/cwl')}
                    >Clankriegsliga</a
                >
            </div>

            <div class="drawer-nav-group">
                <div class="group-header">Clash Royale</div>
                <a
                    href="/cr/clans"
                    class="drawer-sub-link"
                    on:click|preventDefault={() => navigate('cr/clans')}
                    >Clans</a
                >
            </div>
        </div>

        {#if $user}
            <div class="drawer-section">
                <span class="section-title">Account</span>
                <a
                    href="/account"
                    class="drawer-nav-link"
                    on:click|preventDefault={() => navigate('account')}
                >
                    <div class="drawer-link-content">
                        <svg
                            class="drawer-item-icon"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="2"
                        >
                            <path
                                d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"
                            /><circle cx="12" cy="7" r="4" />
                        </svg>
                        Verknüpfte Accounts
                    </div>
                </a>
                {#if userClans.length === 1}
                    <a
                        href="/{userClans[0]
                            .gameType}/clan/{userClans[0].tag.replace('#', '')}"
                        class="drawer-nav-link"
                        on:click|preventDefault={() =>
                            navigate(
                                `${userClans[0].gameType}/clan/${userClans[0].tag.replace(
                                    '#',
                                    '',
                                )}`,
                            )}
                    >
                        <div class="drawer-link-content">
                            {#if userClans[0].badgeUrl}
                                <img
                                    src={userClans[0].badgeUrl}
                                    alt={userClans[0].name}
                                    class="drawer-item-icon clan-badge-mini"
                                />
                            {/if}
                            Dein Clan
                        </div>
                    </a>
                {:else}
                    <a
                        href="/my-clans"
                        class="drawer-nav-link"
                        on:click|preventDefault={() => navigate('my-clans')}
                    >
                        <div class="drawer-link-content">
                            <svg
                                class="drawer-item-icon"
                                viewBox="0 0 24 24"
                                fill="none"
                                stroke="currentColor"
                                stroke-width="2"
                            >
                                <path
                                    d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"
                                />
                                <circle cx="9" cy="7" r="4" />
                                <path d="M23 21v-2a4 4 0 0 0-3-3.87" />
                                <path d="M16 3.13a4 4 0 0 1 0 7.75" />
                            </svg>
                            Deine Clans
                        </div>
                    </a>
                {/if}

                {#if $user.is_admin}
                    <div class="drawer-nav-group admin">
                        <div class="group-header">Admin</div>
                        <button
                            class="drawer-sub-link admin-link"
                            on:click={() => navigate('admin/clans')}
                            >Alle Clans</button
                        >
                        <button
                            class="drawer-sub-link admin-link"
                            on:click={() => navigate('admin')}>Dashboard</button
                        >
                    </div>
                {/if}
            </div>
        {/if}
    </div>

    <div class="drawer-footer">
        <button
            class="logout-btn-drawer"
            on:click={() => {
                logout();
                mobileMenuOpen = false;
            }}
        >
            <svg
                class="logout-icon"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
            >
                <path
                    d="M9 21H5a2 2 0 01-2-2V5a2 2 0 012-2h4M16 17l5-5-5-5M21 12H9"
                />
            </svg>
            Logout
        </button>
    </div>
</aside>

<style>
    .header {
        position: sticky;
        top: 0;
        z-index: 100;
        background: rgba(10, 10, 15, 0.8);
        backdrop-filter: blur(20px);
        border-bottom: 1px solid rgba(255, 255, 255, 0.06);
        transition: all 0.4s ease;
    }

    .header.light {
        background: rgba(255, 255, 255, 0.8);
        border-bottom: 1px solid rgba(0, 0, 0, 0.06);
    }

    .header-content {
        max-width: 1400px;
        margin: 0 auto;
        padding: 0 2rem;
        height: 64px;
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 2rem;
    }

    .logo {
        display: flex;
        align-items: center;
        gap: 12px;
        text-decoration: none;
        flex-shrink: 0;
        background: transparent;
        border: none;
        padding: 0;
        cursor: pointer;
        font: inherit;
        outline: none;
    }

    .logo:focus {
        outline: none;
    }

    .logo-icon {
        width: 40px;
        height: 40px;
        border-radius: 10px;
        background: #1e1f22;
        padding: 4px;
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.4);
        transition: transform 0.3s ease;
    }

    .logo:hover .logo-icon {
        transform: scale(1.05);
    }

    .logo-text {
        font-size: 1.25rem;
        font-weight: 700;
        color: #fff;
        transition: color 0.4s ease;
    }

    .header.light .logo-text {
        color: #1a1a2e;
    }

    /* Navigation */
    .nav {
        display: flex;
        align-items: center;
        gap: 0.25rem;
    }

    .dropdown {
        position: relative;
    }

    .nav-link {
        display: flex;
        align-items: center;
        gap: 4px;
        padding: 10px 16px;
        color: rgba(255, 255, 255, 0.7);
        text-decoration: none;
        font-size: 0.9rem;
        font-weight: 500;
        border-radius: 8px;
        border: none;
        background: transparent;
        cursor: pointer;
        transition: all 0.2s ease;
        outline: none;
    }

    .nav-link:focus {
        outline: none;
    }

    .nav-link:hover {
        color: #fff;
        background: rgba(255, 255, 255, 0.1);
    }

    .nav.light .nav-link {
        color: rgba(0, 0, 0, 0.6);
    }

    .nav.light .nav-link:hover {
        color: #1a1a2e;
        background: rgba(0, 0, 0, 0.05);
    }

    .dropdown-arrow {
        width: 14px;
        height: 14px;
        transition: transform 0.2s ease;
        opacity: 0.6;
    }

    .dropdown:hover .dropdown-arrow {
        transform: rotate(180deg);
    }

    .dropdown-menu {
        position: absolute;
        top: calc(100% + 8px);
        left: 50%;
        transform: translateX(-50%) translateY(8px);
        min-width: 180px;
        max-height: calc(100vh - 80px);
        overflow-y: auto;
        padding: 8px;
        background: rgba(30, 32, 35, 0.98);
        border: 1px solid rgba(255, 255, 255, 0.08);
        border-radius: 12px;
        box-shadow: 0 16px 48px rgba(0, 0, 0, 0.5);
        opacity: 0;
        visibility: hidden;
        transition: all 0.25s cubic-bezier(0.4, 0, 0.2, 1);
    }

    .dropdown:hover .dropdown-menu {
        opacity: 1;
        visibility: visible;
        transform: translateX(-50%) translateY(0);
    }

    .header.light .dropdown-menu {
        background: rgba(255, 255, 255, 0.98);
        border: 1px solid rgba(0, 0, 0, 0.08);
        box-shadow: 0 16px 48px rgba(0, 0, 0, 0.12);
    }

    .dropdown-item {
        display: flex;
        align-items: center;
        gap: 10px;
        padding: 10px 14px;
        color: rgba(255, 255, 255, 0.8);
        text-decoration: none;
        font-size: 0.875rem;
        font-weight: 500;
        border-radius: 8px;
        transition: all 0.2s ease;
        outline: none;
        border: none;
        background: transparent;
        cursor: pointer;
        font-family: inherit;
        justify-content: flex-start;
    }

    .dropdown-item:focus {
        outline: none;
    }

    .dropdown-item:hover {
        color: #fff;
        background: rgba(88, 101, 242, 0.2);
    }

    .header.light .dropdown-item {
        color: rgba(0, 0, 0, 0.7);
    }

    .header.light .dropdown-item:hover {
        color: #1a1a2e;
        background: rgba(88, 101, 242, 0.1);
    }

    /* Header Actions */
    .header-actions {
        display: flex;
        align-items: center;
        gap: 1rem;
        flex-shrink: 0;
    }

    .auth-loading {
        width: 60px;
        height: 38px;
        background: rgba(255, 255, 255, 0.05);
        border-radius: 8px;
    }

    .login-btn {
        display: flex;
        align-items: center;
        gap: 0.5rem;
        padding: 0.5rem 1rem;
        background: #5865f2;
        color: white;
        border-radius: 8px;
        font-weight: 600;
        font-size: 0.9rem;
        transition: all 0.3s ease;
        border: none;
        cursor: pointer;
        outline: none;
    }

    .login-btn:focus {
        outline: none;
    }

    .login-btn:hover {
        background: #4752c4;
        transform: translateY(-2px);
        box-shadow: 0 4px 12px rgba(88, 101, 242, 0.3);
    }

    .discord-icon {
        width: 18px;
        height: 18px;
    }

    .user-btn {
        display: flex;
        align-items: center;
        gap: 10px;
        background: rgba(255, 255, 255, 0.03);
        border: 1px solid rgba(255, 255, 255, 0.08);
        padding: 6px 14px 6px 6px;
        border-radius: 100px;
        color: white;
        font-size: 0.9rem;
        font-weight: 500;
        cursor: pointer;
        transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
        outline: none;
    }

    .user-btn:focus {
        outline: none;
    }

    .header.light .user-btn {
        background: rgba(0, 0, 0, 0.03);
        border: 1px solid rgba(0, 0, 0, 0.08);
        color: #1e293b;
    }

    .user-btn:hover {
        background: rgba(255, 255, 255, 0.08);
        border-color: rgba(255, 255, 255, 0.15);
        transform: translateY(-1px);
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
    }

    .header.light .user-btn:hover {
        background: rgba(0, 0, 0, 0.06);
        border-color: rgba(0, 0, 0, 0.12);
        box-shadow: 0 4px 12px rgba(0, 0, 0, 0.05);
    }

    .avatar-wrapper {
        position: relative;
        display: flex;
    }

    .status-indicator {
        position: absolute;
        bottom: 0;
        right: 0;
        width: 10px;
        height: 10px;
        background: #22c55e;
        border: 2px solid #0a0a0f;
        border-radius: 50%;
    }

    .header.light .status-indicator {
        border-color: #fff;
    }

    .user-avatar {
        width: 32px;
        height: 32px;
        border-radius: 50%;
        object-fit: cover;
        border: 1px solid rgba(255, 255, 255, 0.1);
    }

    .user-avatar-placeholder {
        width: 32px;
        height: 32px;
        border-radius: 50%;
        background: linear-gradient(135deg, #5865f2, #4752c4);
        color: white;
        display: flex;
        align-items: center;
        justify-content: center;
        font-size: 0.85rem;
        font-weight: 700;
        text-shadow: 0 1px 2px rgba(0, 0, 0, 0.2);
    }

    .user-avatar-placeholder.large {
        width: 48px;
        height: 48px;
        font-size: 1.25rem;
        border-radius: 12px;
    }

    .user-dropdown {
        width: 260px;
        max-width: calc(100vw - 32px);
        padding: 12px;
    }

    .user-dropdown-info {
        display: flex;
        align-items: center;
        gap: 12px;
        padding: 8px 12px 16px;
        min-width: 0;
    }

    .user-dropdown-avatar img {
        width: 48px;
        height: 48px;
        border-radius: 12px;
        object-fit: cover;
    }

    .user-dropdown-details {
        display: flex;
        flex-direction: column;
        gap: 2px;
        min-width: 0;
    }

    .user-dropdown-name {
        font-weight: 700;
        color: #fff;
        font-size: 1rem;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .header.light .user-dropdown-name {
        color: #1a1a2e;
    }

    .user-dropdown-id {
        font-size: 0.75rem;
        color: rgba(255, 255, 255, 0.4);
        font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas,
            monospace;
    }

    .header.light .user-dropdown-id {
        color: rgba(0, 0, 0, 0.4);
    }

    .user-dropdown-role-badge {
        display: inline-flex;
        padding: 2px 8px;
        background: rgba(88, 101, 242, 0.15);
        color: #5865f2;
        border-radius: 4px;
        font-size: 0.65rem;
        font-weight: 700;
        text-transform: uppercase;
        letter-spacing: 0.02em;
        margin-top: 4px;
        width: fit-content;
    }

    .dropdown-divider {
        height: 1px;
        background: rgba(255, 255, 255, 0.08);
        margin: 8px -4px;
    }

    .header.light .dropdown-divider {
        background: rgba(0, 0, 0, 0.08);
    }

    .item-icon {
        width: 18px;
        height: 18px;
        opacity: 0.6;
        transition: opacity 0.2s ease;
    }

    .clan-badge-mini {
        opacity: 1 !important;
    }

    .dropdown-item:hover .item-icon {
        opacity: 1;
    }

    .admin-link {
        color: #10b981 !important;
    }

    .admin-link:hover {
        background: rgba(16, 185, 129, 0.1) !important;
    }

    .animate-pulse {
        animation: pulse 1.5s cubic-bezier(0.4, 0, 0.6, 1) infinite;
    }

    @keyframes pulse {
        0%,
        100% {
            opacity: 1;
        }
        50% {
            opacity: 0.5;
        }
    }

    .dropdown-menu-right {
        left: auto !important;
        right: 0 !important;
        transform: translateY(8px) !important;
    }

    .user-profile:hover .dropdown-menu-right {
        transform: translateY(0) !important;
    }

    .logout-btn {
        width: 100%;
        text-align: left;
        color: #f87171 !important;
        font-weight: 600;
        display: flex;
        align-items: center;
        gap: 10px;
        background: transparent;
        border: none;
        cursor: pointer;
        font-family: inherit;
        margin-top: 4px;
        outline: none;
    }

    .logout-btn:focus {
        outline: none;
    }

    .header.light .logout-btn {
        color: #dc2626 !important;
    }

    .logout-icon {
        width: 18px;
        height: 18px;
        opacity: 0.8;
    }

    .logout-btn:hover {
        background: rgba(248, 113, 113, 0.15) !important;
        color: #ff8787 !important;
    }

    .header.light .logout-btn:hover {
        background: rgba(220, 38, 38, 0.08) !important;
        color: #b91c1c !important;
    }

    .header-actions :global(.theme-toggle) {
        position: static;
    }

    .mobile-menu-btn {
        display: none;
        width: 40px;
        height: 40px;
        padding: 8px;
        background: transparent;
        border: none;
        border-radius: 8px;
        color: rgba(255, 255, 255, 0.7);
        cursor: pointer;
        transition: all 0.2s ease;
        outline: none;
    }

    .mobile-menu-btn:focus {
        outline: none;
    }

    .mobile-menu-btn:hover {
        background: rgba(255, 255, 255, 0.1);
        color: #fff;
    }

    .header.light .mobile-menu-btn {
        color: rgba(0, 0, 0, 0.6);
    }

    .header.light .mobile-menu-btn:hover {
        background: rgba(0, 0, 0, 0.05);
        color: #1a1a2e;
    }

    .mobile-menu-btn svg {
        width: 100%;
        height: 100%;
    }

    :global(body.no-scroll) {
        overflow: hidden !important;
    }

    .mobile-backdrop {
        position: fixed;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        background: rgba(0, 0, 0, 0.4);
        backdrop-filter: blur(4px);
        z-index: 1000;
        opacity: 0;
        visibility: hidden;
        transition: all 0.3s ease;
    }

    .mobile-backdrop.visible {
        opacity: 1;
        visibility: visible;
    }

    .mobile-drawer {
        position: fixed;
        top: 0;
        left: 0;
        bottom: 0;
        width: 300px;
        max-width: 85vw;
        background: #0a0a0f !important;
        z-index: 2000;
        transform: translateX(-100%);
        transition: transform 0.3s cubic-bezier(0.4, 0, 0.2, 1);
        display: flex;
        flex-direction: column;
        box-shadow: 20px 0 50px rgba(0, 0, 0, 0.8);
        border-right: 1px solid rgba(255, 255, 255, 0.1);
    }

    .mobile-drawer.light {
        background: #ffffff !important;
        border-right: 1px solid rgba(0, 0, 0, 0.1);
    }

    .mobile-drawer.open {
        transform: translateX(0);
    }

    .drawer-header {
        padding: 1.25rem;
        display: flex;
        align-items: center;
        justify-content: space-between;
        border-bottom: 1px solid rgba(255, 255, 255, 0.06);
    }

    .mobile-drawer.light .drawer-header {
        border-bottom-color: rgba(0, 0, 0, 0.06);
    }

    .logo-text-drawer {
        font-size: 1.1rem;
        font-weight: 700;
        color: #fff;
    }

    .mobile-drawer.light .logo-text-drawer {
        color: #1a1a2e;
    }

    .drawer-close {
        width: 36px;
        height: 36px;
        padding: 8px;
        background: rgba(255, 255, 255, 0.05);
        border: none;
        border-radius: 8px;
        color: #fff;
        cursor: pointer;
    }

    .mobile-drawer.light .drawer-close {
        background: rgba(0, 0, 0, 0.05);
        color: #1a1a2e;
    }

    .drawer-content {
        flex: 1;
        overflow-y: auto;
        overflow-x: hidden;
        -webkit-overflow-scrolling: touch;
        padding: 1rem 0;
        display: flex;
        flex-direction: column;
        gap: 1.5rem;
    }

    .drawer-user-section,
    .drawer-login-section {
        padding: 0 1.25rem;
    }

    .drawer-user-info {
        display: flex;
        align-items: center;
        gap: 1rem;
        padding: 1rem;
        background: rgba(255, 255, 255, 0.03);
        border: 1px solid rgba(255, 255, 255, 0.06);
        border-radius: 12px;
    }

    .mobile-drawer.light .drawer-user-info {
        background: rgba(0, 0, 0, 0.02);
        border-color: rgba(0, 0, 0, 0.06);
    }

    .user-avatar-large {
        width: 48px;
        height: 48px;
        border-radius: 12px;
        object-fit: cover;
    }

    .avatar-wrapper.large .status-indicator {
        width: 14px;
        height: 14px;
        border-width: 3px;
    }

    .drawer-user-details {
        display: flex;
        flex-direction: column;
        min-width: 0;
    }

    .drawer-username {
        font-weight: 700;
        color: #fff;
        font-size: 1rem;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .mobile-drawer.light .drawer-username {
        color: #1a1a2e;
    }

    .drawer-role {
        font-size: 0.75rem;
        color: #5865f2;
        font-weight: 600;
        text-transform: uppercase;
    }

    .drawer-section {
        display: flex;
        flex-direction: column;
        gap: 4px;
    }

    .section-title {
        padding: 0 1.5rem;
        font-size: 0.7rem;
        font-weight: 800;
        color: rgba(255, 255, 255, 0.3);
        text-transform: uppercase;
        letter-spacing: 0.1em;
        margin-bottom: 4px;
    }

    .mobile-drawer.light .section-title {
        color: rgba(0, 0, 0, 0.4);
    }

    .drawer-nav-link {
        display: block;
        padding: 0.75rem 1.5rem;
        color: rgba(255, 255, 255, 0.7);
        text-decoration: none;
        font-size: 0.95rem;
        font-weight: 500;
        text-align: left;
        background: transparent;
        border: none;
        width: 100%;
        cursor: pointer;
        transition: all 0.2s ease;
    }

    .mobile-drawer.light .drawer-nav-link {
        color: rgba(0, 0, 0, 0.7);
    }

    .drawer-nav-link:hover {
        color: #fff;
        background: rgba(255, 255, 255, 0.05);
    }

    .drawer-link-content {
        display: flex;
        align-items: center;
        gap: 12px;
    }

    .drawer-item-icon {
        width: 20px;
        height: 20px;
        object-fit: contain;
    }

    .mobile-drawer.light .drawer-nav-link:hover {
        color: #1a1a2e;
        background: rgba(0, 0, 0, 0.05);
    }

    .drawer-nav-group {
        margin-top: 8px;
        display: flex;
        flex-direction: column;
    }

    .group-header {
        padding: 0.5rem 1.5rem;
        font-size: 0.85rem;
        font-weight: 700;
        color: #fff;
        opacity: 0.9;
    }

    .mobile-drawer.light .group-header {
        color: #1a1a2e;
    }

    .drawer-sub-link {
        padding: 0.6rem 2.5rem;
        color: rgba(255, 255, 255, 0.5);
        text-decoration: none;
        font-size: 0.9rem;
        transition: all 0.2s ease;
        text-align: left;
        background: transparent;
        border: none;
        width: 100%;
        cursor: pointer;
    }

    .mobile-drawer.light .drawer-sub-link {
        color: rgba(0, 0, 0, 0.6);
    }

    .drawer-sub-link:hover {
        color: #5865f2;
        padding-left: 2.75rem;
    }

    .drawer-nav-group.admin .group-header {
        color: #10b981;
    }

    .drawer-footer {
        padding: 1.25rem;
        border-top: 1px solid rgba(255, 255, 255, 0.06);
    }

    .mobile-drawer.light .drawer-footer {
        border-top-color: rgba(0, 0, 0, 0.06);
    }

    .logout-btn-drawer {
        display: flex;
        align-items: center;
        gap: 12px;
        width: 100%;
        padding: 0.75rem 1rem;
        background: rgba(248, 113, 113, 0.1);
        border: 1px solid rgba(248, 113, 113, 0.2);
        border-radius: 10px;
        color: #f87171;
        font-weight: 600;
        cursor: pointer;
        transition: all 0.2s ease;
    }

    .logout-btn-drawer:hover {
        background: rgba(248, 113, 113, 0.2);
        transform: translateY(-1px);
    }

    .full-width {
        width: 100%;
        justify-content: center;
    }

    /* Mobile Styles */
    @media (max-width: 900px) {
        .nav {
            display: none;
        }

        .pc-user,
        .pc-login {
            display: none;
        }

        .mobile-menu-btn {
            display: flex;
            align-items: center;
            justify-content: center;
        }
    }

    @media (max-width: 800px) {
        .header-content {
            padding: 0 1rem;
        }

        .logo-text {
            display: none;
        }
    }

    @media (max-width: 600px) {
        .username {
            display: none;
        }

        .user-btn {
            padding: 6px;
            gap: 0;
        }

        .user-btn .dropdown-arrow {
            display: none;
        }

        .header-actions {
            gap: 0.5rem;
        }
    }
</style>
