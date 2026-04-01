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
// Current URL for the active session — kept in sync with the remote browser.
var currentUrl = signal("");
// Screenshot cache: session_id → { data }
var screenshotCache = {};
// Track placeholder IDs so fetchSessions doesn't remove them
var placeholderIds = new Set();
var urlPollTimer = null;
// Scroll state for scrollbar overlay
var scrollInfo = signal(null); // { scrollTop, scrollHeight, viewportHeight }
var sessionHistory = signal([]);
var selectedHistorySession = signal(null);
var actionLog = signal([]);
var containerEl = null;

// ── URL helpers ─────────────────────────────────────────────

function looksLikeUrl(text) {
	return /^https?:\/\//i.test(text) || /^[a-z0-9]([a-z0-9-]*\.)+[a-z]{2,}/i.test(text);
}

function normalizeUrl(input) {
	var text = input.trim();
	if (/^https?:\/\//i.test(text)) return text;
	if (looksLikeUrl(text)) return `https://${text}`;
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
			var realSessions = data.sessions || [];
			// Preserve placeholder entries that haven't resolved yet
			var placeholders = sessions.value.filter((s) => placeholderIds.has(s.session_id));
			sessions.value = [...placeholders, ...realSessions];
			prefetchScreenshots(realSessions);
			fetchHistory();
		}
	} catch (e) {
		console.error("Failed to fetch browser sessions:", e);
	} finally {
		loading.value = false;
	}
}

function prefetchScreenshots(sessionList) {
	for (var sess of sessionList) {
		if (screenshotCache[sess.session_id]) continue;
		if (!sess.url || sess.url === "about:blank") continue;
		browserAction({ session_id: sess.session_id, action: "screenshot" })
			.then((snap) => {
				if (snap.screenshot) {
					screenshotCache[sess.session_id] = { data: snap.screenshot };
				}
			})
			.catch(() => {});
	}
}

// ── Session history ─────────────────────────────────────────

async function fetchHistory() {
	try {
		var res = await fetch("/api/browser/history");
		if (res.ok) {
			var data = await res.json();
			sessionHistory.value = data.sessions || [];
		}
	} catch {
		// best effort
	}
}

async function fetchActionLog(sessionId) {
	try {
		var res = await fetch(`/api/browser/actions/${sessionId}`);
		if (res.ok) {
			var data = await res.json();
			actionLog.value = data.actions || [];
		}
	} catch {
		actionLog.value = [];
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
		return (data[1] || []).slice(0, 5);
	} catch {
		return [];
	}
}

// ── Screencast frame handling ───────────────────────────────

var frameUnsub = null;

// Keep the frame listener always active — it filters by session_id.
// This avoids teardown/setup gaps when switching sessions.
function ensureFrameListener() {
	if (frameUnsub) return;
	frameUnsub = onEvent("browser.screencast.frame", (payload) => {
		if (payload.session_id !== activeSession.value) return;
		frameData.value = payload.data;
		frameMime.value = "image/jpeg";
		frameMeta.value = payload.metadata;
		frameSeq.value = payload.sequence;
		// Update cache with latest frame so switching back is instant
		screenshotCache[payload.session_id] = { data: payload.data, mime: "image/jpeg", meta: payload.metadata };
	});
}

function stopFrameListener() {
	if (frameUnsub) {
		frameUnsub();
		frameUnsub = null;
	}
}

// ── URL polling — detect in-page navigations (link clicks, etc.) ────

function startUrlPolling() {
	stopUrlPolling();
	urlPollTimer = setInterval(async () => {
		var sid = activeSession.value;
		if (!sid || !screencasting.value) return;
		try {
			var [urlRes, scrollRes] = await Promise.all([
				browserAction({ session_id: sid, action: "get_url" }),
				browserAction({ session_id: sid, action: "evaluate", code: "JSON.stringify({scrollTop:window.scrollY,scrollHeight:document.documentElement.scrollHeight,viewportHeight:window.innerHeight})" }),
			]);
			if (activeSession.value !== sid) return;
			if (urlRes.url) currentUrl.value = urlRes.url;
			if (scrollRes.result) {
				try { scrollInfo.value = JSON.parse(scrollRes.result); } catch {}
			}
		} catch {
			// best effort
		}
	}, 2000);
}

function stopUrlPolling() {
	if (urlPollTimer) {
		clearInterval(urlPollTimer);
		urlPollTimer = null;
	}
}

// ── Session actions ─────────────────────────────────────────

async function sendStartScreencast(sessionId) {
	try {
		await browserAction({
			session_id: sessionId,
			action: "start_screencast",
			quality: 60,
			max_width: 1280,
			max_height: 800,
		});
		screencasting.value = true;
		ensureFrameListener();
	} catch (e) {
		showToast(`Failed to start screencast: ${e.message}`, "error");
	}
}

async function closeSession(sessionId) {
	try {
		await browserAction({ session_id: sessionId, action: "close" });
		if (activeSession.value === sessionId) {
			screencasting.value = false;
			activeSession.value = null;
			frameData.value = null;
			currentUrl.value = "";
			stopUrlPolling();
		}
		delete screenshotCache[sessionId];
		await fetchSessions();
	} catch (e) {
		showToast(`Failed to close session: ${e.message}`, "error");
	}
}

async function navigateSession(sessionId, rawUrl) {
	var url = normalizeUrl(rawUrl);
	try {
		var res = await browserAction({ session_id: sessionId, action: "navigate", url: url });
		currentUrl.value = res.url || url;
		delete screenshotCache[sessionId];
		await fetchSessions();
		if (!screencasting.value && activeSession.value === sessionId) {
			await sendStartScreencast(sessionId);
		}
	} catch (e) {
		showToast(`Navigation failed: ${e.message}`, "error");
	}
}

async function createSession() {
	if (creating.value) return;
	creating.value = true;

	// Don't stop previous screencast — let it run in background
	screencasting.value = false;

	var placeholderId = `creating-${Date.now()}`;
	placeholderIds.add(placeholderId);
	sessions.value = [
		{ session_id: placeholderId, url: "", sandboxed: false, age_secs: 0, idle_secs: 0, creating: true },
		...sessions.value,
	];
	frameData.value = null;
	frameMeta.value = null;
	currentUrl.value = "";
	selectedHistorySession.value = null;
	activeSession.value = placeholderId;

	try {
		var res = await browserAction({ action: "navigate", url: "about:blank" });
		var newId = res.session_id;
		if (!newId) {
			showToast("Failed to create session", "error");
			sessions.value = sessions.value.filter((s) => s.session_id !== placeholderId);
			placeholderIds.delete(placeholderId);
			activeSession.value = null;
			return;
		}
		placeholderIds.delete(placeholderId);
		activeSession.value = newId;
		await fetchSessions();
	} catch (e) {
		showToast(`Failed to create session: ${e.message}`, "error");
		sessions.value = sessions.value.filter((s) => s.session_id !== placeholderId);
		placeholderIds.delete(placeholderId);
		activeSession.value = null;
	} finally {
		creating.value = false;
	}
}

// ── Select / switch session ─────────────────────────────────

async function selectSession(sessionId) {
	if (activeSession.value === sessionId) return;

	// Don't stop the previous session's screencast — it keeps running
	// in the background. The frame listener filters by session_id so
	// old frames are ignored. Stopping/starting screencasts rapidly
	// overwhelms CDP and can crash the browser container.
	screencasting.value = false;

	activeSession.value = sessionId;
	selectedHistorySession.value = null; // exit history view
	frameData.value = null;
	fetching.value = true;

	// Set URL from session list immediately
	var sess = sessions.value.find((s) => s.session_id === sessionId);
	currentUrl.value = sess?.url && sess.url !== "about:blank" ? sess.url : "";

	// Show cached frame/screenshot instantly, or fetch a fresh screenshot
	var cached = screenshotCache[sessionId];
	if (cached) {
		frameData.value = cached.data;
		frameMime.value = cached.mime || "image/png";
		if (cached.meta) {
			frameMeta.value = cached.meta;
		}
		fetching.value = false;
	} else {
		try {
			var snap = await browserAction({ session_id: sessionId, action: "screenshot" });
			if (snap.screenshot && activeSession.value === sessionId) {
				applyScreenshot(snap.screenshot);
				screenshotCache[sessionId] = { data: snap.screenshot, mime: "image/png" };
			}
		} catch {
			// Session might have died — refresh list
			await fetchSessions();
		} finally {
			fetching.value = false;
		}
	}

	// Guard: session might have changed during await
	if (activeSession.value !== sessionId) return;
	await sendStartScreencast(sessionId);
	startUrlPolling();
}

function applyScreenshot(data) {
	frameData.value = data;
	frameMime.value = "image/png";
	// Derive dimensions from the actual image to get correct coordinate mapping.
	// The screenshot is at viewport resolution, so naturalWidth/Height = viewport size.
	var img = new Image();
	img.onload = () => {
		frameMeta.value = {
			device_width: img.naturalWidth,
			device_height: img.naturalHeight,
			offset_top: 0,
		};
	};
	img.src = `data:image/png;base64,${data}`;
}

// ── Mouse/keyboard/scroll input relay ────────────────────────

function canvasCoords(e, canvas) {
	var meta = frameMeta.value;
	if (!meta) return null;
	// Map from canvas CSS position to CDP viewport coordinates.
	// device_width/device_height are the CSS viewport size in DIP.
	// offset_top accounts for browser chrome (infobar, etc).
	var scaleX = meta.device_width / canvas.clientWidth;
	var scaleY = meta.device_height / canvas.clientHeight;
	return {
		x: e.offsetX * scaleX,
		y: e.offsetY * scaleY + (meta.offset_top || 0),
	};
}

var lastMoveTime = 0;

function relayMouseEvent(e, canvas) {
	if (!(activeSession.value && screencasting.value)) return;
	e.preventDefault();

	if (e.type === "mousemove") {
		var now = Date.now();
		if (now - lastMoveTime < 50) return;
		lastMoveTime = now;
	}

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

	browserAction({
		session_id: activeSession.value,
		action: "mouse_input",
		x: coords.x,
		y: coords.y,
		event_type: eventType,
		button: ["left", "middle", "right"][e.button] || "left",
		click_count: e.type === "mousedown" ? e.detail || 1 : 1,
	}).catch(() => {});
}

var wheelAccum = { x: 0, y: 0, cx: 0, cy: 0 };
var wheelTimer = null;

function flushWheel() {
	wheelTimer = null;
	if (wheelAccum.x === 0 && wheelAccum.y === 0) return;
	if (!activeSession.value) return;
	var dx = wheelAccum.x;
	var dy = wheelAccum.y;
	var cx = wheelAccum.cx;
	var cy = wheelAccum.cy;
	wheelAccum.x = 0;
	wheelAccum.y = 0;

	browserAction({
		session_id: activeSession.value,
		action: "mouse_input",
		x: cx,
		y: cy,
		event_type: "mouseWheel",
		button: "left",
		click_count: 0,
		delta_x: dx,
		delta_y: dy,
	}).catch(() => {});
}

function relayWheelEvent(e, canvas) {
	if (!(activeSession.value && screencasting.value)) return;
	e.preventDefault();

	var coords = canvasCoords(e, canvas);
	if (!coords) return;

	wheelAccum.x += e.deltaX;
	wheelAccum.y += e.deltaY;
	wheelAccum.cx = coords.x;
	wheelAccum.cy = coords.y;

	if (!wheelTimer) {
		wheelTimer = setTimeout(flushWheel, 50);
	}
}

function relayKeyEvent(e) {
	if (!(activeSession.value && screencasting.value)) return;
	e.preventDefault();

	var modifiers = 0;
	if (e.altKey) modifiers |= 1;
	if (e.ctrlKey) modifiers |= 2;
	if (e.metaKey) modifiers |= 4;
	if (e.shiftKey) modifiers |= 8;

	browserAction({
		session_id: activeSession.value,
		action: "keyboard_input",
		event_type: e.type === "keydown" ? "keyDown" : "keyUp",
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
								${sess.creating ? "New session" : sess.session_id}
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
	var [editing, setEditing] = useState(false);
	var [editUrl, setEditUrl] = useState("");
	var [navigating, setNavigating] = useState(false);

	// Show the live currentUrl unless the user is actively editing
	var displayUrl = editing ? editUrl : currentUrl.value;

	var [suggestions, setSuggestions] = useState([]);
	var [selectedIdx, setSelectedIdx] = useState(-1);
	var [showDropdown, setShowDropdown] = useState(false);
	var debounceRef = useRef(null);
	var wrapperRef = useRef(null);

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

		if (looksLikeUrl(q)) {
			var dest = /^https?:\/\//i.test(q) ? q : `https://${q}`;
			items.push({ type: "url", label: dest, icon: "\u{1F310}" });
		}

		for (var s of googleSuggestions) {
			items.push({ type: "search", label: s, icon: "\u{1F50D}" });
		}

		for (var sess of sessions.value) {
			if (sess.url && sess.url !== "about:blank" && sess.url.toLowerCase().includes(q)) {
				if (!items.some((i) => i.label === sess.url)) {
					items.push({ type: "history", label: sess.url, icon: "\u{1F4C4}" });
				}
			}
		}

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
		setEditing(true);
		setEditUrl(val);
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
		setShowDropdown(false);
		setSuggestions([]);
		setEditing(false);
		doNavigate(nav);
	}

	async function doNavigate(raw) {
		if (!activeSession.value) return;
		setEditing(false);
		setNavigating(true);
		await navigateSession(activeSession.value, raw);
		setNavigating(false);
	}

	function onKeyDown(e) {
		if (!showDropdown || suggestions.length === 0) {
			if (e.key === "Enter") {
				e.preventDefault();
				if (displayUrl.trim()) doNavigate(displayUrl.trim());
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
				} else if (displayUrl.trim()) {
					setShowDropdown(false);
					doNavigate(displayUrl.trim());
				}
				break;
			case "Escape":
				setShowDropdown(false);
				setSelectedIdx(-1);
				setEditing(false);
				break;
		}
	}

	if (!activeSession.value) return null;

	return html`<div ref=${wrapperRef} class="relative mb-3">
		<form onSubmit=${(e) => { e.preventDefault(); if (displayUrl.trim()) { setShowDropdown(false); doNavigate(displayUrl.trim()); } }} class="flex items-center gap-2">
			<input
				type="text"
				class="flex-1 rounded border border-[var(--border)] bg-[var(--surface)] text-[var(--text-strong)] outline-none focus:border-[var(--accent)]"
				style="padding: 3px 10px; font-size: 0.75rem;"
				placeholder="Search or enter URL..."
				value=${displayUrl}
				onInput=${onInput}
				onKeyDown=${onKeyDown}
				onFocus=${() => { setEditing(true); setEditUrl(currentUrl.value); if (suggestions.length > 0) setShowDropdown(true); }}
				onBlur=${() => { setTimeout(() => setEditing(false), 200); }}
				autocomplete="off"
			/>
			<button
				type="submit"
				class="provider-btn provider-btn-sm"
				disabled=${navigating || !displayUrl.trim()}
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
	var cleanupRef = useRef(null);

	// rAF-gated rendering: store latest frame, draw at display refresh rate.
	// Avoids wasted draws when frames arrive faster than the monitor refreshes.
	var pendingFrameRef = useRef(null);
	var rafRef = useRef(null);

	useEffect(() => {
		if (!frameData.value) return;
		pendingFrameRef.current = { data: frameData.value, mime: frameMime.value };

		if (!rafRef.current) {
			rafRef.current = requestAnimationFrame(() => {
				rafRef.current = null;
				var pending = pendingFrameRef.current;
				if (!(pending && canvasRef.current)) return;

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
				img.src = `data:${pending.mime};base64,${pending.data}`;
			});
		}

		return () => {
			if (rafRef.current) {
				cancelAnimationFrame(rafRef.current);
				rafRef.current = null;
			}
		};
	}, [frameData.value]);

	function canvasRefCallback(canvas) {
		if (cleanupRef.current) {
			cleanupRef.current();
			cleanupRef.current = null;
		}
		canvasRef.current = canvas;
		if (!canvas) return;

		function onMouse(e) { relayMouseEvent(e, canvas); }
		function onWheel(e) { relayWheelEvent(e, canvas); }
		function onCtx(e) { e.preventDefault(); }
		canvas.addEventListener("mousedown", onMouse);
		canvas.addEventListener("mouseup", onMouse);
		canvas.addEventListener("mousemove", onMouse);
		canvas.addEventListener("wheel", onWheel, { passive: false });
		canvas.addEventListener("contextmenu", onCtx);
		canvas.setAttribute("tabindex", "0");
		canvas.addEventListener("keydown", relayKeyEvent);
		canvas.addEventListener("keyup", relayKeyEvent);
		canvas.focus();

		cleanupRef.current = () => {
			canvas.removeEventListener("mousedown", onMouse);
			canvas.removeEventListener("mouseup", onMouse);
			canvas.removeEventListener("mousemove", onMouse);
			canvas.removeEventListener("wheel", onWheel);
			canvas.removeEventListener("contextmenu", onCtx);
			canvas.removeEventListener("keydown", relayKeyEvent);
			canvas.removeEventListener("keyup", relayKeyEvent);
		};
	}

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

	// Scrollbar calculations
	var si = scrollInfo.value;
	var showScrollbar = si && si.scrollHeight > si.viewportHeight;
	var thumbPct = showScrollbar ? (si.viewportHeight / si.scrollHeight) * 100 : 100;
	var thumbTopPct = showScrollbar ? (si.scrollTop / si.scrollHeight) * 100 : 0;

	function onScrollbarClick(e) {
		if (!showScrollbar || !activeSession.value) return;
		var rect = e.currentTarget.getBoundingClientRect();
		var clickPct = (e.clientY - rect.top) / rect.height;
		var targetScroll = clickPct * si.scrollHeight - si.viewportHeight / 2;
		browserAction({
			session_id: activeSession.value,
			action: "evaluate",
			code: `window.scrollTo(0, ${Math.max(0, Math.round(targetScroll))})`,
		}).catch(() => {});
	}

	return html`<div class="flex-1 flex flex-col min-h-0">
		<div class="flex items-center justify-between mb-1 text-[10px] text-[var(--muted)]">
			<span>Session: ${activeSession.value}</span>
			<span>Frame #${frameSeq.value}</span>
			${frameMeta.value ? html`<span>${frameMeta.value.device_width}x${frameMeta.value.device_height}</span>` : null}
		</div>
		<div class="relative inline-block w-full">
			<canvas
				ref=${canvasRefCallback}
				class="block w-full rounded-lg border border-[var(--border)] cursor-crosshair bg-black"
				style="aspect-ratio: ${frameMeta.value ? `${frameMeta.value.device_width} / ${frameMeta.value.device_height}` : '16 / 9'};"
			/>
			${showScrollbar ? html`
				<div
					class="absolute top-0 right-0 w-2 h-full rounded-r-lg cursor-pointer opacity-40 hover:opacity-70 transition-opacity"
					style="background: var(--surface2);"
					onClick=${onScrollbarClick}
				>
					<div
						class="absolute w-full rounded-full"
						style="background: var(--muted); top: ${thumbTopPct}%; height: ${thumbPct}%; min-height: 20px;"
					/>
				</div>
			` : null}
		</div>
	</div>`;
}

function SessionHistory() {
	useEffect(() => {
		fetchHistory();
	}, []);

	var closed = sessionHistory.value.filter((s) => s.closed_at);
	if (closed.length === 0) return null;

	return html`<div class="mt-4">
		<div class="text-xs font-medium text-[var(--muted)] mb-2">History</div>
		<div class="flex flex-col gap-1">
			${closed.map((sess) => {
				var isSelected = selectedHistorySession.value === sess.session_id;
				return html`
					<div
						key=${sess.session_id}
						class="rounded border p-2 text-xs cursor-pointer transition-colors ${isSelected ? 'border-[var(--accent)] bg-[var(--accent)]/5' : 'border-[var(--border)] bg-[var(--surface)] hover:border-[var(--accent)]/50'}"
						onClick=${() => {
							// Deselect live session, show history
							activeSession.value = null;
							screencasting.value = false;
							frameData.value = null;
							selectedHistorySession.value = sess.session_id;
							fetchActionLog(sess.session_id);
						}}
					>
						<div class="flex items-center justify-between gap-2">
							<div class="flex-1 min-w-0">
								<div class="font-mono text-[var(--text-strong)] truncate">${sess.session_id}</div>
								<div class="text-[var(--muted)] truncate mt-0.5">${sess.url || "(no page loaded)"}</div>
							</div>
							<span class="text-[10px] px-1.5 py-0.5 rounded bg-[var(--surface2)] text-[var(--muted)] shrink-0">closed</span>
						</div>
						<div class="text-[var(--muted)] mt-1">${sess.created_at}</div>
					</div>
				`;
			})}
		</div>
	</div>`;
}

function ActionLogPanel() {
	var sid = selectedHistorySession.value;
	if (!sid) return null;

	var sess = sessionHistory.value.find((s) => s.session_id === sid);

	return html`<div class="flex-1 flex flex-col min-h-0 overflow-y-auto">
		<div class="flex items-center justify-between mb-2">
			<div>
				<div class="text-sm font-medium text-[var(--text-strong)]">Session Log</div>
				<div class="text-xs text-[var(--muted)] font-mono">${sid}</div>
				${sess ? html`<div class="text-xs text-[var(--muted)]">${sess.created_at} \u2014 ${sess.closed_at || "active"}</div>` : null}
			</div>
			<button
				class="provider-btn provider-btn-secondary provider-btn-sm"
				onClick=${() => { selectedHistorySession.value = null; }}
			>
				Back
			</button>
		</div>
		${actionLog.value.length === 0
			? html`<div class="text-xs text-[var(--muted)] p-3">No actions recorded for this session.</div>`
			: html`<div class="flex flex-col gap-1">
				${actionLog.value.map((entry) => html`
					<div key=${entry.id} class="rounded border border-[var(--border)] p-2 text-xs bg-[var(--surface)]">
						<div class="flex items-center justify-between gap-2">
							<span class="font-mono font-medium ${entry.success ? 'text-[var(--text-strong)]' : 'text-[var(--error)]'}">
								${entry.action}
							</span>
							<span class="text-[var(--muted)]">${entry.duration_ms}ms</span>
						</div>
						${entry.url ? html`<div class="text-[var(--muted)] truncate mt-0.5">${entry.url}</div>` : null}
						${entry.error ? html`<div class="text-[var(--error)] mt-0.5">${entry.error}</div>` : null}
						<div class="text-[var(--muted)] mt-0.5">${entry.created_at}</div>
					</div>
				`)}
			</div>`
		}
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
			Create browser sessions or view ones created by agents. Click a session to view and
			interact with it. Agents share the same cookies across all sessions.
		</div>

		<div class="flex flex-col lg:flex-row gap-4 flex-1 min-h-0">
			<div class="lg:w-80 shrink-0 overflow-y-auto">
				<${SessionList} />
				<${SessionHistory} />
			</div>
			<div class="flex-1 flex flex-col min-w-0">
				${selectedHistorySession.value
					? html`<${ActionLogPanel} />`
					: html`<fragment>
						<${NavigateBar} />
						<${BrowserCanvas} />
					</fragment>`
				}
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
	stopUrlPolling();
	if (containerEl) {
		render(null, containerEl);
		containerEl = null;
	}
	screencasting.value = false;
	activeSession.value = null;
	frameData.value = null;
	currentUrl.value = "";
}
