<script lang="ts">
	import { onMount } from 'svelte';
	import { fetchSources, createSource, type Source } from '$lib/api';

	let sources: Source[] = $state([]);
	let loading = $state(false);
	let error = $state('');
	let success = $state('');

	// Form
	let newName = $state('');
	let newDesc = $state('');

	async function loadSources() {
		loading = true;
		error = '';
		try {
			sources = await fetchSources();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to load sources';
		}
		loading = false;
	}

	onMount(() => {
		loadSources();
	});

	async function handleSubmit() {
		if (!newName.trim()) return;
		error = '';
		success = '';
		try {
			await createSource({
				name: newName.trim(),
				description: newDesc.trim() || undefined
			});
			newName = '';
			newDesc = '';
			success = 'Source created successfully';
			await loadSources();
		} catch (e) {
			error = e instanceof Error ? e.message : 'Failed to create source';
		}
	}

	function formatTime(ts: string): string {
		return new Date(ts).toLocaleString();
	}
</script>

<h1>Sources</h1>

<section class="form-card">
	<h2>Register New Source</h2>
	<form onsubmit={(e) => { e.preventDefault(); handleSubmit(); }}>
		<div class="form-row">
			<input type="text" placeholder="Source name" bind:value={newName} required />
			<input type="text" placeholder="Description (optional)" bind:value={newDesc} />
			<button type="submit">Add Source</button>
		</div>
	</form>
	{#if success}
		<p class="success">{success}</p>
	{/if}
	{#if error}
		<p class="error">{error}</p>
	{/if}
</section>

{#if loading}
	<p class="muted">Loading...</p>
{:else}
	<section class="source-list">
		{#if sources.length === 0}
			<p class="muted">No sources registered yet.</p>
		{:else}
			{#each sources as source (source.id)}
				<div class="source-card">
					<div class="source-name">{source.name}</div>
					{#if source.description}
						<div class="source-desc">{source.description}</div>
					{/if}
					<div class="source-meta">
						<span>ID: {source.id}</span>
						<span>Created: {formatTime(source.created_at)}</span>
					</div>
				</div>
			{/each}
		{/if}
	</section>
{/if}

<style>
	h1 {
		margin-bottom: 1.5rem;
		font-size: 1.5rem;
	}

	h2 {
		font-size: 1rem;
		margin-bottom: 1rem;
		color: var(--text-secondary);
	}

	.error {
		color: var(--error);
		margin-top: 0.75rem;
		font-size: 0.85rem;
	}

	.success {
		color: var(--success);
		margin-top: 0.75rem;
		font-size: 0.85rem;
	}

	.muted {
		color: var(--text-secondary);
		font-size: 0.85rem;
	}

	.form-card {
		background: var(--bg-card);
		border: 1px solid var(--border);
		border-radius: 8px;
		padding: 1.25rem;
		margin-bottom: 2rem;
	}

	.form-row {
		display: flex;
		gap: 0.75rem;
		flex-wrap: wrap;
	}

	.form-row input {
		background: var(--bg-secondary);
		border: 1px solid var(--border);
		color: var(--text-primary);
		padding: 0.5rem 0.75rem;
		border-radius: 6px;
		font-size: 0.85rem;
		flex: 1;
		min-width: 150px;
	}

	.form-row button {
		background: var(--accent);
		color: white;
		border: none;
		padding: 0.5rem 1.25rem;
		border-radius: 6px;
		cursor: pointer;
		font-size: 0.85rem;
		flex-shrink: 0;
	}

	.form-row button:hover {
		opacity: 0.9;
	}

	.source-list {
		display: flex;
		flex-direction: column;
		gap: 0.75rem;
	}

	.source-card {
		background: var(--bg-card);
		border: 1px solid var(--border);
		border-radius: 8px;
		padding: 1rem 1.25rem;
	}

	.source-name {
		font-weight: 600;
		font-size: 1rem;
		margin-bottom: 0.25rem;
	}

	.source-desc {
		color: var(--text-secondary);
		font-size: 0.85rem;
		margin-bottom: 0.5rem;
	}

	.source-meta {
		display: flex;
		gap: 1.5rem;
		font-size: 0.75rem;
		color: var(--text-secondary);
	}
</style>
