<script lang="ts">
  // Landing page for the Schafkopf Bot.
</script>

<main id="app">
  <header class="nav">
    <div class="container">
      <h1 class="brand">Schafkopf Bot</h1>
      <nav>
        <a href="#features">Features</a>
        <a href="#how">How it works</a>
        <a href="#contact">Contact</a>
        <a class="cta" href="#try">Try</a>
      </nav>
    </div>
  </header>

  <section class="hero container">
    <div class="hero-content">
      <h2>Play, train and analyse Schafkopf with an intelligent bot</h2>
      <p class="tagline">A friendly opponent and coach for solo practice, teaching games and building your skills.</p>
      <div class="actions">
        <a class="btn primary" id="try" href="mailto:hello@schafkopf.bot?subject=I%20want%20to%20try%20the%20bot">Play with the bot</a>
        <a class="btn" href="#features">See features</a>
      </div>
    </div>

    <div class="hero-visual" aria-hidden="true">
      <svg width="360" height="220" viewBox="0 0 360 220" fill="none" xmlns="http://www.w3.org/2000/svg" role="img">
        <rect x="0" y="0" width="360" height="220" rx="12" fill="var(--card)" />
        <g transform="translate(24,28)">
          <rect width="80" height="120" rx="8" fill="#fff" opacity="0.95" />
          <rect x="90" width="80" height="120" rx="8" fill="#fff" opacity="0.95" />
          <rect x="180" width="80" height="120" rx="8" fill="#fff" opacity="0.95" />
        </g>
        <text x="24" y="190" fill="var(--muted)" font-size="12">Schafkopf Bot — practice anytime</text>
      </svg>
    </div>
  </section>

  <section id="features" class="features container">
    <h3>Key features</h3>
    <div class="grid">
      <article class="card">
        <h4>Play against an AI</h4>
        <p>Fast opponents at multiple skill levels — perfect for solo training and testing strategies.</p>
      </article>
      <article class="card">
        <h4>Hand analysis</h4>
        <p>Review played hands with suggestions and common mistakes highlighted to accelerate learning.</p>
      </article>
      <article class="card">
        <h4>Compatible interfaces</h4>
        <p>Connect via chat platforms or a web interface — invite the bot to your games and tournaments.</p>
      </article>
      <article class="card">
        <h4>Privacy-first</h4>
        <p>Local game history and optional anonymous usage — designed with player privacy in mind.</p>
      </article>
    </div>
  </section>

  <section id="how" class="how container">
    <h3>How it works</h3>
    <ol>
      <li>Start a new game and choose a difficulty.</li>
      <li>Play hands while the bot offers optional hints and post-game analysis.</li>
      <li>Export or review hands to track progress.</li>
    </ol>
  </section>

  <section id="contact" class="contact container">
    <h3>Get early access</h3>
    <p>Interested in trying the bot or integrating it into a platform? Send a message and we'll reply with next steps.</p>
    <a class="btn primary" href="mailto:hello@schafkopf.bot?subject=Early%20access%20request">Contact us</a>
  </section>

  <footer class="site-footer">
    <div class="container">
      <small>© {new Date().getFullYear()} Schafkopf Bot — built with Svelte</small>
      <nav>
        <a href="https://github.com/Vale54321/schafkop-neu" target="_blank" rel="noreferrer">Source</a>
      </nav>
    </div>
  </footer>
</main>

<style>
  /* keep styles local to App for the landing layout */
  :global(:root) {
    --bg: #121212;
    --text: #eaeaea;
    --muted: #9aa3b2;
    --card: #1a1a1a;
    --accent: #7c6cff;
    --glass: rgba(255,255,255,0.04);
  }

  main {
    color: var(--text);
    background: linear-gradient(180deg, #0d0d0d 0%, #121212 100%);
    min-height: 100vh;
    display: flex;
    flex-direction: column;
  }

  .container {
    max-width: 1100px;
    margin: 0 auto;
    padding: 2rem;
  }

  .nav {
    border-bottom: 1px solid rgba(255,255,255,0.04);
    backdrop-filter: blur(6px);
  }
  .nav .container {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .brand { font-size: 1.2rem; font-weight: 700; margin: 0; }
  nav a { color: var(--muted); margin-left: 1rem; text-decoration: none; }
  nav a.cta { color: var(--text); background: var(--accent); padding: 0.5rem 0.9rem; border-radius: 8px; }

  .hero {
    display: grid;
    grid-template-columns: 1fr 420px;
    gap: 2rem;
    align-items: center;
    padding-top: 3rem;
    padding-bottom: 3rem;
  }
  .hero h2 { font-size: 2.1rem; margin: 0 0 0.5rem 0; }
  .tagline { color: var(--muted); margin-bottom: 1rem; }
  .actions { display:flex; gap:0.75rem; }
  .btn { display:inline-block; padding:0.6rem 1rem; border-radius:8px; text-decoration:none; color:var(--text); background:var(--glass); border:1px solid rgba(255,255,255,0.02); }
  .btn.primary { background: var(--accent); color: #fff; }

  .hero-visual { display:flex; justify-content:center; }
  .features { padding-top: 2rem; padding-bottom:2rem; }
  .features h3 { margin-bottom:1rem; }
  .grid { display:grid; grid-template-columns:repeat(auto-fit,minmax(220px,1fr)); gap:1rem; }
  .card { background: var(--card); padding:1rem; border-radius:12px; box-shadow: 0 6px 20px rgba(0,0,0,0.4); }

  .how { background: linear-gradient(180deg, transparent, rgba(255,255,255,0.01)); padding:2rem; border-radius:12px; margin-bottom:1rem; }
  ol { padding-left:1.2rem; color:var(--muted); }

  .contact { text-align:center; padding:2rem 0; }

  .site-footer { border-top:1px solid rgba(255,255,255,0.04); margin-top:auto; }
  .site-footer .container { display:flex; justify-content:space-between; align-items:center; padding:1rem 2rem; }
  .site-footer small { color:var(--muted); }

  @media (max-width: 880px) {
    .hero { grid-template-columns: 1fr; }
    .site-footer .container { flex-direction:column; gap:0.75rem; }
  }
</style>
