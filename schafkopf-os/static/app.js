const btn = document.getElementById('pingBtn');
const out = document.getElementById('result');

btn?.addEventListener('click', async () => {
  out.textContent = 'Requesting /api/ping…';
  try {
    const res = await fetch('/api');
    const json = await res.json();
    out.textContent = JSON.stringify(json, null, 2);
  } catch (err) {
    out.textContent = 'Error: ' + (err?.message || String(err));
  }
});
