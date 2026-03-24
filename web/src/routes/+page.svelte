<script lang="ts">
	import { onMount, onDestroy } from 'svelte';
	import { fetchStats, fetchEvents, connectWebSocket, type TelemetryEvent, type Stats } from '$lib/api';

	let stats: Stats | null = $state(null);
	let recentEvents: TelemetryEvent[] = $state([]);
	let liveEvents: TelemetryEvent[] = $state([]);
	let ws: WebSocket | null = null;
	let error: string = $state('');

	onMount(async () => {
		try {
			stats = await fetchStats();
			recentEvents = await fetchEvents({ limit: 20 });
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load data';
		}

		ws = connectWebSocket((event) => {
			liveEvents = [event, ...liveEvents].slice(0, 50);
		});
	});

	onDestroy(() => {
		if (ws) ws.close();
	});

	function severityColor(sev: string): string {
		const map: Record<string, string> = {
			info: 'var(--info)',
			warn: 'var(--warn)',
			error: 'var(--error)',
			critical: 'var(--critical)'
		};
		return map[sev] || 'var(--text-secondary)';
	}

	function severityCount(sev: string): number {
		if (!stats) return 0;
		const row = stats.by_severity.find((r) => r.label === sev);
		return row?.count || 0;
	}

	function formatTime(ts: string): string {
		return new Date(ts).toLocaleString();
	}
</script>

<h1>Dashboard</h1>

{#if error}
	<p class="error">{error}</p>
{/if}

<div class="stats-grid">
	<div class="stat-card">
		<div class="stat-value">{stats?.total ?? '-'}</div>
		<div class="stat-label">Total Events</div>
	</div>
	<div class="stat-card">
		<div class="stat-value">{stats?.last_hour ?? '-'}</div>
		<div class="stat-label">Last Hour</div>
	</div>
	<div class="stat-card">
		<div class="stat-value">{stats?.last_day ?? '-'}</div>
		<div class="stat-label">Last 24h</div>
	</div>
	{#each ['info', 'warn', 'error', 'critical'] as sev}
		<div class="stat-card">
			<div class="stat-value" style="color: {severityColor(sev)}">{severityCount(sev)}</div>
			<div class="stat-label">{sev.charAt(0).toUpperCase() + sev.slice(1)}</div>
		</div>
	{/each}
</div>

<div class="panels">
	<section class="panel">
		<h2>Live Feed</h2>
		{#if liveEvents.length === 0}
			<p class="muted">Waiting for live events...</p>
		{:else}
			<div class="event-list">
				{#each liveEvents as event (event.id)}
					<div class="event-row">
						<span class="badge" style="background: {severityColor(event.severity)}">{event.severity}</span>
						<span class="event-type">{event.event_type}</span>
						<span class="event-source">{event.source}</span>
						<span class="event-time">{formatTime(event.timestamp)}</span>
					</div>
				{/each}
			</div>
		{/if}
	</section>

	<section class="panel">
		<h2>Recent Events</h2>
		{#if recentEvents.length === 0}
			<p class="muted">No events yet.</p>
		{:else}
			<div class="event-list">
				{#each recentEvents as event (event.id)}
					<div class="event-row">
						<span class="badge" style="background: {severityColor(event.severity)}">{event.severity}</span>
						<span class="event-type">{event.event_type}</span>
						<span class="event-source">{event.source}</span>
						<span class="event-time">{formatTime(event.timestamp)}</span>
					</div>
				{/each}
			</div>
		{/if}
	</section>
</div>

<style>
	h1 {
		margin-bottom: 1.5rem;
		font-size: 1.5rem;
	}

	.error {
		color: var(--error);
		margin-bottom: 1rem;
	}

	.stats-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
		gap: 1rem;
		margin-bottom: 2rem;
	}

	.stat-card {
		background: var(--bg-card);
		border: 1px solid var(--border);
		border-radius: 8px;
		padding: 1.25rem;
		text-align: center;
	}

	.stat-value {
		font-size: 1.75rem;
		font-weight: 700;
	}

	.stat-label {
		color: var(--text-secondary);
		font-size: 0.8rem;
		margin-top: 0.25rem;
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	.panels {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 1.5rem;
	}

	.panel {
		background: var(--bg-card);
		border: 1px solid var(--border);
		border-radius: 8px;
		padding: 1.25rem;
	}

	.panel h2 {
		font-size: 1rem;
		margin-bottom: 1rem;
		color: var(--text-secondary);
	}

	.muted {
		color: var(--text-secondary);
		font-size: 0.85rem;
	}

	.event-list {
		display: flex;
		flex-direction: column;
		gap: 0.5rem;
		max-height: 400px;
		overflow-y: auto;
	}

	.event-row {
		display: flex;
		align-items: center;
		gap: 0.75rem;
		padding: 0.5rem;
		border-radius: 4px;
		font-size: 0.85rem;
		background: var(--bg-secondary);
	}

	.badge {
		padding: 0.15rem 0.5rem;
		border-radius: 4px;
		font-size: 0.7rem;
		color: white;
		font-weight: 600;
		text-transform: uppercase;
		flex-shrink: 0;
	}

	.event-type {
		font-weight: 500;
	}

	.event-source {
		color: var(--text-secondary);
	}

	.event-time {
		margin-left: auto;
		color: var(--text-secondary);
		font-size: 0.75rem;
		flex-shrink: 0;
	}
</style>
