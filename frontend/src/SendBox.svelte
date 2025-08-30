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

  <div class="mt-2 flex gap-2 items-center">
    <input class="flex-1 p-2 rounded bg-gray-800 text-sm text-gray-100" bind:value={txt} placeholder={placeholder} on:keydown={(e) => e.key === 'Enter' && doSend()} disabled={disabled} />
    <button class="px-3 py-1 bg-primary text-white rounded text-sm" on:click={doSend} disabled={disabled}>Send</button>
  </div>
