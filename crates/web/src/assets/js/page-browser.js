// ── Browser viewer page (Preact + HTM + Signals) ─────────
//
// Shows active browser sessions, lets users view the browser via
// CDP screencast, send mouse/keyboard input, and manage sessions.

import { signal } from "@preact/signals";
import { html } from "htm/preact";
import { render } from "preact";
import { useEffect, useRef, useState } from "preact/hooks";
import { onEvent } from "./events.js";
import { showToast } from "./ui.js";

var sessions = signal([]);
var loading = signal(false);
var activeSession = signal(null);
var screencasting = signal(false);
var frameData = signal(null);
var frameMeta = signal(null);
var frameSeq = signal(0);
var containerEl = null;

// ── API helpers ─────────────────────────────────────────────

async function browserAction(params) {
	var res = await fetch("/api/browser/action", {
		method: "POST",
		headers: { "Content-Type": "application/json" },
		body: JSON.stringify(params),
	});
	if (!res.ok) {
		var err = await res.json().catch(() => ({ error: "request failed" }));
		throw new Error(err.error || err.code || "browser action failed");
	}
	return res.json();
}

async function fetchSessions() {
	loading.value = true;
	try {
		var res = await fetch("/api/browser/sessions");
		if (res.ok) {
			var data = await res.json();
			sessions.value = data.sessions || [];
		}
	} catch (e) {
		console.error("Failed to fetch browser sessions:", e);
	} finally {
		loading.value = false;
	}
}

// ── Screencast frame handling ───────────────────────────────

var frameUnsub = null;

function startFrameListener() {
	if (frameUnsub) return;
	frameUnsub = onEvent("browser.screencast.frame", (payload) => {
		if (payload.session_id !== activeSession.value) return;
		frameData.value = payload.data;
		frameMeta.value = payload.metadata;
		frameSeq.value = payload.sequence;
	});
}

function stopFrameListener() {
	if (frameUnsub) {
		frameUnsub();
		frameUnsub = null;
	}
}

// ── Session actions ─────────────────────────────────────────

async function startScreencast(sessionId) {
	try {
		await browserAction({
			session_id: sessionId,
			action: "start_screencast",
			quality: 60,
			max_width: 1280,
			max_height: 800,
		});
		activeSession.value = sessionId;
		screencasting.value = true;
		startFrameListener();
		showToast("Screencast started");
	} catch (e) {
		showToast(`Failed to start screencast: ${e.message}`, "error");
	}
}

async function stopScreencast(sessionId) {
	try {
		await browserAction({
			session_id: sessionId,
			action: "stop_screencast",
		});
	} catch {
		// best effort
	}
	screencasting.value = false;
	activeSession.value = null;
	frameData.value = null;
	stopFrameListener();
	showToast("Screencast stopped");
}

async function closeSession(sessionId) {
	try {
		await browserAction({
			session_id: sessionId,
			action: "close",
		});
		showToast("Session closed");
		if (activeSession.value === sessionId) {
			screencasting.value = false;
			activeSession.value = null;
			frameData.value = null;
			stopFrameListener();
		}
		await fetchSessions();
	} catch (e) {
		showToast(`Failed to close session: ${e.message}`, "error");
	}
}

async function exportCookies(sessionId) {
	try {
		var res = await browserAction({
			session_id: sessionId,
			action: "export_cookies",
		});
		if (res.cookies && res.cookies.length > 0) {
			var text = JSON.stringify(res.cookies, null, 2);
			await navigator.clipboard.writeText(text);
			showToast(`${res.cookies.length} cookies copied to clipboard`);
		} else {
			showToast("No cookies found in this session");
		}
	} catch (e) {
		showToast(`Failed to export cookies: ${e.message}`, "error");
	}
}

async function navigateSession(sessionId, url) {
	try {
		var res = await browserAction({
			session_id: sessionId,
			action: "navigate",
			url: url,
		});
		showToast(`Navigated to ${res.url || url}`);
		await fetchSessions();
	} catch (e) {
		showToast(`Navigation failed: ${e.message}`, "error");
	}
}

async function createSession() {
	try {
		var res = await browserAction({
			action: "navigate",
			url: "about:blank",
		});
		var newId = res.session_id;
		if (!newId) {
			showToast("Failed to create session", "error");
			return;
		}
		await fetchSessions();
		await startScreencast(newId);
	} catch (e) {
		showToast(`Failed to create session: ${e.message}`, "error");
	}
}

// ── Mouse/keyboard input relay ──────────────────────────────

function relayMouseEvent(e, canvas) {
	if (!(activeSession.value && screencasting.value)) return;
	var rect = canvas.getBoundingClientRect();
	var meta = frameMeta.value;
	if (!meta) return;

	// Translate canvas coordinates to browser viewport coordinates
	var scaleX = meta.device_width / rect.width;
	var scaleY = meta.device_height / rect.height;
	var x = (e.clientX - rect.left) * scaleX;
	var y = (e.clientY - rect.top) * scaleY;

	var eventType;
	switch (e.type) {
		case "mousedown":
			eventType = "mousePressed";
			break;
		case "mouseup":
			eventType = "mouseReleased";
			break;
		case "mousemove":
			eventType = "mouseMoved";
			break;
		default:
			return;
	}

	var button = ["left", "middle", "right"][e.button] || "left";

	// Fire and forget — don't await for mouse events
	browserAction({
		session_id: activeSession.value,
		action: "mouse_input",
		x: x,
		y: y,
		event_type: eventType,
		button: button,
		click_count: e.type === "mousedown" ? e.detail || 1 : 1,
	}).catch(() => {});
}

function relayKeyEvent(e) {
	if (!(activeSession.value && screencasting.value)) return;
	e.preventDefault();

	var eventType = e.type === "keydown" ? "keyDown" : "keyUp";
	var modifiers = 0;
	if (e.altKey) modifiers |= 1;
	if (e.ctrlKey) modifiers |= 2;
	if (e.metaKey) modifiers |= 4;
	if (e.shiftKey) modifiers |= 8;

	browserAction({
		session_id: activeSession.value,
		action: "keyboard_input",
		event_type: eventType,
		key: e.key,
		code: e.code,
		text: e.key.length === 1 ? e.key : undefined,
		modifiers: modifiers || undefined,
	}).catch(() => {});
}

// ── Components ──────────────────────────────────────────────

function SessionList() {
	useEffect(() => {
		fetchSessions();
		var interval = setInterval(fetchSessions, 5000);
		return () => clearInterval(interval);
	}, []);

	var s = sessions.value;
	if (loading.value && s.length === 0) {
		return html`<div class="text-xs text-[var(--muted)] p-3">Loading sessions...</div>`;
	}

	if (s.length === 0) {
		return html`<div class="text-xs text-[var(--muted)] p-3">
			No active browser sessions. Click "New Session" to create one, or sessions will appear here when the agent uses the browser tool.
		</div>`;
	}

	return html`<div class="flex flex-col gap-2">
		${s.map(
			(sess) => html`
				<div
					key=${sess.session_id}
					class="rounded-lg border border-[var(--border)] p-3 bg-[var(--surface)] flex flex-col gap-2"
				>
					<div class="flex items-center justify-between gap-2">
						<div class="flex-1 min-w-0">
							<div class="text-xs font-mono text-[var(--text-strong)] truncate" title=${sess.session_id}>
								${sess.session_id.slice(0, 12)}...
							</div>
							<div class="text-xs text-[var(--muted)] truncate mt-0.5" title=${sess.url}>
								${sess.url || "(no page loaded)"}
							</div>
						</div>
						<div class="flex items-center gap-1 shrink-0">
							${sess.sandboxed ? html`<span class="text-[10px] px-1.5 py-0.5 rounded bg-blue-500/10 text-blue-500">sandbox</span>` : null}
							${sess.screencasting ? html`<span class="text-[10px] px-1.5 py-0.5 rounded bg-green-500/10 text-green-500">live</span>` : null}
						</div>
					</div>
					<div class="flex items-center gap-1.5 text-xs">
						<span class="text-[var(--muted)]">Age: ${formatDuration(sess.age_secs)}</span>
						<span class="text-[var(--muted)]">Idle: ${formatDuration(sess.idle_secs)}</span>
					</div>
					<div class="flex items-center gap-1.5 flex-wrap">
						${
							activeSession.value === sess.session_id && screencasting.value
								? html`<button class="provider-btn-danger text-xs px-2 py-1" onClick=${() => stopScreencast(sess.session_id)}>
										Stop Viewing
									</button>`
								: html`<button class="provider-btn text-xs px-2 py-1" onClick=${() => startScreencast(sess.session_id)}>
										View
									</button>`
						}
						<button
							class="provider-btn-secondary text-xs px-2 py-1"
							onClick=${() => exportCookies(sess.session_id)}
						>
							Export Cookies
						</button>
						<button
							class="provider-btn-danger text-xs px-2 py-1"
							onClick=${() => closeSession(sess.session_id)}
						>
							Close
						</button>
					</div>
				</div>
			`,
		)}
	</div>`;
}

function formatDuration(secs) {
	if (secs < 60) return `${secs}s`;
	if (secs < 3600) return `${Math.floor(secs / 60)}m`;
	return `${Math.floor(secs / 3600)}h ${Math.floor((secs % 3600) / 60)}m`;
}

function NavigateBar() {
	var [url, setUrl] = useState("");
	var [navigating, setNavigating] = useState(false);

	async function handleNavigate(e) {
		e.preventDefault();
		if (!(url.trim() && activeSession.value)) return;
		setNavigating(true);
		await navigateSession(activeSession.value, url.trim());
		setNavigating(false);
	}

	if (!activeSession.value) return null;

	return html`<form onSubmit=${handleNavigate} class="flex items-center gap-2 mb-3">
		<input
			type="text"
			class="flex-1 px-3 py-1.5 text-xs rounded border border-[var(--border)] bg-[var(--surface)] text-[var(--text-strong)] outline-none focus:border-[var(--accent)]"
			placeholder="Navigate to URL..."
			value=${url}
			onInput=${(e) => setUrl(e.target.value)}
		/>
		<button
			type="submit"
			class="provider-btn text-xs px-3 py-1.5"
			disabled=${navigating || !url.trim()}
		>
			${navigating ? "..." : "Go"}
		</button>
	</form>`;
}

function BrowserCanvas() {
	var canvasRef = useRef(null);
	var imgRef = useRef(null);

	// Draw frame onto canvas when new frame arrives
	useEffect(() => {
		if (!(frameData.value && canvasRef.current)) return;
		var img = imgRef.current;
		if (!img) {
			img = new Image();
			imgRef.current = img;
		}
		img.onload = () => {
			var canvas = canvasRef.current;
			if (!canvas) return;
			canvas.width = img.naturalWidth;
			canvas.height = img.naturalHeight;
			var ctx = canvas.getContext("2d");
			ctx.drawImage(img, 0, 0);
		};
		img.src = `data:image/jpeg;base64,${frameData.value}`;
	}, [frameData.value]);

	// Attach input handlers
	useEffect(() => {
		var canvas = canvasRef.current;
		if (!canvas) return;

		function onMouse(e) {
			relayMouseEvent(e, canvas);
		}
		canvas.addEventListener("mousedown", onMouse);
		canvas.addEventListener("mouseup", onMouse);
		canvas.addEventListener("mousemove", onMouse);

		// Keyboard: focus the canvas to receive key events
		canvas.setAttribute("tabindex", "0");
		canvas.addEventListener("keydown", relayKeyEvent);
		canvas.addEventListener("keyup", relayKeyEvent);

		return () => {
			canvas.removeEventListener("mousedown", onMouse);
			canvas.removeEventListener("mouseup", onMouse);
			canvas.removeEventListener("mousemove", onMouse);
			canvas.removeEventListener("keydown", relayKeyEvent);
			canvas.removeEventListener("keyup", relayKeyEvent);
		};
	}, []);

	if (!(screencasting.value && activeSession.value)) {
		return html`<div class="flex-1 flex items-center justify-center text-xs text-[var(--muted)] border border-dashed border-[var(--border)] rounded-lg min-h-[300px]">
			Select a session and click "View" to see the browser
		</div>`;
	}

	if (!frameData.value) {
		return html`<div class="flex-1 flex items-center justify-center text-xs text-[var(--muted)] border border-dashed border-[var(--border)] rounded-lg min-h-[300px]">
			Waiting for first frame...
		</div>`;
	}

	return html`<div class="flex-1 flex flex-col min-h-0">
		<div class="flex items-center justify-between mb-1 text-[10px] text-[var(--muted)]">
			<span>Session: ${activeSession.value?.slice(0, 12)}...</span>
			<span>Frame #${frameSeq.value}</span>
			${frameMeta.value ? html`<span>${frameMeta.value.device_width}x${frameMeta.value.device_height}</span>` : null}
		</div>
		<canvas
			ref=${canvasRef}
			class="w-full rounded-lg border border-[var(--border)] cursor-crosshair bg-black"
			style="aspect-ratio: 16/10; object-fit: contain;"
			onClick=${(e) => e.target.focus()}
		/>
	</div>`;
}

function BrowserPage() {
	return html`<div class="flex-1 flex flex-col min-w-0 p-4 gap-3 overflow-y-auto">
		<div class="flex items-center justify-between">
			<h2 class="text-base font-medium text-[var(--text-strong)]">Browser Sessions</h2>
			<div class="flex items-center gap-2">
				<button class="provider-btn text-xs px-2 py-1" onClick=${createSession}>
					New Session
				</button>
				<button class="provider-btn-secondary text-xs px-2 py-1" onClick=${fetchSessions}>
					Refresh
				</button>
			</div>
		</div>

		<div class="text-xs text-[var(--muted)] max-w-form">
			Create browser sessions or view ones created by agents. Click "View" to see the
			browser screen, interact with mouse/keyboard to log in to websites, and agents
			will share the same cookies. Use "Export Cookies" to copy session data.
		</div>

		<div class="flex flex-col lg:flex-row gap-4 flex-1 min-h-0">
			<div class="lg:w-80 shrink-0">
				<${SessionList} />
			</div>
			<div class="flex-1 flex flex-col min-w-0">
				<${NavigateBar} />
				<${BrowserCanvas} />
			</div>
		</div>
	</div>`;
}

// ── Init / Teardown ─────────────────────────────────────────

export function initBrowser(container) {
	containerEl = container;
	render(html`<${BrowserPage} />`, container);
}

export function teardownBrowser() {
	stopFrameListener();
	if (containerEl) {
		render(null, containerEl);
		containerEl = null;
	}
	screencasting.value = false;
	activeSession.value = null;
	frameData.value = null;
}
