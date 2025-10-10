const btn = document.getElementById('pingBtn');
const versionBtn = document.getElementById('versionBtn');
const out = document.getElementById('result');

btn?.addEventListener('click', async () => {
  out.textContent = 'Requesting /api/ping…';
  try {
    const res = await fetch('/api/ping');
    const json = await res.json();
    out.textContent = JSON.stringify(json, null, 2);
  } catch (err) {
    out.textContent = 'Error: ' + (err?.message || String(err));
  }
});

versionBtn?.addEventListener('click', async () => {
  out.textContent = 'Requesting /api/version…';
  try {
    const res = await fetch('/api/version');
    const json = await res.json();
    out.textContent = JSON.stringify(json, null, 2);
  } catch (err) {
    out.textContent = 'Error: ' + (err?.message || String(err));
  }
});
