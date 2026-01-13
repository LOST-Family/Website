<script lang="ts">
    export let theme: 'dark' | 'light' = 'dark';
    export let logo: string;
</script>

<footer class="footer" class:light={theme === 'light'}>
    <div class="footer-content">
        <div class="footer-brand">
            <img src={logo} alt="LOST Family" class="footer-logo" />
            <span class="footer-name">LOST Family</span>
        </div>
        <p class="footer-text">© 2026 LOST Family · Clash of Clans Alliance</p>
    </div>
</footer>

<style>
    .footer {
        margin-top: 4rem;
        padding: 3rem 2rem;
        border-top: 1px solid rgba(255, 255, 255, 0.06);
        background: rgba(0, 0, 0, 0.2);
        transition: all 0.4s ease;
    }

    .footer.light {
        border-top: 1px solid rgba(0, 0, 0, 0.06);
        background: rgba(0, 0, 0, 0.02);
    }

    .footer-content {
        max-width: 1400px;
        margin: 0 auto;
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 1rem;
    }

    .footer-brand {
        display: flex;
        align-items: center;
        gap: 0.75rem;
    }

    .footer-logo {
        width: 32px;
        height: 32px;
        border-radius: 8px;
        background: #1e1f22;
        padding: 4px;
    }

    .footer-name {
        font-size: 1.1rem;
        font-weight: 700;
        color: #fff;
        transition: color 0.4s ease;
    }

    .footer.light .footer-name {
        color: #0f172a;
    }

    .footer-text {
        margin: 0;
        font-size: 0.875rem;
        color: rgba(255, 255, 255, 0.4);
        transition: color 0.4s ease;
    }

    .footer.light .footer-text {
        color: #94a3b8;
    }

    @media (max-width: 800px) {
        .footer {
            padding: 2rem 1rem;
        }
    }
</style>
