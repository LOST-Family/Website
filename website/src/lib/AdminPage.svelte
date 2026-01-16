<script lang="ts">
    import { onMount } from 'svelte';
    import { user, loading as authLoading } from './auth';

    export let theme: 'dark' | 'light' = 'dark';
    export let apiBaseUrl: string = '';

    interface ServiceStatus {
        status: string;
        latency: number;
        uptime_minutes: number;
    }

    let statusData: {
        upstream_coc: ServiceStatus;
        upstream_cr: ServiceStatus;
        supercell_coc: ServiceStatus;
        supercell_cr: ServiceStatus;
        website: ServiceStatus;
    } | null = null;
    let latencyHistory: { api: string; latency: number; timestamp: number }[] =
        [];
    let loading = false;
    let error: string | null = null;

    async function fetchData() {
        if (loading) return;
        loading = true;
        error = null;
        try {
            const [statusRes, latencyRes] = await Promise.all([
                fetch(`${apiBaseUrl}/api/admin/status`, {
                    credentials: 'include',
                }),
                fetch(`${apiBaseUrl}/api/admin/latency`, {
                    credentials: 'include',
                }),
            ]);

            if (statusRes.ok) {
                statusData = await statusRes.json();
            } else if (statusRes.status === 403) {
                error =
                    'Zugriff verweigert: Administrator-Rechte erforderlich.';
            } else {
                error = `Fehler beim Abrufen des Status (HTTP ${statusRes.status})`;
            }

            if (latencyRes.ok) {
                latencyHistory = await latencyRes.json();
            }
        } catch (e) {
            console.error('Failed to fetch data:', e);
            error =
                'Netzwerkfehler beim Abrufen der Daten. Ist das Backend erreichbar?';
        } finally {
            loading = false;
        }
    }

    onMount(() => {
        if (!$authLoading && $user?.is_admin) {
            fetchData();
        }
    });

    $: if (!$authLoading && $user?.is_admin && !statusData && !error && !loading) {
        fetchData();
    }

    $: if (!$authLoading && !$user?.is_admin) {
        error = 'Administrator-Zutritt verweigert.';
    }

    function getStatusColor(status: string | undefined) {
        if (status === 'ONLINE') return 'text-green-500';
        if (status === 'OFFLINE') return 'text-red-500';
        return 'text-gray-500';
    }

    $: upstreamCocData = latencyHistory.filter(
        (d) => d.api === 'upstream_coc' && d.latency !== -1
    );
    $: upstreamCrData = latencyHistory.filter(
        (d) => d.api === 'upstream_cr' && d.latency !== -1
    );
    $: supercellCocData = latencyHistory.filter(
        (d) => d.api === 'supercell_coc' && d.latency !== -1
    );
    $: supercellCrData = latencyHistory.filter(
        (d) => d.api === 'supercell_cr' && d.latency !== -1
    );
    $: websiteData = latencyHistory.filter(
        (d) => d.api === 'website' && d.latency !== -1
    );

    let hoveredPoint: {
        api: string;
        latency: number;
        timestamp: number;
    } | null = null;
    let hoverX = 0;
    let hoverY = 0;

    const SVG_WIDTH = 400;
    const SVG_HEIGHT = 100;

    function handleMouseMove(
        e: MouseEvent,
        data: { latency: number; timestamp: number; api: string }[],
        api: string
    ) {
        if (!data.length) return;

        const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
        const x = e.clientX - rect.left;
        const width = rect.width;

        const minTime = Math.min(...data.map((d) => d.timestamp));
        const maxTime = Math.max(...data.map((d) => d.timestamp));
        const timeRange = maxTime - minTime || 1;

        const hoveredTimestamp = minTime + (x / width) * timeRange;

        // Find closest point
        const closest = data.reduce((prev, curr) => {
            return Math.abs(curr.timestamp - hoveredTimestamp) <
                Math.abs(prev.timestamp - hoveredTimestamp)
                ? curr
                : prev;
        });

        hoveredPoint = closest;
        // Use SVG coordinates for internal SVG elements (line/circle)
        hoverX = ((closest.timestamp - minTime) / timeRange) * SVG_WIDTH;

        const maxLatency = Math.max(...data.map((d) => d.latency), 100);
        // Use percentage (0-100) for tooltip positioning
        hoverY = (1 - closest.latency / maxLatency) * 100;
    }

    function formatTime(timestamp: number) {
        return new Date(timestamp * 1000).toLocaleTimeString([], {
            hour: '2-digit',
            minute: '2-digit',
        });
    }

    function formatDuration(minutes: number) {
        if (minutes < 60) return `${minutes}m`;
        const h = Math.floor(minutes / 60);
        const m = minutes % 60;
        if (m === 0) return `${h}h`;
        return `${h}h ${m}m`;
    }

    function generatePath(
        data: { latency: number; timestamp: number }[],
        width: number,
        height: number
    ) {
        if (data.length < 2) return '';

        const minTime = Math.min(...data.map((d) => d.timestamp));
        const maxTime = Math.max(...data.map((d) => d.timestamp));
        const timeRange = maxTime - minTime || 1;

        const maxLatency = Math.max(...data.map((d) => d.latency), 100);

        return data
            .map((d, i) => {
                const x = ((d.timestamp - minTime) / timeRange) * width;
                const y = height - (d.latency / maxLatency) * height;
                return `${i === 0 ? 'M' : 'L'} ${x} ${y}`;
            })
            .join(' ');
    }
</script>

<div class="admin-page" class:light={theme === 'light'}>
    <div class="container">
        <header class="admin-header">
            <div class="header-main">
                <h1>Admin Dashboard</h1>
            </div>
        </header>

        {#if error}
            <div class="error-container" in:fade>
                {#if error.includes('Zutritt verweigert')}
                    <div class="access-denied-container">
                        <div class="lock-icon">
                            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                                <rect x="3" y="11" width="18" height="11" rx="2" ry="2" />
                                <path d="M7 11V7a5 5 0 0 1 10 0v4" />
                            </svg>
                        </div>
                        <h3>Zutritt verweigert</h3>
                        <p>Du benötigst Administrator-Rechte, um auf diese Seite zuzugreifen.</p>
                        <div class="deny-actions">
                            <button class="action-btn secondary" on:click={() => window.history.back()}>
                                Zurück
                            </button>
                            <button class="action-btn" on:click={() => (window.location.href = '/')}>
                                Zur Startseite
                            </button>
                        </div>
                    </div>
                {:else}
                    <div class="error-icon">⚠️</div>
                    <p>{error}</p>
                    {#if $user?.is_admin}
                        <button on:click={fetchData} class="retry-btn">
                            Erneut versuchen
                        </button>
                    {/if}
                {/if}
            </div>
        {:else if loading || $authLoading}
            <div class="loading-state">
                <div class="spinner"></div>
                <p>Status wird geladen...</p>
            </div>
        {:else if statusData}
            <div class="status-grid">
                <div class="status-card">
                    <div class="status-icon">🌐</div>
                    <div class="status-info">
                        <h3>Website</h3>
                        <div
                            class="status-badge"
                            class:online={statusData.website.status ===
                                'ONLINE'}
                        >
                            {statusData.website.status}
                        </div>
                    </div>
                    <div class="status-metrics">
                        <div class="metric">
                            <span class="label">Uptime</span>
                            <span class="value"
                                >{formatDuration(
                                    statusData.website.uptime_minutes
                                )}</span
                            >
                        </div>
                        <div class="metric">
                            <span class="label">Latenz</span>
                            <span class="value"
                                >{statusData.website.latency}ms</span
                            >
                        </div>
                    </div>
                </div>

                <div class="status-card coc-card">
                    <div class="status-icon">⚔️</div>
                    <div class="status-info">
                        <h3>CoC Upstream API</h3>
                        <div
                            class="status-badge"
                            class:online={statusData.upstream_coc.status ===
                                'ONLINE'}
                        >
                            {statusData.upstream_coc.status}
                        </div>
                    </div>
                    <div class="status-metrics">
                        <div class="metric">
                            <span class="label">Uptime</span>
                            <span class="value"
                                >{formatDuration(
                                    statusData.upstream_coc.uptime_minutes
                                )}</span
                            >
                        </div>
                        <div class="metric">
                            <span class="label">Latenz</span>
                            <span class="value"
                                >{statusData.upstream_coc.latency}ms</span
                            >
                        </div>
                    </div>
                </div>

                <div class="status-card coc-card">
                    <div class="status-icon">🛡️</div>
                    <div class="status-info">
                        <h3>CoC Supercell API</h3>
                        <div
                            class="status-badge"
                            class:online={statusData.supercell_coc.status ===
                                'ONLINE'}
                        >
                            {statusData.supercell_coc.status}
                        </div>
                    </div>
                    <div class="status-metrics">
                        <div class="metric">
                            <span class="label">Uptime</span>
                            <span class="value"
                                >{formatDuration(
                                    statusData.supercell_coc.uptime_minutes
                                )}</span
                            >
                        </div>
                        <div class="metric">
                            <span class="label">Latenz</span>
                            <span class="value"
                                >{statusData.supercell_coc.latency}ms</span
                            >
                        </div>
                    </div>
                </div>

                <div class="status-card cr-card">
                    <div class="status-icon">👑</div>
                    <div class="status-info">
                        <h3>CR Upstream API</h3>
                        <div
                            class="status-badge"
                            class:online={statusData.upstream_cr.status ===
                                'ONLINE'}
                        >
                            {statusData.upstream_cr.status}
                        </div>
                    </div>
                    <div class="status-metrics">
                        <div class="metric">
                            <span class="label">Uptime</span>
                            <span class="value"
                                >{formatDuration(
                                    statusData.upstream_cr.uptime_minutes
                                )}</span
                            >
                        </div>
                        <div class="metric">
                            <span class="label">Latenz</span>
                            <span class="value"
                                >{statusData.upstream_cr.latency}ms</span
                            >
                        </div>
                    </div>
                </div>

                <div class="status-card cr-card">
                    <div class="status-icon">🏆</div>
                    <div class="status-info">
                        <h3>CR Supercell API</h3>
                        <div
                            class="status-badge"
                            class:online={statusData.supercell_cr.status ===
                                'ONLINE'}
                        >
                            {statusData.supercell_cr.status}
                        </div>
                    </div>
                    <div class="status-metrics">
                        <div class="metric">
                            <span class="label">Uptime</span>
                            <span class="value"
                                >{formatDuration(
                                    statusData.supercell_cr.uptime_minutes
                                )}</span
                            >
                        </div>
                        <div class="metric">
                            <span class="label">Latenz</span>
                            <span class="value"
                                >{statusData.supercell_cr.latency}ms</span
                            >
                        </div>
                    </div>
                </div>
            </div>

            <section class="latency-section">
                <h2>Latenz (24h)</h2>

                <!-- CoC Charts -->
                <h3 class="charts-game-title">Clash of Clans</h3>
                <div class="charts-container">
                    <div class="chart-box">
                        <div class="chart-header">
                            <h3>CoC Upstream API</h3>
                            <span class="avg-latency">
                                {upstreamCocData.length > 0
                                    ? Math.round(
                                          upstreamCocData.reduce(
                                              (a, b) => a + b.latency,
                                              0
                                          ) / upstreamCocData.length
                                      )
                                    : 0}ms avg
                            </span>
                        </div>
                        <div
                            class="svg-container"
                            on:mouseleave={() => (hoveredPoint = null)}
                            role="presentation"
                        >
                            {#if upstreamCocData.length > 1}
                                <svg
                                    viewBox="0 0 400 100"
                                    preserveAspectRatio="none"
                                    on:mousemove={(e) =>
                                        handleMouseMove(
                                            e,
                                            upstreamCocData,
                                            'upstream_coc'
                                        )}
                                    role="img"
                                    aria-label="Upstream CoC API Latency Chart"
                                >
                                    <path
                                        d={generatePath(
                                            upstreamCocData,
                                            400,
                                            100
                                        )}
                                        fill="none"
                                        stroke="#3ba55c"
                                        stroke-width="2"
                                    />
                                    {#if hoveredPoint && hoveredPoint.api === 'upstream_coc'}
                                        <line
                                            x1={hoverX}
                                            y1="0"
                                            x2={hoverX}
                                            y2="100"
                                            stroke="rgba(255,255,255,0.2)"
                                            stroke-width="1"
                                        />
                                        <circle
                                            cx={hoverX}
                                            cy={hoverY}
                                            r="4"
                                            fill="#3ba55c"
                                        />
                                    {/if}
                                </svg>

                                {#if hoveredPoint && hoveredPoint.api === 'upstream_coc'}
                                    <div
                                        class="tooltip"
                                        style="left: {(hoverX / SVG_WIDTH) *
                                            100}%; top: {hoverY}%"
                                    >
                                        <span class="time"
                                            >{formatTime(
                                                hoveredPoint.timestamp
                                            )}</span
                                        >
                                        <span class="value"
                                            >{hoveredPoint.latency}ms</span
                                        >
                                    </div>
                                {/if}
                            {:else}
                                <div class="no-data">Keine Daten verfügbar</div>
                            {/if}
                        </div>
                    </div>

                    <div class="chart-box">
                        <div class="chart-header">
                            <h3>CoC Supercell API</h3>
                            <span class="avg-latency sc">
                                {supercellCocData.length > 0
                                    ? Math.round(
                                          supercellCocData.reduce(
                                              (a, b) => a + b.latency,
                                              0
                                          ) / supercellCocData.length
                                      )
                                    : 0}ms avg
                            </span>
                        </div>
                        <div
                            class="svg-container"
                            on:mouseleave={() => (hoveredPoint = null)}
                            role="presentation"
                        >
                            {#if supercellCocData.length > 1}
                                <svg
                                    viewBox="0 0 400 100"
                                    preserveAspectRatio="none"
                                    on:mousemove={(e) =>
                                        handleMouseMove(
                                            e,
                                            supercellCocData,
                                            'supercell_coc'
                                        )}
                                    role="img"
                                    aria-label="Supercell CoC API Latency Chart"
                                >
                                    <path
                                        d={generatePath(
                                            supercellCocData,
                                            400,
                                            100
                                        )}
                                        fill="none"
                                        stroke="#22c55e"
                                        stroke-width="2"
                                    />
                                    {#if hoveredPoint && hoveredPoint.api === 'supercell_coc'}
                                        <line
                                            x1={hoverX}
                                            y1="0"
                                            x2={hoverX}
                                            y2="100"
                                            stroke="rgba(255,255,255,0.2)"
                                            stroke-width="1"
                                        />
                                        <circle
                                            cx={hoverX}
                                            cy={hoverY}
                                            r="4"
                                            fill="#22c55e"
                                        />
                                    {/if}
                                </svg>

                                {#if hoveredPoint && hoveredPoint.api === 'supercell_coc'}
                                    <div
                                        class="tooltip"
                                        style="left: {(hoverX / SVG_WIDTH) *
                                            100}%; top: {hoverY}%"
                                    >
                                        <span class="time"
                                            >{formatTime(
                                                hoveredPoint.timestamp
                                            )}</span
                                        >
                                        <span class="value"
                                            >{hoveredPoint.latency}ms</span
                                        >
                                    </div>
                                {/if}
                            {:else}
                                <div class="no-data">Keine Daten verfügbar</div>
                            {/if}
                        </div>
                    </div>
                </div>

                <!-- CR Charts -->
                <h3 class="charts-game-title">Clash Royale</h3>
                <div class="charts-container">
                    <div class="chart-box">
                        <div class="chart-header">
                            <h3>CR Upstream API</h3>
                            <span class="avg-latency cr">
                                {upstreamCrData.length > 0
                                    ? Math.round(
                                          upstreamCrData.reduce(
                                              (a, b) => a + b.latency,
                                              0
                                          ) / upstreamCrData.length
                                      )
                                    : 0}ms avg
                            </span>
                        </div>
                        <div
                            class="svg-container"
                            on:mouseleave={() => (hoveredPoint = null)}
                            role="presentation"
                        >
                            {#if upstreamCrData.length > 1}
                                <svg
                                    viewBox="0 0 400 100"
                                    preserveAspectRatio="none"
                                    on:mousemove={(e) =>
                                        handleMouseMove(
                                            e,
                                            upstreamCrData,
                                            'upstream_cr'
                                        )}
                                    role="img"
                                    aria-label="Upstream CR API Latency Chart"
                                >
                                    <path
                                        d={generatePath(
                                            upstreamCrData,
                                            400,
                                            100
                                        )}
                                        fill="none"
                                        stroke="#5865f2"
                                        stroke-width="2"
                                    />
                                    {#if hoveredPoint && hoveredPoint.api === 'upstream_cr'}
                                        <line
                                            x1={hoverX}
                                            y1="0"
                                            x2={hoverX}
                                            y2="100"
                                            stroke="rgba(255,255,255,0.2)"
                                            stroke-width="1"
                                        />
                                        <circle
                                            cx={hoverX}
                                            cy={hoverY}
                                            r="4"
                                            fill="#5865f2"
                                        />
                                    {/if}
                                </svg>

                                {#if hoveredPoint && hoveredPoint.api === 'upstream_cr'}
                                    <div
                                        class="tooltip"
                                        style="left: {(hoverX / SVG_WIDTH) *
                                            100}%; top: {hoverY}%"
                                    >
                                        <span class="time"
                                            >{formatTime(
                                                hoveredPoint.timestamp
                                            )}</span
                                        >
                                        <span class="value"
                                            >{hoveredPoint.latency}ms</span
                                        >
                                    </div>
                                {/if}
                            {:else}
                                <div class="no-data">Keine Daten verfügbar</div>
                            {/if}
                        </div>
                    </div>

                    <div class="chart-box">
                        <div class="chart-header">
                            <h3>CR Supercell API</h3>
                            <span class="avg-latency cr">
                                {supercellCrData.length > 0
                                    ? Math.round(
                                          supercellCrData.reduce(
                                              (a, b) => a + b.latency,
                                              0
                                          ) / supercellCrData.length
                                      )
                                    : 0}ms avg
                            </span>
                        </div>
                        <div
                            class="svg-container"
                            on:mouseleave={() => (hoveredPoint = null)}
                            role="presentation"
                        >
                            {#if supercellCrData.length > 1}
                                <svg
                                    viewBox="0 0 400 100"
                                    preserveAspectRatio="none"
                                    on:mousemove={(e) =>
                                        handleMouseMove(
                                            e,
                                            supercellCrData,
                                            'supercell_cr'
                                        )}
                                    role="img"
                                    aria-label="Supercell CR API Latency Chart"
                                >
                                    <path
                                        d={generatePath(
                                            supercellCrData,
                                            400,
                                            100
                                        )}
                                        fill="none"
                                        stroke="#7289da"
                                        stroke-width="2"
                                    />
                                    {#if hoveredPoint && hoveredPoint.api === 'supercell_cr'}
                                        <line
                                            x1={hoverX}
                                            y1="0"
                                            x2={hoverX}
                                            y2="100"
                                            stroke="rgba(255,255,255,0.2)"
                                            stroke-width="1"
                                        />
                                        <circle
                                            cx={hoverX}
                                            cy={hoverY}
                                            r="4"
                                            fill="#7289da"
                                        />
                                    {/if}
                                </svg>

                                {#if hoveredPoint && hoveredPoint.api === 'supercell_cr'}
                                    <div
                                        class="tooltip"
                                        style="left: {(hoverX / SVG_WIDTH) *
                                            100}%; top: {hoverY}%"
                                    >
                                        <span class="time"
                                            >{formatTime(
                                                hoveredPoint.timestamp
                                            )}</span
                                        >
                                        <span class="value"
                                            >{hoveredPoint.latency}ms</span
                                        >
                                    </div>
                                {/if}
                            {:else}
                                <div class="no-data">Keine Daten verfügbar</div>
                            {/if}
                        </div>
                    </div>
                </div>

                <!-- Website Chart -->
                <h3 class="charts-game-title">Website</h3>
                <div class="charts-container single">
                    <div class="chart-box">
                        <div class="chart-header">
                            <h3>Website</h3>
                            <span class="avg-latency web">
                                {websiteData.length > 0
                                    ? Math.round(
                                          websiteData.reduce(
                                              (a, b) => a + b.latency,
                                              0
                                          ) / websiteData.length
                                      )
                                    : 0}ms avg
                            </span>
                        </div>
                        <div
                            class="svg-container"
                            on:mouseleave={() => (hoveredPoint = null)}
                            role="presentation"
                        >
                            {#if websiteData.length > 1}
                                <svg
                                    viewBox="0 0 400 100"
                                    preserveAspectRatio="none"
                                    on:mousemove={(e) =>
                                        handleMouseMove(
                                            e,
                                            websiteData,
                                            'website'
                                        )}
                                    role="img"
                                    aria-label="Website Latency Chart"
                                >
                                    <path
                                        d={generatePath(websiteData, 400, 100)}
                                        fill="none"
                                        stroke="#f59e0b"
                                        stroke-width="2"
                                    />
                                    {#if hoveredPoint && hoveredPoint.api === 'website'}
                                        <line
                                            x1={hoverX}
                                            y1="0"
                                            x2={hoverX}
                                            y2="100"
                                            stroke="rgba(255,255,255,0.2)"
                                            stroke-width="1"
                                        />
                                        <circle
                                            cx={hoverX}
                                            cy={hoverY}
                                            r="4"
                                            fill="#f59e0b"
                                        />
                                    {/if}
                                </svg>

                                {#if hoveredPoint && hoveredPoint.api === 'website'}
                                    <div
                                        class="tooltip"
                                        style="left: {(hoverX / SVG_WIDTH) *
                                            100}%; top: {hoverY}%"
                                    >
                                        <span class="time"
                                            >{formatTime(
                                                hoveredPoint.timestamp
                                            )}</span
                                        >
                                        <span class="value"
                                            >{hoveredPoint.latency}ms</span
                                        >
                                    </div>
                                {/if}
                            {:else}
                                <div class="no-data">Keine Daten verfügbar</div>
                            {/if}
                        </div>
                    </div>
                </div>
            </section>
        {/if}
    </div>
</div>

<style>
    .admin-page {
        min-height: 100vh;
        padding: 4rem 2rem;
        background: radial-gradient(
                circle at top right,
                rgba(88, 101, 242, 0.05),
                transparent 400px
            ),
            radial-gradient(
                circle at bottom left,
                rgba(59, 130, 246, 0.05),
                transparent 400px
            );
    }

    .container {
        max-width: 1000px;
        margin: 0 auto;
    }

    .no-data {
        display: flex;
        align-items: center;
        justify-content: center;
        height: 100px;
        color: rgba(255, 255, 255, 0.3);
        font-size: 0.875rem;
    }

    .admin-header {
        margin-bottom: 3rem;
    }

    .admin-header h1 {
        font-size: 2.5rem;
        margin: 0;
        background: linear-gradient(135deg, #fff 0%, #a5b4fc 100%);
        -webkit-background-clip: text;
        background-clip: text;
        -webkit-text-fill-color: transparent;
    }

    .light .admin-header h1 {
        background: linear-gradient(135deg, #1a1a2e 0%, #4f46e5 100%);
        -webkit-background-clip: text;
        background-clip: text;
        -webkit-text-fill-color: transparent;
    }

    .status-grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
        gap: 1.5rem;
    }

    .status-card {
        background: rgba(255, 255, 255, 0.03);
        border: 1px solid rgba(255, 255, 255, 0.08);
        border-radius: 20px;
        padding: 1.5rem;
        display: flex;
        align-items: center;
        gap: 1.5rem;
        transition:
            transform 0.3s ease,
            background 0.3s ease;
    }

    .light .status-card {
        background: white;
        border-color: rgba(0, 0, 0, 0.05);
        box-shadow: 0 4px 15px rgba(0, 0, 0, 0.03);
    }

    .status-card:hover {
        transform: translateY(-5px);
        background: rgba(255, 255, 255, 0.05);
    }

    .status-icon {
        font-size: 2rem;
        width: 60px;
        height: 60px;
        background: rgba(255, 255, 255, 0.03);
        border-radius: 15px;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .light .status-icon {
        background: #f8fafc;
    }

    .status-info h3 {
        margin: 0 0 0.5rem 0;
        font-size: 1.1rem;
        color: rgba(255, 255, 255, 0.9);
    }

    .status-metrics {
        margin-left: auto;
        display: flex;
        flex-direction: column;
        gap: 0.5rem;
        text-align: right;
    }

    .metric {
        display: flex;
        flex-direction: column;
    }

    .metric .label {
        font-size: 0.7rem;
        text-transform: uppercase;
        color: rgba(255, 255, 255, 0.4);
        font-weight: 700;
        letter-spacing: 0.05em;
    }

    .light .metric .label {
        color: rgba(0, 0, 0, 0.4);
    }

    .metric .value {
        font-size: 0.95rem;
        font-weight: 700;
        color: rgba(255, 255, 255, 0.9);
    }

    .light .metric .value {
        color: #1a1a2e;
    }

    .light .status-info h3 {
        color: #1a1a2e;
    }

    .status-badge {
        display: inline-block;
        padding: 0.25rem 0.75rem;
        border-radius: 100px;
        font-size: 0.8rem;
        font-weight: 700;
        background: rgba(239, 68, 68, 0.1);
        color: #ef4444;
        border: 1px solid rgba(239, 68, 68, 0.2);
    }

    .status-badge.online {
        background: rgba(34, 197, 94, 0.1);
        color: #22c55e;
        border: 1px solid rgba(34, 197, 94, 0.2);
    }

    .latency-section {
        margin-top: 4rem;
    }

    .latency-section h2 {
        font-size: 1.5rem;
        margin-bottom: 1.5rem;
        color: rgba(255, 255, 255, 0.9);
    }

    .light .latency-section h2 {
        color: #1a1a2e;
    }

    .charts-container {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
        gap: 1.5rem;
    }

    .chart-box {
        background: rgba(255, 255, 255, 0.02);
        border: 1px solid rgba(255, 255, 255, 0.05);
        border-radius: 20px;
        padding: 1.5rem;
    }

    .light .chart-box {
        background: white;
        border-color: rgba(0, 0, 0, 0.05);
        box-shadow: 0 4px 15px rgba(0, 0, 0, 0.03);
    }

    .chart-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 1rem;
    }

    .chart-header h3 {
        margin: 0;
        font-size: 1rem;
        color: rgba(255, 255, 255, 0.7);
    }

    .light .chart-header h3 {
        color: rgba(0, 0, 0, 0.6);
    }

    .avg-latency {
        font-size: 0.9rem;
        font-weight: 600;
        color: #3ba55c;
    }

    .avg-latency.sc {
        color: #22c55e;
    }

    .avg-latency.cr {
        color: #5865f2;
    }

    .avg-latency.web {
        color: #f59e0b;
    }

    .charts-game-title {
        font-size: 1.1rem;
        font-weight: 600;
        color: rgba(255, 255, 255, 0.8);
        margin: 1.5rem 0 1rem 0;
        padding-bottom: 0.5rem;
        border-bottom: 1px solid rgba(255, 255, 255, 0.1);
    }

    .charts-game-title:first-of-type {
        margin-top: 0;
    }

    .light .charts-game-title {
        color: rgba(0, 0, 0, 0.7);
        border-color: rgba(0, 0, 0, 0.1);
    }

    .charts-container.single {
        max-width: 600px;
    }

    .svg-container {
        width: 100%;
        height: 100px;
        background: rgba(0, 0, 0, 0.1);
        border-radius: 10px;
        position: relative;
    }

    .light .svg-container {
        background: #f8fafc;
    }

    .svg-container svg {
        width: 100%;
        height: 100%;
    }

    .error-container {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        padding: 6rem 2rem;
        color: #b9bbbe;
        text-align: center;
        gap: 1.5rem;
    }

    .access-denied-container {
        max-width: 400px;
        display: flex;
        flex-direction: column;
        align-items: center;
        gap: 1.5rem;
        padding: 3rem;
        background: rgba(255, 255, 255, 0.03);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 24px;
        box-shadow: 0 20px 50px rgba(0, 0, 0, 0.3);
    }

    .light .access-denied-container {
        background: white;
        border-color: #e2e8f0;
        box-shadow: 0 10px 30px rgba(0, 0, 0, 0.05);
    }

    .lock-icon {
        width: 80px;
        height: 80px;
        background: rgba(239, 68, 68, 0.1);
        color: #ef4444;
        display: flex;
        align-items: center;
        justify-content: center;
        border-radius: 20px;
        margin-bottom: 0.5rem;
    }

    .lock-icon svg {
        width: 40px;
        height: 40px;
    }

    .access-denied-container h3 {
        font-size: 1.75rem;
        font-weight: 800;
        margin: 0;
        color: white;
    }

    .light .access-denied-container h3 {
        color: #1e293b;
    }

    .access-denied-container p {
        color: rgba(255, 255, 255, 0.6);
        font-size: 1.1rem;
        line-height: 1.5;
        margin: 0;
    }

    .light .access-denied-container p {
        color: #64748b;
    }

    .deny-actions {
        display: flex;
        gap: 1rem;
        margin-top: 1rem;
    }

    .action-btn {
        padding: 0.8rem 1.75rem;
        border-radius: 12px;
        font-weight: 700;
        cursor: pointer;
        transition: all 0.2s ease;
        border: none;
        background: #5865f2;
        color: white;
    }

    .action-btn:hover {
        background: #4752c4;
        transform: translateY(-2px);
    }

    .action-btn.secondary {
        background: rgba(255, 255, 255, 0.05);
        border: 1px solid rgba(255, 255, 255, 0.1);
        color: white;
    }

    .light .action-btn.secondary {
        background: #f1f5f9;
        border-color: #e2e8f0;
        color: #475569;
    }

    .action-btn.secondary:hover {
        background: rgba(255, 255, 255, 0.1);
    }

    .light .action-btn.secondary:hover {
        background: #e2e8f0;
    }

    .error-icon {
        font-size: 3rem;
        margin-bottom: 1rem;
    }

    .retry-btn {
        margin-top: 1.5rem;
        padding: 0.75rem 1.5rem;
        background: rgba(255, 255, 255, 0.1);
        border: 1px solid rgba(255, 255, 255, 0.1);
        border-radius: 12px;
        color: white;
        cursor: pointer;
        font-weight: 600;
        transition: all 0.2s;
    }

    .retry-btn:hover {
        background: rgba(255, 255, 255, 0.15);
    }

    .loading-state {
        text-align: center;
        padding: 4rem;
    }

    .spinner {
        width: 40px;
        height: 40px;
        border: 3px solid rgba(88, 101, 242, 0.1);
        border-top-color: #5865f2;
        border-radius: 50%;
        animation: spin 1s linear infinite;
        margin: 0 auto 1.5rem;
    }

    @keyframes spin {
        to {
            transform: rotate(360deg);
        }
    }

    .no-data {
        display: flex;
        align-items: center;
        justify-content: center;
        height: 100px;
        color: rgba(255, 255, 255, 0.3);
        font-size: 0.875rem;
    }

    .tooltip {
        position: absolute;
        top: 0;
        transform: translate(-50%, -120%);
        background: #1e1e2e;
        border: 1px solid rgba(255, 255, 255, 0.2);
        padding: 0.5rem 0.8rem;
        border-radius: 8px;
        display: flex;
        flex-direction: column;
        gap: 2px;
        pointer-events: none;
        z-index: 100;
        box-shadow: 0 4px 15px rgba(0, 0, 0, 0.6);
        white-space: nowrap;
    }

    .tooltip .time {
        font-size: 0.7rem;
        color: rgba(255, 255, 255, 0.5);
    }

    .tooltip .value {
        font-size: 0.9rem;
        font-weight: 600;
        color: white;
    }

    svg {
        cursor: crosshair;
    }
</style>
