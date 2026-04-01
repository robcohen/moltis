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
var frameMime = signal("image/jpeg");
var frameMeta = signal(null);
var frameSeq = signal(0);
var creating = signal(false);
var fetching = signal(false);
// Screenshot cache: session_id → { data, scale }
var screenshotCache = {};
var containerEl = null;

// ── URL helpers ─────────────────────────────────────────────

/** Return true if text looks like a navigable URL or domain. */
function looksLikeUrl(text) {
	return /^https?:\/\//i.test(text) || /^[a-z0-9]([a-z0-9-]*\.)+[a-z]{2,}/i.test(text);
}

/** Ensure URL has a scheme — prepend https:// for bare domains. */
function normalizeUrl(input) {
	var text = input.trim();
	if (/^https?:\/\//i.test(text)) return text;
	if (looksLikeUrl(text)) return `https://${text}`;
	// Treat as Google search
	return `https://www.google.com/search?q=${encodeURIComponent(text)}`;
}

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
	var data = await res.json();
	if (data.success === false) {
		throw new Error(data.error || "browser action failed");
	}
	return data;
}

async function fetchSessions() {
	loading.value = true;
	try {
		var res = await fetch("/api/browser/sessions");
		if (res.ok) {
			var data = await res.json();
			sessions.value = data.sessions || [];
			prefetchScreenshots(data.sessions || []);
		}
	} catch (e) {
		console.error("Failed to fetch browser sessions:", e);
	} finally {
		loading.value = false;
	}
}

/** Prefetch screenshots for sessions we don't have cached yet. */
function prefetchScreenshots(sessionList) {
	for (var sess of sessionList) {
		if (screenshotCache[sess.session_id]) continue;
		if (!sess.url || sess.url === "about:blank") continue;
		// Fire and forget — populate cache in background
		browserAction({ session_id: sess.session_id, action: "screenshot" })
			.then((snap) => {
				if (snap.screenshot) {
					screenshotCache[sess.session_id] = {
						data: snap.screenshot,
						scale: snap.screenshot_scale || 1,
					};
				}
			})
			.catch(() => {});
	}
}

// ── Google suggestions ──────────────────────────────────────

var suggestAbort = null;

async function fetchSuggestions(query) {
	if (suggestAbort) suggestAbort.abort();
	if (!query || query.length < 2) return [];
	var ctrl = new AbortController();
	suggestAbort = ctrl;
	try {
		var url = `https://suggestqueries.google.com/complete/search?client=firefox&q=${encodeURIComponent(query)}`;
		var res = await fetch(url, { signal: ctrl.signal });
		if (!res.ok) return [];
		var data = await res.json();
		// Firefox format: ["query", ["suggestion1", "suggestion2", ...]]
		return (data[1] || []).slice(0, 5);
	} catch {
		return [];
	}
}

// ── Screencast frame handling ───────────────────────────────

var frameUnsub = null;

function startFrameListener() {
	if (frameUnsub) return;
	frameUnsub = onEvent("browser.screencast.frame", (payload) => {
		if (payload.session_id !== activeSession.value) return;
		frameData.value = payload.data;
		frameMime.value = "image/jpeg";
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

async function navigateSession(sessionId, rawUrl) {
	var url = normalizeUrl(rawUrl);
	try {
		await browserAction({
			session_id: sessionId,
			action: "navigate",
			url: url,
		});
		await fetchSessions();
		// Auto-start screencast after navigating to a real page
		if (!screencasting.value && activeSession.value === sessionId) {
			await startScreencast(sessionId);
		}
	} catch (e) {
		showToast(`Navigation failed: ${e.message}`, "error");
	}
}

async function createSession() {
	if (creating.value) return;
	creating.value = true;

	// Stop current screencast before creating a new session
	if (screencasting.value && activeSession.value) {
		stopScreencast(activeSession.value);
	}

	// Immediately add a placeholder session to the list and select it
	var placeholderId = `creating-${Date.now()}`;
	sessions.value = [
		{ session_id: placeholderId, url: "", sandboxed: false, age_secs: 0, idle_secs: 0, creating: true },
		...sessions.value,
	];
	frameData.value = null;
	frameMeta.value = null;
	screencasting.value = false;
	activeSession.value = placeholderId;

	try {
		var res = await browserAction({
			action: "navigate",
			url: "about:blank",
		});
		var newId = res.session_id;
		if (!newId) {
			showToast("Failed to create session", "error");
			// Remove placeholder
			sessions.value = sessions.value.filter((s) => s.session_id !== placeholderId);
			activeSession.value = null;
			return;
		}
		// Replace placeholder with real session
		activeSession.value = newId;
		await fetchSessions();
	} catch (e) {
		showToast(`Failed to create session: ${e.message}`, "error");
		sessions.value = sessions.value.filter((s) => s.session_id !== placeholderId);
		activeSession.value = null;
	} finally {
		creating.value = false;
	}
}

// ── Mouse/keyboard/scroll input relay ────────────────────────

function canvasCoords(e, canvas) {
	var rect = canvas.getBoundingClientRect();
	var meta = frameMeta.value;
	if (!meta) return null;
	var scaleX = meta.device_width / rect.width;
	var scaleY = meta.device_height / rect.height;
	return {
		x: (e.clientX - rect.left) * scaleX,
		y: (e.clientY - rect.top) * scaleY,
	};
}

function relayMouseEvent(e, canvas) {
	if (!(activeSession.value && screencasting.value)) return;
	// Prevent text selection, drag, and context menu on the canvas
	e.preventDefault();

	var coords = canvasCoords(e, canvas);
	if (!coords) return;

	var eventType;
	switch (e.type) {
		case "mousedown":
			eventType = "mousePressed";
			canvas.focus();
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

	browserAction({
		session_id: activeSession.value,
		action: "mouse_input",
		x: coords.x,
		y: coords.y,
		event_type: eventType,
		button: button,
		click_count: e.type === "mousedown" ? e.detail || 1 : 1,
	}).catch(() => {});
}

function relayWheelEvent(e, canvas) {
	if (!(activeSession.value && screencasting.value)) return;
	e.preventDefault();

	var coords = canvasCoords(e, canvas);
	if (!coords) return;

	browserAction({
		session_id: activeSession.value,
		action: "mouse_input",
		x: coords.x,
		y: coords.y,
		event_type: "mouseWheel",
		button: "left",
		click_count: 0,
		delta_x: e.deltaX,
		delta_y: e.deltaY,
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

	async function selectSession(sessionId) {
		if (activeSession.value === sessionId) return;
		// Stop current screencast if switching sessions
		if (screencasting.value && activeSession.value) {
			stopScreencast(activeSession.value);
		}
		activeSession.value = sessionId;
		frameData.value = null;

		// Use prefetch cache for instant display, otherwise fetch live
		var cached = screenshotCache[sessionId];
		if (cached) {
			frameData.value = cached.data;
			frameMime.value = "image/png";
			frameMeta.value = {
				device_width: 1280 * cached.scale,
				device_height: 800 * cached.scale,
			};
		} else {
			fetching.value = true;
			try {
				var snap = await browserAction({
					session_id: sessionId,
					action: "screenshot",
				});
				if (snap.screenshot) {
					frameData.value = snap.screenshot;
					frameMime.value = "image/png";
					frameMeta.value = {
						device_width: snap.screenshot_scale ? 1280 * snap.screenshot_scale : 1280,
						device_height: snap.screenshot_scale ? 800 * snap.screenshot_scale : 800,
					};
					screenshotCache[sessionId] = {
						data: snap.screenshot,
						scale: snap.screenshot_scale || 1,
					};
				}
			} catch {
				// Best effort
			} finally {
				fetching.value = false;
			}
		}

		screencasting.value = true;
		await startScreencast(sessionId);
	}

	return html`<div class="flex flex-col gap-2">
		${s.map(
			(sess) => {
				var isActive = activeSession.value === sess.session_id;
				return html`
				<div
					key=${sess.session_id}
					class="rounded-lg border p-3 flex flex-col gap-2 transition-colors ${sess.creating ? 'border-[var(--border)] bg-[var(--surface)] opacity-75' : isActive ? 'border-[var(--accent)] bg-[var(--accent)]/5' : 'border-[var(--border)] bg-[var(--surface)] hover:border-[var(--accent)]/50 cursor-pointer'}"
					onClick=${() => { if (!sess.creating) selectSession(sess.session_id); }}
				>
					<div class="flex items-center justify-between gap-2">
						<div class="flex-1 min-w-0">
							<div class="text-xs font-mono text-[var(--text-strong)] truncate" title=${sess.session_id}>
								${sess.creating ? "New session" : sess.session_id.slice(0, 12) + "..."}
							</div>
							<div class="text-xs text-[var(--muted)] truncate mt-0.5" title=${sess.url}>
								${sess.creating ? "Starting browser\u2026" : sess.url || "(no page loaded)"}
							</div>
						</div>
						<div class="flex items-center gap-1 shrink-0">
							${sess.sandboxed ? html`<span class="text-[10px] px-1.5 py-0.5 rounded bg-blue-500/10 text-blue-500">sandbox</span>` : null}
							${sess.creating
								? html`<span class="text-[10px] px-1.5 py-0.5 rounded bg-orange-500/10 text-orange-500">creating</span>`
								: isActive && screencasting.value
									? html`<span class="text-[10px] px-1.5 py-0.5 rounded bg-green-500/10 text-green-500">live</span>`
									: sess.url && sess.url !== "about:blank"
										? html`<span class="text-[10px] px-1.5 py-0.5 rounded bg-yellow-500/10 text-yellow-500">paused</span>`
										: null}
						</div>
					</div>
					<div class="flex items-center gap-1.5 text-xs">
						<span class="text-[var(--muted)]">Age: ${formatDuration(sess.age_secs)}</span>
						<span class="text-[var(--muted)]">Idle: ${formatDuration(sess.idle_secs)}</span>
					</div>
					${!sess.creating ? html`<div class="flex items-center justify-end" onClick=${(e) => e.stopPropagation()}>
						<button
							class="text-[10px] text-[var(--muted)] hover:text-[var(--error)] transition-colors"
							onClick=${() => closeSession(sess.session_id)}
						>
							Close
						</button>
					</div>` : null}
				</div>
			`; },
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
	var lastSessionRef = useRef(null);

	// Reset URL bar when switching sessions
	var currentSession = activeSession.value;
	if (currentSession !== lastSessionRef.current) {
		lastSessionRef.current = currentSession;
		// Find the session's current URL to display
		var sess = sessions.value.find((s) => s.session_id === currentSession);
		var sessionUrl = sess?.url && sess.url !== "about:blank" ? sess.url : "";
		setUrl(sessionUrl);
	}
	var [suggestions, setSuggestions] = useState([]);
	var [selectedIdx, setSelectedIdx] = useState(-1);
	var [showDropdown, setShowDropdown] = useState(false);
	var debounceRef = useRef(null);
	var wrapperRef = useRef(null);

	// Close dropdown on outside click
	useEffect(() => {
		function onClickOutside(e) {
			if (wrapperRef.current && !wrapperRef.current.contains(e.target)) {
				setShowDropdown(false);
			}
		}
		document.addEventListener("mousedown", onClickOutside);
		return () => document.removeEventListener("mousedown", onClickOutside);
	}, []);

	function buildItems(query, googleSuggestions) {
		var items = [];
		var q = query.trim().toLowerCase();
		if (!q) return items;

		// If it looks like a URL, offer direct navigation
		if (looksLikeUrl(q)) {
			var dest = /^https?:\/\//i.test(q) ? q : `https://${q}`;
			items.push({ type: "url", label: dest, icon: "\u{1F310}" });
		}

		// Google suggestions
		for (var s of googleSuggestions) {
			items.push({ type: "search", label: s, icon: "\u{1F50D}" });
		}

		// Session URLs that match
		for (var sess of sessions.value) {
			if (sess.url && sess.url !== "about:blank" && sess.url.toLowerCase().includes(q)) {
				// Avoid duplicate if already in list
				if (!items.some((i) => i.label === sess.url)) {
					items.push({ type: "history", label: sess.url, icon: "\u{1F4C4}" });
				}
			}
		}

		// If no URL match and no items yet, offer search
		if (!looksLikeUrl(q) && !items.some((i) => i.type === "url")) {
			items.unshift({
				type: "search-go",
				label: `Search Google for "${q}"`,
				value: `https://www.google.com/search?q=${encodeURIComponent(q)}`,
				icon: "\u{1F50D}",
			});
		}

		return items.slice(0, 8);
	}

	function onInput(e) {
		var val = e.target.value;
		setUrl(val);
		setSelectedIdx(-1);

		if (debounceRef.current) clearTimeout(debounceRef.current);
		if (val.trim().length < 2) {
			setSuggestions([]);
			setShowDropdown(false);
			return;
		}

		debounceRef.current = setTimeout(async () => {
			var google = await fetchSuggestions(val.trim());
			var items = buildItems(val, google);
			setSuggestions(items);
			setShowDropdown(items.length > 0);
		}, 200);
	}

	function selectItem(item) {
		var nav = item.value || item.label;
		setUrl(nav);
		setShowDropdown(false);
		setSuggestions([]);
		doNavigate(nav);
	}

	async function doNavigate(raw) {
		if (!activeSession.value) return;
		setNavigating(true);
		await navigateSession(activeSession.value, raw);
		setNavigating(false);
	}

	function onKeyDown(e) {
		if (!showDropdown || suggestions.length === 0) {
			if (e.key === "Enter") {
				e.preventDefault();
				if (url.trim()) doNavigate(url.trim());
			}
			return;
		}

		switch (e.key) {
			case "ArrowDown":
				e.preventDefault();
				setSelectedIdx((i) => Math.min(i + 1, suggestions.length - 1));
				break;
			case "ArrowUp":
				e.preventDefault();
				setSelectedIdx((i) => Math.max(i - 1, -1));
				break;
			case "Enter":
				e.preventDefault();
				if (selectedIdx >= 0 && suggestions[selectedIdx]) {
					selectItem(suggestions[selectedIdx]);
				} else if (url.trim()) {
					setShowDropdown(false);
					doNavigate(url.trim());
				}
				break;
			case "Escape":
				setShowDropdown(false);
				setSelectedIdx(-1);
				break;
		}
	}

	if (!activeSession.value) return null;

	return html`<div ref=${wrapperRef} class="relative mb-3">
		<form onSubmit=${(e) => { e.preventDefault(); if (url.trim()) { setShowDropdown(false); doNavigate(url.trim()); } }} class="flex items-center gap-2">
			<input
				type="text"
				class="flex-1 rounded border border-[var(--border)] bg-[var(--surface)] text-[var(--text-strong)] outline-none focus:border-[var(--accent)]"
				style="padding: 3px 10px; font-size: 0.75rem;"
				placeholder="Search or enter URL..."
				value=${url}
				onInput=${onInput}
				onKeyDown=${onKeyDown}
				onFocus=${() => { if (suggestions.length > 0) setShowDropdown(true); }}
				autocomplete="off"
			/>
			<button
				type="submit"
				class="provider-btn provider-btn-sm"
				disabled=${navigating || !url.trim()}
			>
				${navigating ? "\u2026" : "Go"}
			</button>
		</form>
		${showDropdown && suggestions.length > 0 ? html`
			<div class="absolute left-0 right-0 top-full mt-1 rounded-lg border border-[var(--border)] bg-[var(--surface)] shadow-lg z-50 overflow-hidden" style="max-height: 320px; overflow-y: auto;">
				${suggestions.map((item, idx) => html`
					<button
						key=${idx}
						class="w-full text-left px-3 py-2 text-xs flex items-center gap-2 hover:bg-[var(--bg-hover)] ${idx === selectedIdx ? 'bg-[var(--bg-hover)]' : ''}"
						style="border: none; background: ${idx === selectedIdx ? 'var(--bg-hover)' : 'transparent'}; cursor: pointer;"
						onMouseDown=${(e) => { e.preventDefault(); selectItem(item); }}
						onMouseEnter=${() => setSelectedIdx(idx)}
					>
						<span class="shrink-0 w-4 text-center">${item.icon}</span>
						<span class="truncate text-[var(--text-strong)]">${item.label}</span>
						${item.type === "url" ? html`<span class="ml-auto text-[var(--muted)] text-[10px] shrink-0">Go to site</span>` : null}
						${item.type === "history" ? html`<span class="ml-auto text-[var(--muted)] text-[10px] shrink-0">Open tab</span>` : null}
					</button>
				`)}
			</div>
		` : null}
	</div>`;
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
		img.src = `data:${frameMime.value};base64,${frameData.value}`;
	}, [frameData.value]);

	// Attach input handlers
	useEffect(() => {
		var canvas = canvasRef.current;
		if (!canvas) return;

		function onMouse(e) {
			relayMouseEvent(e, canvas);
		}
		function onWheel(e) {
			relayWheelEvent(e, canvas);
		}
		function onContextMenu(e) {
			e.preventDefault();
		}
		canvas.addEventListener("mousedown", onMouse);
		canvas.addEventListener("mouseup", onMouse);
		canvas.addEventListener("mousemove", onMouse);
		canvas.addEventListener("wheel", onWheel, { passive: false });
		canvas.addEventListener("contextmenu", onContextMenu);

		// Keyboard: focus the canvas to receive key events
		canvas.setAttribute("tabindex", "0");
		canvas.addEventListener("keydown", relayKeyEvent);
		canvas.addEventListener("keyup", relayKeyEvent);

		return () => {
			canvas.removeEventListener("mousedown", onMouse);
			canvas.removeEventListener("mouseup", onMouse);
			canvas.removeEventListener("mousemove", onMouse);
			canvas.removeEventListener("wheel", onWheel);
			canvas.removeEventListener("contextmenu", onContextMenu);
			canvas.removeEventListener("keydown", relayKeyEvent);
			canvas.removeEventListener("keyup", relayKeyEvent);
		};
	}, []);

	if (!activeSession.value) {
		return html`<div class="flex-1 flex items-center justify-center text-xs text-[var(--muted)] border border-dashed border-[var(--border)] rounded-lg min-h-[300px]">
			Select a session to view the browser
		</div>`;
	}

	if (!screencasting.value && !fetching.value && !frameData.value) {
		return html`<div class="flex-1 flex items-center justify-center text-xs text-[var(--muted)] border border-dashed border-[var(--border)] rounded-lg min-h-[300px]">
			Enter a URL above to start browsing
		</div>`;
	}

	if (fetching.value && !frameData.value) {
		return html`<div class="flex-1 flex items-center justify-center text-xs text-[var(--muted)] border border-dashed border-[var(--border)] rounded-lg min-h-[300px]">
			Fetching browser view\u2026
		</div>`;
	}

	if (!frameData.value) {
		return html`<div class="flex-1 flex items-center justify-center text-xs text-[var(--muted)] border border-dashed border-[var(--border)] rounded-lg min-h-[300px]">
			Waiting for first frame\u2026
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
				<button class="provider-btn provider-btn-sm" onClick=${createSession} disabled=${creating.value}>
					${creating.value ? "Creating\u2026" : "New Session"}
				</button>
				<button class="provider-btn provider-btn-secondary provider-btn-sm" onClick=${fetchSessions}>
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
