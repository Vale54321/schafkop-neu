# schafkopf-os — Test Frontend (Axum + Askama)

This crate serves a tiny test frontend using Axum + Askama and static assets.

## Run

```bash
cargo run --manifest-path schafkopf-os/Cargo.toml
```

Then open:

- http://127.0.0.1:3000 — server renders an Askama template
- Click "Ping API" to call `GET /api/ping`

Or test the API directly:

```bash
curl -s http://127.0.0.1:3000/api/ping | jq
```

## Structure

- `src/main.rs` — Axum server with routes `/` and `/api/ping`, and static files at `/static`
- `templates/index.html` — Askama template for the homepage
- `static/styles.css` — Minimal styling
- `static/app.js` — Browser script to call the ping API
