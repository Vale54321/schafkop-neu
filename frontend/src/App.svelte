<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { writable } from 'svelte/store';
  import SendBox from './SendBox.svelte';

  const ports = writable<Array<any>>([]);
  const log = writable<string[]>([]);
  const connected = writable(false);
  const portOpen = writable(false);
  let es: EventSource | null = null;

  function addLog(line: string) {
  // keep a fairly large history and prefix with an ISO timestamp
  log.update((l) => [new Date().toISOString() + '  ' + line, ...l].slice(0, 2000));
  }

  async function list() {
  console.log('[serial] list() called');
    const res = await fetch('/serial/ports');
    const data = await res.json();
  console.log('[serial] list() ->', data);
    ports.set(data);
  }

  async function openPort(path: string) {
  console.log('[serial] openPort()', path);
  const res = await fetch('/serial/open', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ path }) });
  const j = await res.json().catch(() => null);
  console.log('[serial] openPort result', j);
  // log result and refresh status
  if (j?.ok) addLog('PORT: open requested ' + path);
  else addLog('PORT: open request failed - ' + (j?.error ?? 'unknown'));
  setTimeout(getStatus, 300);
  }

  async function closePort() {
  console.log('[serial] closePort()');
  const res = await fetch('/serial/close', { method: 'POST' });
  const j = await res.json().catch(() => null);
  console.log('[serial] closePort result', j);
  addLog('PORT: close requested');
  }

  async function sendLine(txt: string) {
  console.log('[serial] sendLine()', txt);
  const res = await fetch('/serial/send', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ payload: txt }) });
  const j = await res.json().catch(() => null);
  console.log('[serial] sendLine result', j);
  if (!j?.ok) addLog('TX error: ' + (j?.error ?? 'unknown'));
  }

  async function getStatus() {
    console.log('[serial] getStatus()');
    try {
      const res = await fetch('/serial/status');
      const j = await res.json();
      console.log('[serial] getStatus ->', j);
      portOpen.set(!!j.open);
    } catch (e) {
      console.warn('[serial] getStatus failed', e);
      portOpen.set(false);
    }
  }

  function startStream() {
    if (es) return;
    console.log('[serial] startStream()');
    es = new EventSource('/serial/stream');
    // connection lifecycle events - also log them so the serial box shows open/close/errors
    es.addEventListener('open', () => { console.log('[serial][SSE] open'); connected.set(true); addLog('[SSE] connected'); });
    es.addEventListener('close', () => { console.log('[serial][SSE] close'); connected.set(false); addLog('[SSE] disconnected'); });
    es.addEventListener('error', (e: any) => { console.warn('[serial][SSE] error', e); addLog('[SSE] error: ' + (e?.message ?? JSON.stringify(e))); });

    // data events contain the actual lines emitted by the Pico; parse JSON payloads but fall back to raw
    es.addEventListener('data', (ev: any) => {
      console.log('[serial][SSE] data event', ev.data);
      try {
        const d = JSON.parse(ev.data);
        // always show the raw line emitted by the firmware
        addLog('RX: ' + d.line);
      } catch (e) {
        addLog('RX (raw): ' + ev.data);
      }
    });
  }

  function stopStream() {
  console.log('[serial] stopStream()');
  es?.close();
  es = null;
  connected.set(false);
  }

  let statusInterval: any;
  // quick test command defaults
  let stepVal: string = '4096';
  let dirVal: string = '1';
  let speedVal: string = '3000';
  onMount(() => {
    list();
    startStream();
    statusInterval = setInterval(getStatus, 1000);
    getStatus();
  });
  onDestroy(() => {
    stopStream();
    clearInterval(statusInterval);
  });

  function sendStepCmd() {
    const steps = Number(stepVal);
    const dir = Number(dirVal) === 1 ? 1 : 0;
    if (!Number.isFinite(steps) || steps <= 0) {
      addLog('TX error: invalid STEP value: ' + stepVal);
      return;
    }
    const line = `STEP ${steps} ${dir}`;
    sendLine(line);
    addLog('TX: ' + line);
  }

  function sendSpeedCmd() {
    const d = Number(speedVal);
    if (!Number.isFinite(d) || d <= 0) {
      addLog('TX error: invalid SPEED value: ' + speedVal);
      return;
    }
    const line = `SPEED ${d}`;
    sendLine(line);
    addLog('TX: ' + line);
  }
</script>

<main class="min-h-screen bg-gradient-to-b from-bg to-surface text-gray-100 p-8">
    <h1 class="text-3xl font-bold underline">
    Hello world!
  </h1>
  <div class="container mx-auto">
    <header class="flex items-center justify-between mb-6">
      <div>
        <h1 class="text-2xl font-semibold">Raspberry Pico</h1>
        <p class="text-sm text-gray-400">Serial monitor & command tester</p>
      </div>
      <div class="flex items-center gap-3">
        <span class="text-sm text-gray-300">SSE: {$connected ? 'connected' : 'disconnected'}</span>
        <button class="px-3 py-1 bg-primary text-white rounded-md text-sm" on:click={startStream}>Start</button>
        <button class="px-3 py-1 bg-gray-700 text-white rounded-md text-sm" on:click={stopStream}>Stop</button>
      </div>
    </header>

    <div class="grid grid-cols-12 gap-6">
      <aside class="col-span-4 bg-gray-900/60 p-4 rounded-lg border border-gray-800">
        <section class="mb-4">
          <h3 class="text-sm text-gray-300 font-medium">Available ports</h3>
          <div class="flex items-center gap-2 mt-2">
            <button class="px-2 py-1 text-sm bg-gray-700 rounded" on:click={list}>Refresh</button>
            <button class="px-2 py-1 text-sm bg-red-600 rounded" on:click={closePort}>Close port</button>
          </div>
          <ul class="mt-3 space-y-2">
            {#each $ports as p}
              <li class="flex items-center justify-between bg-gray-800/40 p-2 rounded">
                <code class="text-xs text-gray-200">{p.path}</code>
                <button class="px-2 py-1 text-sm bg-primary rounded" on:click={() => openPort(p.path)}>Open</button>
              </li>
            {/each}
          </ul>
        </section>

        <section class="mb-4">
          <h3 class="text-sm text-gray-300 font-medium">Send</h3>
          <div class="mt-2 flex items-center gap-2">
            <div class="text-xs text-gray-300">Port: <span class="font-medium">{$portOpen ? 'open' : 'closed'}</span></div>
          </div>
          <div class="mt-2">
            <SendBox disabled={!$portOpen} on:send={(e) => { sendLine(e.detail); addLog('TX: ' + e.detail); }} />
          </div>
        </section>

        <section>
          <h3 class="text-sm text-gray-300 font-medium">Quick commands</h3>
          <div class="mt-2 space-y-2">
            <div class="flex gap-2 items-center">
              <input class="w-28 p-2 rounded bg-gray-800 text-sm" type="number" bind:value={stepVal} min="1" />
              <select class="p-2 rounded bg-gray-800 text-sm" bind:value={dirVal}>
                <option value="1">1 (forward)</option>
                <option value="0">0 (reverse)</option>
              </select>
              <button class="px-3 py-1 bg-primary text-white rounded" on:click={sendStepCmd} disabled={!$portOpen}>Send STEP</button>
            </div>

            <div class="flex gap-2 items-center">
              <input class="w-28 p-2 rounded bg-gray-800 text-sm" type="number" bind:value={speedVal} min="1" />
              <button class="px-3 py-1 bg-primary text-white rounded" on:click={sendSpeedCmd} disabled={!$portOpen}>Send SPEED</button>
            </div>
          </div>
        </section>
      </aside>

      <section class="col-span-8 bg-gray-900/60 p-4 rounded-lg border border-gray-800 flex flex-col">
        <div class="flex items-center justify-between mb-3">
          <h3 class="text-sm text-gray-300 font-medium">Live log</h3>
          <div class="text-xs text-gray-400">Showing last {2000} lines</div>
        </div>
        <div class="flex-1 overflow-auto bg-[#0b0b0b] p-3 rounded text-sm font-mono text-gray-100">
          {#each $log as l}
            <div class="py-0.5"><code class="text-xs">{l}</code></div>
          {/each}
        </div>
      </section>
    </div>
  </div>
</main>

<style>
  main { padding: 1rem; font-family: system-ui, -apple-system, 'Segoe UI', Roboto; color:#111 }
  h1 { margin-bottom:0.5rem }
  section { margin-top:1rem; }
  button { margin-left:0.5rem }
</style>



