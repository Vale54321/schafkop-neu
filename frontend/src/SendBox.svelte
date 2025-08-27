<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  export let placeholder = 'Type a line to send';
  export let disabled: boolean = false;
  const dispatch = createEventDispatcher();
  let txt = '';
  function doSend() {
    if (!txt) return;
  console.log('[SendBox] dispatch send', txt);
  dispatch('send', txt);
    txt = '';
  }
</script>

  <div style="margin-top:0.5rem; display:flex; gap:0.5rem; align-items:center;">
  <input bind:value={txt} placeholder={placeholder} on:keydown={(e) => e.key === 'Enter' && doSend()} style="flex:1; padding:0.4rem;" disabled={disabled} />
  <button on:click={doSend} disabled={disabled}>Send</button>
</div>
