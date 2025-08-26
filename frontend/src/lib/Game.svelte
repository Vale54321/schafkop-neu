<script lang="ts">
  // Simple game starter UI that posts a start command to the backend.
  import { writable } from 'svelte/store';

  type Seat = 'North' | 'East' | 'South' | 'West';

  let robotSeat: Seat = 'South';
  let difficulty: 'easy' | 'medium' | 'hard' = 'medium';
  let includeRobot = true;
  let isStarting = false;
  const logs = writable<string[]>([]);

  function appendLog(line: string) {
    logs.update((ls) => {
      ls.push(line);
      return ls.slice(-50);
    });
  }

  async function startGame() {
    isStarting = true;
    appendLog(`Requesting new game (robotSeat=${robotSeat}, difficulty=${difficulty}, includeRobot=${includeRobot})`);

    try {
      const resp = await fetch('/api/game/start', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ robotSeat, difficulty, includeRobot }),
      });

      if (!resp.ok) {
        const text = await resp.text();
        throw new Error(text || resp.statusText);
      }

      const data = await resp.json();
      appendLog('Game started: ' + JSON.stringify(data));
    } catch (err) {
      appendLog('Error starting game: ' + String(err));
    } finally {
      isStarting = false;
    }
  }
</script>

<section class="game-starter card">
  <h4>Start a new physical game</h4>
  <p class="muted">The robot will join as one of the four players and physically plays cards on the table.</p>

  <div class="form-row">
    <label>Robot seat</label>
    <select bind:value={robotSeat} aria-label="Robot seat">
      <option value="North">North</option>
      <option value="East">East</option>
      <option value="South">South</option>
      <option value="West">West</option>
    </select>
  </div>

  <div class="form-row">
    <label>Difficulty</label>
    <select bind:value={difficulty} aria-label="Difficulty">
      <option value="easy">Easy</option>
      <option value="medium">Medium</option>
      <option value="hard">Hard</option>
    </select>
  </div>

  <div class="form-row-inline">
    <label><input type="checkbox" bind:checked={includeRobot} /> Include robot in this game</label>
  </div>

  <div class="actions">
    <button class="btn primary" on:click={startGame} disabled={isStarting}> {isStarting ? 'Starting…' : 'Start game'} </button>
  </div>

  <div class="log">
    <h5>Activity</h5>
    <ul>
      {#each $logs.slice().reverse() as line}
        <li>{line}</li>
      {/each}
    </ul>
  </div>
</section>

<style>
  .game-starter { padding: 1rem; border-radius: 10px; }
  .muted { color: var(--muted); margin-top: 0.25rem; }
  .form-row { display:flex; flex-direction:column; gap:0.35rem; margin-top:0.8rem; }
  .form-row-inline { margin-top:0.8rem; }
  select { padding:0.5rem; border-radius:8px; background:transparent; color:inherit; border:1px solid rgba(255,255,255,0.04); }
  .actions { margin-top:1rem; }
  .log { margin-top:1rem; max-height:160px; overflow:auto; font-size:0.9rem; color:var(--muted); }
  .log ul { list-style:none; padding:0; margin:0; display:flex; flex-direction:column; gap:0.25rem; }
</style>
