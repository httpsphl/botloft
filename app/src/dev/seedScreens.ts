// Dev only: a Designer drawing screens in the fake preview, so the design
// area has something live to show. The page arrives in pieces, as the
// daemon's drafts do while the model writes it.

import type { FakeBotloft } from "../lib/fake";
import type { CrewId } from "../lib/protocol.gen";

const LANDING = `<!doctype html>
<html><head><meta charset="utf-8"><title>Padaria Aurora</title>
<style>
body{margin:0;font-family:Georgia,serif;background:#fbf6ef;color:#3b2a1a}
header{display:flex;justify-content:space-between;align-items:center;padding:28px 64px}
.logo{font-size:26px;font-weight:bold;letter-spacing:.5px}
nav a{margin-left:28px;color:#3b2a1a;text-decoration:none;font-family:system-ui}
.hero{display:grid;grid-template-columns:1.1fr .9fr;gap:48px;padding:40px 64px 72px;align-items:center}
h1{font-size:64px;line-height:1.02;margin:0 0 20px}
.hero p{font:18px/1.6 system-ui;color:#6b5540;max-width:460px}
.cta{display:inline-block;margin-top:24px;background:#c2410c;color:#fff;padding:16px 28px;border-radius:999px;font:600 16px system-ui}
.bread{height:360px;border-radius:32px;background:radial-gradient(circle at 30% 30%,#f5c27a,#b45309 70%);box-shadow:0 30px 60px #b4530944}
.cards{display:grid;grid-template-columns:repeat(3,1fr);gap:24px;padding:0 64px 64px}
.card{background:#fff;border-radius:20px;padding:24px;box-shadow:0 8px 24px #3b2a1a14;font-family:system-ui}
.card b{display:block;font:bold 20px Georgia,serif;margin-bottom:6px}
</style></head>
<body>
<header><div class="logo">Padaria Aurora</div><nav><a href="#">Pães</a><a href="#">Doces</a><a href="#">Encomendas</a></nav></header>
<section class="hero"><div><h1>Pão quente, todo dia às 6h.</h1>
<p>Fermentação natural de 36 horas, farinha orgânica e um forno a lenha que não apaga desde 1987.</p>
<a class="cta" href="#">Ver o cardápio</a></div><div class="bread"></div></section>
<section class="cards"><div class="card"><b>Sourdough</b>Casca crocante, miolo aberto.</div>
<div class="card"><b>Croissant</b>81 camadas de manteiga francesa.</div>
<div class="card"><b>Bolo de fubá</b>A receita da vó, sem mudar nada.</div></section>
</body></html>`;

const APP = `<!doctype html><html><head><meta name="botloft-device" content="mobile">
<style>body{margin:0;font-family:system-ui;background:#111;color:#fff}
.top{padding:56px 24px 16px;font-size:28px;font-weight:700}
.item{margin:12px 16px;padding:18px;border-radius:18px;background:#1f1f1f;display:flex;justify-content:space-between}
.price{color:#f59e0b;font-weight:600}</style></head>
<body><div class="top">Encomendas</div>
<div class="item">2 Sourdough<span class="price">R$ 38</span></div>
<div class="item">6 Croissants<span class="price">R$ 54</span></div>
<div class="item">Bolo de fubá<span class="price">R$ 29</span></div></body></html>`;

/** A Designer that draws the landing page again and again. */
export function seedScreens(fake: FakeBotloft, crewId: CrewId): void {
  const designer = fake.addBot(crewId, "Designer", "Draws the bakery's screens");
  fake.setBotState(designer.id, "busy", 1);
  fake.screens.add(designer.id, "encomendas.html", APP, {
    folder: "site",
    device: "mobile",
    at: fake.now - 60_000,
  });
  const path = "C:\\Work\\site\\index.html";
  let written = 0;
  let resting = 0;
  setInterval(() => {
    if (resting > 0) {
      resting -= 1;
      if (resting === 0) {
        written = 0;
      }
      return;
    }
    written = Math.min(LANDING.length, written + 24);
    const done = written === LANDING.length;
    if (done) {
      fake.screens.add(designer.id, "index.html", LANDING, { folder: "site" });
      resting = 30;
    }
    fake.screens.draft(designer.id, path, LANDING.slice(0, written), done);
  }, 150);
}
