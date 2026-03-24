<script lang="ts">
	import { onMount } from 'svelte';
	import { fetchEvents, type TelemetryEvent } from '$lib/api';

	let events: TelemetryEvent[] = $state([]);
	let loading = $state(false);
	let error = $state('');

	// Filters
	let filterType = $state('');
	let filterSource = $state('');
	let filterSeverity = $state('');

	// Pagination
	let limit = 25;
	let offset = $state(0);
	let hasMore = $state(true);

	// Expanded row
	let expandedId: string | null = $state(null);

	async function loadEvents() {
		loading = true;
		error = '';
		try {
			const result = await fetchEvents({
				event_type: filterType || undefined,
				source: filterSource || undefined,
				severity: filterSeverity || undefined,
				limit,
				offset
			});
			events = result;
			hasMore = result.length === limit;
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load events';
		}
		loading = false;
	}

	onMount(() => {
		loadEvents();
	});

	function applyFilters() {
		offset = 0;
		loadEvents();
	}

	function nextPage() {
		offset += limit;
		loadEvents();
	}

	function prevPage() {
		offset = Math.max(0, offset - limit);
		loadEvents();
	}

	function toggleExpand(id: string) {
		expandedId = expandedId === id ? null : id;
	}

	function severityColor(sev: string): string {
		const map: Record<string, string> = {
			info: 'var(--info)',
			warn: 'var(--warn)',
			error: 'var(--error)',
			critical: 'var(--critical)'
		};
		return map[sev] || 'var(--text-secondary)';
	}

	function formatTime(ts: string): string {
		return new Date(ts).toLocaleString();
	}
</script>

<h1>Events</h1>

<div class="filters">
	<input type="text" placeholder="Event type" bind:value={filterType} />
	<input type="text" placeholder="Source" bind:value={filterSource} />
	<select bind:value={filterSeverity}>
		<option value="">All severities</option>
		<option value="info">Info</option>
		<option value="warn">Warn</option>
		<option value="error">Error</option>
		<option value="critical">Critical</option>
	</select>
	<button onclick={applyFilters}>Filter</button>
</div>

{#if error}
	<p class="error">{error}</p>
{/if}

{#if loading}
	<p class="muted">Loading...</p>
{:else}
	<table>
		<thead>
			<tr>
				<th>Severity</th>
				<th>Type</th>
				<th>Source</th>
				<th>Timestamp</th>
			</tr>
		</thead>
		<tbody>
			{#each events as event (event.id)}
				<tr class="clickable" onclick={() => toggleExpand(event.id)}>
					<td><span class="badge" style="background: {severityColor(event.severity)}">{event.severity}</span></td>
					<td>{event.event_type}</td>
					<td>{event.source}</td>
					<td>{formatTime(event.timestamp)}</td>
				</tr>
				{#if expandedId === event.id}
					<tr class="expanded">
						<td colspan="4">
							<div class="detail">
								<div><strong>ID:</strong> {event.id}</div>
								<div><strong>Payload:</strong></div>
								<pre>{JSON.stringify(event.payload, null, 2)}</pre>
							</div>
						</td>
					</tr>
				{/if}
			{/each}
		</tbody>
	</table>

	<div class="pagination">
		<button onclick={prevPage} disabled={offset === 0}>Previous</button>
		<span class="muted">Showing {offset + 1} - {offset + events.length}</span>
		<button onclick={nextPage} disabled={!hasMore}>Next</button>
	</div>
{/if}

<style>
	h1 {
		margin-bottom: 1.5rem;
		font-size: 1.5rem;
	}

	.error {
		color: var(--error);
		margin-bottom: 1rem;
	}

	.muted {
		color: var(--text-secondary);
		font-size: 0.85rem;
	}

	.filters {
		display: flex;
		gap: 0.75rem;
		margin-bottom: 1.5rem;
		flex-wrap: wrap;
	}

	.filters input,
	.filters select {
		background: var(--bg-card);
		border: 1px solid var(--border);
		color: var(--text-primary);
		padding: 0.5rem 0.75rem;
		border-radius: 6px;
		font-size: 0.85rem;
	}

	.filters button {
		background: var(--accent);
		color: white;
		border: none;
		padding: 0.5rem 1.25rem;
		border-radius: 6px;
		cursor: pointer;
		font-size: 0.85rem;
	}

	.filters button:hover {
		opacity: 0.9;
	}

	table {
		width: 100%;
		border-collapse: collapse;
		background: var(--bg-card);
		border-radius: 8px;
		overflow: hidden;
	}

	th {
		text-align: left;
		padding: 0.75rem 1rem;
		background: var(--bg-secondary);
		color: var(--text-secondary);
		font-size: 0.8rem;
		text-transform: uppercase;
		letter-spacing: 0.05em;
	}

	td {
		padding: 0.625rem 1rem;
		border-top: 1px solid var(--border);
		font-size: 0.85rem;
	}

	.clickable {
		cursor: pointer;
	}

	.clickable:hover {
		background: var(--bg-hover);
	}

	.badge {
		padding: 0.15rem 0.5rem;
		border-radius: 4px;
		font-size: 0.7rem;
		color: white;
		font-weight: 600;
		text-transform: uppercase;
	}

	.expanded td {
		background: var(--bg-secondary);
	}

	.detail {
		padding: 0.5rem;
		font-size: 0.85rem;
	}

	.detail pre {
		background: var(--bg-primary);
		padding: 0.75rem;
		border-radius: 6px;
		margin-top: 0.5rem;
		overflow-x: auto;
		font-size: 0.8rem;
		color: var(--text-secondary);
	}

	.pagination {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 1rem;
		margin-top: 1.5rem;
	}

	.pagination button {
		background: var(--bg-card);
		color: var(--text-primary);
		border: 1px solid var(--border);
		padding: 0.5rem 1rem;
		border-radius: 6px;
		cursor: pointer;
		font-size: 0.85rem;
	}

	.pagination button:disabled {
		opacity: 0.4;
		cursor: not-allowed;
	}

	.pagination button:hover:not(:disabled) {
		background: var(--bg-hover);
	}
</style>
