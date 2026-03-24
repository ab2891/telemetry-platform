<script lang="ts">
	let { children } = $props();
</script>

<svelte:head>
	<title>Telemetry Platform</title>
</svelte:head>

<div class="app">
	<aside class="sidebar">
		<div class="logo">
			<h2>Telemetry</h2>
		</div>
		<nav>
			<a href="/">Dashboard</a>
			<a href="/events">Events</a>
			<a href="/sources">Sources</a>
		</nav>
	</aside>
	<main class="content">
		{@render children()}
	</main>
</div>

<style>
	:global(*) {
		margin: 0;
		padding: 0;
		box-sizing: border-box;
	}

	:global(body) {
		font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
		background: var(--bg-primary);
		color: var(--text-primary);
	}

	:global(:root) {
		--bg-primary: #0f1117;
		--bg-secondary: #1a1d27;
		--bg-card: #21242f;
		--bg-hover: #2a2d3a;
		--border: #2e3140;
		--text-primary: #e1e4eb;
		--text-secondary: #8b8fa3;
		--accent: #6c5ce7;
		--info: #3498db;
		--warn: #f39c12;
		--error: #e74c3c;
		--critical: #e91e63;
		--success: #2ecc71;
	}

	.app {
		display: flex;
		min-height: 100vh;
	}

	.sidebar {
		width: 220px;
		background: var(--bg-secondary);
		border-right: 1px solid var(--border);
		padding: 1.5rem 0;
		flex-shrink: 0;
	}

	.logo {
		padding: 0 1.5rem 1.5rem;
		border-bottom: 1px solid var(--border);
		margin-bottom: 1rem;
	}

	.logo h2 {
		color: var(--accent);
		font-size: 1.25rem;
	}

	nav {
		display: flex;
		flex-direction: column;
		gap: 0.25rem;
		padding: 0 0.75rem;
	}

	nav a {
		color: var(--text-secondary);
		text-decoration: none;
		padding: 0.625rem 0.75rem;
		border-radius: 6px;
		font-size: 0.9rem;
		transition: all 0.15s;
	}

	nav a:hover {
		background: var(--bg-hover);
		color: var(--text-primary);
	}

	.content {
		flex: 1;
		padding: 2rem;
		overflow-y: auto;
	}
</style>
