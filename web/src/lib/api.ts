const API_BASE = 'http://localhost:8081';
const WS_URL = 'ws://localhost:8082/ws';

export interface TelemetryEvent {
	id: string;
	event_type: string;
	source: string;
	payload: Record<string, unknown>;
	severity: 'info' | 'warn' | 'error' | 'critical';
	timestamp: string;
	created_at: string;
}

export interface Source {
	id: string;
	name: string;
	description: string | null;
	created_at: string;
}

export interface CountRow {
	label: string;
	count: number;
}

export interface Stats {
	total: number;
	by_severity: CountRow[];
	by_event_type: CountRow[];
	last_hour: number;
	last_day: number;
}

export async function fetchEvents(params?: {
	event_type?: string;
	source?: string;
	severity?: string;
	limit?: number;
	offset?: number;
}): Promise<TelemetryEvent[]> {
	const url = new URL(`${API_BASE}/api/events`);
	if (params) {
		Object.entries(params).forEach(([k, v]) => {
			if (v !== undefined && v !== '') url.searchParams.set(k, String(v));
		});
	}
	const res = await fetch(url.toString());
	if (!res.ok) throw new Error(`Failed to fetch events: ${res.status}`);
	return res.json();
}

export async function fetchEvent(id: string): Promise<TelemetryEvent> {
	const res = await fetch(`${API_BASE}/api/events/${id}`);
	if (!res.ok) throw new Error(`Failed to fetch event: ${res.status}`);
	return res.json();
}

export async function createEvent(event: {
	event_type: string;
	source: string;
	payload: Record<string, unknown>;
	severity: string;
}): Promise<TelemetryEvent> {
	const res = await fetch(`${API_BASE}/api/events`, {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify(event)
	});
	if (!res.ok) throw new Error(`Failed to create event: ${res.status}`);
	return res.json();
}

export async function fetchSources(): Promise<Source[]> {
	const res = await fetch(`${API_BASE}/api/sources`);
	if (!res.ok) throw new Error(`Failed to fetch sources: ${res.status}`);
	return res.json();
}

export async function createSource(source: {
	name: string;
	description?: string;
}): Promise<Source> {
	const res = await fetch(`${API_BASE}/api/sources`, {
		method: 'POST',
		headers: { 'Content-Type': 'application/json' },
		body: JSON.stringify(source)
	});
	if (!res.ok) throw new Error(`Failed to create source: ${res.status}`);
	return res.json();
}

export async function fetchStats(): Promise<Stats> {
	const res = await fetch(`${API_BASE}/api/stats`);
	if (!res.ok) throw new Error(`Failed to fetch stats: ${res.status}`);
	return res.json();
}

export function connectWebSocket(onMessage: (event: TelemetryEvent) => void): WebSocket {
	const ws = new WebSocket(WS_URL);
	ws.onmessage = (e) => {
		try {
			const data = JSON.parse(e.data) as TelemetryEvent;
			onMessage(data);
		} catch {
			// ignore parse errors
		}
	};
	ws.onclose = () => {
		// Reconnect after 3 seconds
		setTimeout(() => connectWebSocket(onMessage), 3000);
	};
	return ws;
}
