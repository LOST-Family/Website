<script lang="ts">
    import { onMount } from 'svelte';
    import { user, loading, login, fetchUser } from './lib/auth';
    import type { GameType } from './lib/auth';

    // Components
    import Header from './lib/Header.svelte';
    import Hero from './lib/Hero.svelte';
    import DiscordSection from './lib/DiscordSection.svelte';
    import FeaturesSection from './lib/FeaturesSection.svelte';
    import ClansSection from './lib/ClansSection.svelte';
    import ClansPage from './lib/ClansPage.svelte';
    import MyClansPage from './lib/MyClansPage.svelte';
    import AllClansAdminPage from './lib/AllClansAdminPage.svelte';
    import ProfilePage from './lib/ProfilePage.svelte';
    import AdminPage from './lib/AdminPage.svelte';
    import Footer from './lib/Footer.svelte';

    // Logos
    import lostLogo from './assets/Logos/Lost.png';

    // Banners
    import lostBanner from './assets/Assets/banner-lost.png';

    // Assets for features
    import kothImage from './assets/Assets/koth.png';
    import pushEventImage from './assets/Assets/push-event.png';
    import bigFarmImage from './assets/Assets/BIG-FARM.png';
    import twoVtwoImage from './assets/Assets/2v2.png';

    let theme: 'dark' | 'light' = 'dark';
    let mounted = false;
    let currentPath = window.location.pathname;
    let lastClansPath =
        currentPath === '/my-clans'
            ? '/my-clans'
            : currentPath === '/cr/clans'
              ? '/cr/clans'
              : currentPath === '/admin/clans'
                ? '/admin/clans'
                : '/coc/clans';

    const apiBaseUrl =
        import.meta.env.VITE_API_BASE_URL !== undefined
            ? import.meta.env.VITE_API_BASE_URL
            : 'http://localhost:8888';

    onMount(() => {
        mounted = true;
        fetchUser();

        // Simple SPA routing
        const handlePopState = () => {
            currentPath = window.location.pathname;
        };

        window.addEventListener('popstate', handlePopState);
        return () => window.removeEventListener('popstate', handlePopState);
    });

    function handleThemeToggle(
        event: CustomEvent<{ theme: 'dark' | 'light' }>
    ) {
        theme = event.detail.theme;
    }

    function handleNavigate(event: CustomEvent<string>) {
        const newPath = event.detail === 'home' ? '/' : `/${event.detail}`;

        // Track the last clans list page visited
        if (
            newPath === '/coc/clans' ||
            newPath === '/cr/clans' ||
            newPath === '/my-clans' ||
            newPath === '/admin/clans'
        ) {
            lastClansPath = newPath;
        }

        if (currentPath !== newPath) {
            window.history.pushState({}, '', newPath);
            currentPath = newPath;
            window.scrollTo({ top: 0, behavior: 'smooth' });
        }
    }
</script>

<div class="app" class:light={theme === 'light'} class:mounted>
    <Header
        {theme}
        logo={lostLogo}
        on:themeToggle={handleThemeToggle}
        on:navigate={handleNavigate}
    />

    <main>
        {#if currentPath === '/' || currentPath === ''}
            <Hero banner={lostBanner} {theme} {mounted} />

            <DiscordSection {theme} fallbackIcon={lostLogo} />

            <ClansSection
                {theme}
                {apiBaseUrl}
                gameType="coc"
                title="Clash of Clans"
                description="Unsere Clans in Clash of Clans"
            />

            <ClansSection
                {theme}
                {apiBaseUrl}
                gameType="cr"
                title="Clash Royale"
                description="Unsere Clans in Clash Royale"
            />

            <FeaturesSection
                {theme}
                {kothImage}
                {pushEventImage}
                {bigFarmImage}
                {twoVtwoImage}
            />
        {:else if currentPath === '/coc/clans'}
            <ClansPage
                {theme}
                {apiBaseUrl}
                gameType="coc"
                on:navigate={handleNavigate}
            />
        {:else if currentPath === '/cr/clans'}
            <ClansPage
                {theme}
                {apiBaseUrl}
                gameType="cr"
                on:navigate={handleNavigate}
            />
        {:else if currentPath === '/my-clans'}
            <MyClansPage {theme} {apiBaseUrl} on:navigate={handleNavigate} />
        {:else if currentPath === '/account'}
            <ProfilePage {theme} {apiBaseUrl} />
        {:else if currentPath.startsWith('/profile/')}
            <ProfilePage
                {theme}
                {apiBaseUrl}
                viewUserId={currentPath.split('/').pop()}
            />
        {:else if currentPath === '/admin'}
            <AdminPage {theme} {apiBaseUrl} />
        {:else if currentPath === '/admin/clans'}
            <AllClansAdminPage
                {theme}
                {apiBaseUrl}
                on:navigate={handleNavigate}
            />
        {:else if currentPath.startsWith('/coc/clan/')}
            {#if $loading}
                <div class="auth-message-container">
                    <div class="spinner"></div>
                </div>
            {:else if !$user}
                <div class="auth-message-container">
                    <div class="auth-card">
                        <div class="auth-icon">🔒</div>
                        <h2>Login erforderlich</h2>
                        <p>
                            Du musst eingeloggt sein, um Clan-Details und
                            Member-Listen zu sehen.
                        </p>
                        <button class="login-button" on:click={login}>
                            <svg viewBox="0 0 24 24" fill="currentColor"
                                ><path
                                    d="M20.317 4.37a19.791 19.791 0 0 0-4.885-1.515.074.074 0 0 0-.079.037c-.21.375-.444.864-.608 1.25a18.27 18.27 0 0 0-5.487 0 12.64 12.64 0 0 0-.617-1.25.077.077 0 0 0-.079-.037A19.736 19.736 0 0 0 3.677 4.37a.07.07 0 0 0-.032.027C.533 9.046-.32 13.58.099 18.057a.082.082 0 0 0 .031.057 19.9 19.9 0 0 0 5.993 3.03.078.078 0 0 0 .084-.028 14.09 14.09 0 0 0 1.226-1.994.076.076 0 0 0-.041-.106 13.107 13.107 0 0 1-1.872-.892.077.077 0 0 1-.008-.128 10.2 10.2 0 0 0 .372-.292.074.074 0 0 1 .077-.01c3.928 1.793 8.18 1.793 12.062 0a.074.074 0 0 1 .078.01c.12.098.246.198.373.292a.077.077 0 0 1-.006.127 12.299 12.299 0 0 1-1.873.892.077.077 0 0 0-.041.107c.36.698.772 1.362 1.225 1.993a.076.076 0 0 0 .084.028 19.839 19.839 0 0 0 6.002-3.03.077.077 0 0 0 .032-.054c.5-5.177-.838-9.674-3.549-13.66a.061.061 0 0 0-.031-.03z"
                                /></svg
                            >
                            Mit Discord anmelden
                        </button>
                    </div>
                </div>
            {:else}
                <ClansPage
                    {theme}
                    {apiBaseUrl}
                    gameType="coc"
                    clanTag={'#' + currentPath.split('/')[3]}
                    backPath={lastClansPath}
                    on:navigate={handleNavigate}
                />
            {/if}
        {:else if currentPath.startsWith('/cr/clan/')}
            {#if $loading}
                <div class="auth-message-container">
                    <div class="spinner"></div>
                </div>
            {:else if !$user}
                <div class="auth-message-container">
                    <div class="auth-card">
                        <div class="auth-icon">🔒</div>
                        <h2>Login erforderlich</h2>
                        <p>
                            Du musst eingeloggt sein, um Clan-Details und
                            Member-Listen zu sehen.
                        </p>
                        <button class="login-button" on:click={login}>
                            <svg viewBox="0 0 24 24" fill="currentColor"
                                ><path
                                    d="M20.317 4.37a19.791 19.791 0 0 0-4.885-1.515.074.074 0 0 0-.079.037c-.21.375-.444.864-.608 1.25a18.27 18.27 0 0 0-5.487 0 12.64 12.64 0 0 0-.617-1.25.077.077 0 0 0-.079-.037A19.736 19.736 0 0 0 3.677 4.37a.07.07 0 0 0-.032.027C.533 9.046-.32 13.58.099 18.057a.082.082 0 0 0 .031.057 19.9 19.9 0 0 0 5.993 3.03.078.078 0 0 0 .084-.028 14.09 14.09 0 0 0 1.226-1.994.076.076 0 0 0-.041-.106 13.107 13.107 0 0 1-1.872-.892.077.077 0 0 1-.008-.128 10.2 10.2 0 0 0 .372-.292.074.074 0 0 1 .077-.01c3.928 1.793 8.18 1.793 12.062 0a.074.074 0 0 1 .078.01c.12.098.246.198.373.292a.077.077 0 0 1-.006.127 12.299 12.299 0 0 1-1.873.892.077.077 0 0 0-.041.107c.36.698.772 1.362 1.225 1.993a.076.076 0 0 0 .084.028 19.839 19.839 0 0 0 6.002-3.03.077.077 0 0 0 .032-.054c.5-5.177-.838-9.674-3.549-13.66a.061.061 0 0 0-.031-.03z"
                                /></svg
                            >
                            Mit Discord anmelden
                        </button>
                    </div>
                </div>
            {:else}
                <ClansPage
                    {theme}
                    {apiBaseUrl}
                    gameType="cr"
                    clanTag={'#' + currentPath.split('/')[3]}
                    backPath={lastClansPath}
                    on:navigate={handleNavigate}
                />
            {/if}
        {:else if currentPath.startsWith('/clan/')}
            <!-- Legacy route - redirect to CoC -->
            {#if $loading}
                <div class="auth-message-container">
                    <div class="spinner"></div>
                </div>
            {:else if !$user}
                <div class="auth-message-container">
                    <div class="auth-card">
                        <div class="auth-icon">🔒</div>
                        <h2>Login erforderlich</h2>
                        <p>
                            Du musst eingeloggt sein, um Clan-Details und
                            Member-Listen zu sehen.
                        </p>
                        <button class="login-button" on:click={login}>
                            <svg viewBox="0 0 24 24" fill="currentColor"
                                ><path
                                    d="M20.317 4.37a19.791 19.791 0 0 0-4.885-1.515.074.074 0 0 0-.079.037c-.21.375-.444.864-.608 1.25a18.27 18.27 0 0 0-5.487 0 12.64 12.64 0 0 0-.617-1.25.077.077 0 0 0-.079-.037A19.736 19.736 0 0 0 3.677 4.37a.07.07 0 0 0-.032.027C.533 9.046-.32 13.58.099 18.057a.082.082 0 0 0 .031.057 19.9 19.9 0 0 0 5.993 3.03.078.078 0 0 0 .084-.028 14.09 14.09 0 0 0 1.226-1.994.076.076 0 0 0-.041-.106 13.107 13.107 0 0 1-1.872-.892.077.077 0 0 1-.008-.128 10.2 10.2 0 0 0 .372-.292.074.074 0 0 1 .077-.01c3.928 1.793 8.18 1.793 12.062 0a.074.074 0 0 1 .078.01c.12.098.246.198.373.292a.077.077 0 0 1-.006.127 12.299 12.299 0 0 1-1.873.892.077.077 0 0 0-.041.107c.36.698.772 1.362 1.225 1.993a.076.076 0 0 0 .084.028 19.839 19.839 0 0 0 6.002-3.03.077.077 0 0 0 .032-.054c.5-5.177-.838-9.674-3.549-13.66a.061.061 0 0 0-.031-.03z"
                                /></svg
                            >
                            Mit Discord anmelden
                        </button>
                    </div>
                </div>
            {:else}
                <ClansPage
                    {theme}
                    {apiBaseUrl}
                    gameType="coc"
                    clanTag={'#' + currentPath.split('/')[2]}
                    backPath={lastClansPath}
                    on:navigate={handleNavigate}
                />
            {/if}
        {/if}

        <Footer {theme} logo={lostLogo} />
    </main>
</div>

<style>
    :global(body) {
        margin: 0;
        font-family:
            'Inter',
            system-ui,
            -apple-system,
            sans-serif;
        overflow-x: hidden;
    }

    .app {
        min-height: 100vh;
        background: linear-gradient(
            180deg,
            #0a0a0f 0%,
            #12121a 50%,
            #0f1419 100%
        );
        background-attachment: fixed;
        transition: all 0.5s cubic-bezier(0.4, 0, 0.2, 1);
    }

    .app.light {
        background: linear-gradient(
            180deg,
            #f8fafc 0%,
            #f1f5f9 50%,
            #e2e8f0 100%
        );
    }

    .app::before {
        content: '';
        position: fixed;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        background: radial-gradient(
                ellipse 80% 50% at 20% 20%,
                rgba(88, 101, 242, 0.12) 0%,
                transparent 50%
            ),
            radial-gradient(
                ellipse 60% 40% at 80% 80%,
                rgba(59, 165, 92, 0.08) 0%,
                transparent 50%
            ),
            radial-gradient(
                ellipse 50% 30% at 50% 50%,
                rgba(255, 183, 77, 0.05) 0%,
                transparent 50%
            );
        pointer-events: none;
        z-index: 0;
        transition: opacity 0.5s ease;
    }

    .app.light::before {
        background: radial-gradient(
                ellipse 80% 50% at 20% 20%,
                rgba(88, 101, 242, 0.08) 0%,
                transparent 50%
            ),
            radial-gradient(
                ellipse 60% 40% at 80% 80%,
                rgba(59, 165, 92, 0.06) 0%,
                transparent 50%
            );
    }

    main {
        position: relative;
        z-index: 1;
    }

    /* Auth Message */
    .auth-message-container {
        min-height: 60vh;
        display: flex;
        align-items: center;
        justify-content: center;
        padding: 2rem;
    }

    .auth-card {
        background: var(--bg-secondary, rgba(30, 41, 59, 0.7));
        backdrop-filter: blur(12px);
        padding: 3rem;
        border-radius: 24px;
        text-align: center;
        max-width: 400px;
        border: 1px solid rgba(255, 255, 255, 0.1);
        box-shadow: 0 20px 50px rgba(0, 0, 0, 0.3);
    }

    .auth-icon {
        font-size: 3rem;
        margin-bottom: 1.5rem;
    }

    .auth-card h2 {
        font-size: 1.8rem;
        font-weight: 800;
        margin-bottom: 1rem;
        color: #fff;
    }

    .app.light .auth-card h2 {
        color: #1e293b;
    }

    .auth-card p {
        color: rgba(255, 255, 255, 0.6);
        margin-bottom: 2rem;
        line-height: 1.6;
    }

    .app.light .auth-card p {
        color: #64748b;
    }

    .login-button {
        display: flex;
        align-items: center;
        justify-content: center;
        gap: 0.8rem;
        width: 100%;
        padding: 1rem;
        background: #5865f2;
        color: white;
        border: none;
        border-radius: 12px;
        font-weight: 700;
        font-size: 1rem;
        cursor: pointer;
        transition: all 0.2s;
    }

    .login-button:hover {
        background: #4752c4;
        transform: translateY(-2px);
        box-shadow: 0 8px 15px rgba(88, 101, 242, 0.3);
    }

    .login-button svg {
        width: 20px;
        height: 20px;
    }

    .spinner {
        width: 40px;
        height: 40px;
        border: 3px solid rgba(88, 101, 242, 0.2);
        border-top-color: #5865f2;
        border-radius: 50%;
        animation: spin 1s linear infinite;
    }

    @keyframes spin {
        to {
            transform: rotate(360deg);
        }
    }
</style>
