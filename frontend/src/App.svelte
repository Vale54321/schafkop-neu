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
    log.update((l) => [new Date().toISOString() + '  ' + line, ...l].slice(0, 200));
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
  // refresh status
  setTimeout(getStatus, 300);
  }

  async function closePort() {
  console.log('[serial] closePort()');
  const res = await fetch('/serial/close', { method: 'POST' });
  console.log('[serial] closePort result', await res.json().catch(() => null));
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
    es.addEventListener('open', () => { console.log('[serial][SSE] open'); connected.set(true); });
    es.addEventListener('close', () => { console.log('[serial][SSE] close'); connected.set(false); });
    es.addEventListener('error', (e: any) => { console.warn('[serial][SSE] error', e); addLog('SSE error'); });
    es.addEventListener('data', (ev: any) => {
      console.log('[serial][SSE] data event', ev.data);
      try {
        const d = JSON.parse(ev.data);
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
</script>

<main>
  <h1>Raspberry Pico — Serial Monitor</h1>

  <section>
    <h3>Available ports</h3>
    <button on:click={list}>Refresh</button>
    <ul>
      {#each $ports as p}
        <li>
          <code>{p.path}</code>
          <button on:click={() => openPort(p.path)}>Open</button>
        </li>
      {/each}
    </ul>
    <button on:click={closePort}>Close port</button>
  </section>

  <section>
    <h3>Send</h3>
    <div style="display:flex; align-items:center; gap:1rem;">
  <div>Port: {$portOpen ? 'open' : 'closed'}</div>
  <SendBox disabled={!$portOpen} on:send={(e) => { sendLine(e.detail); addLog('TX: ' + e.detail); }} />
    </div>
  </section>

  <section>
    <h3>Live log ({$connected ? 'connected' : 'disconnected'})</h3>
    <button on:click={startStream}>Start stream</button>
    <button on:click={stopStream}>Stop stream</button>
    <div style="height:320px; overflow:auto; background:#111; color:#dfe6ef; padding:8px; border-radius:6px;">
      {#each $log as l}
        <div><code>{l}</code></div>
      {/each}
    </div>
  </section>
</main>

<style>
  main { padding: 1rem; font-family: system-ui, -apple-system, 'Segoe UI', Roboto; color:#111 }
  h1 { margin-bottom:0.5rem }
  section { margin-top:1rem; }
  button { margin-left:0.5rem }
</style>



