<script setup lang="ts">
import { useI18n } from "vue-i18n";
import CtaLink from "../components/ui/CtaLink.vue";
import { SAMPLE_HERO } from "../samples";

const { t } = useI18n();

const surfaceItems = [
    { key: "namespace", mark: "NS", tone: "orange" },
    { key: "table", mark: "TB", tone: "cyan" },
    { key: "service", mark: "SV", tone: "violet" },
] as const;

const lifecycleItems = ["define", "connect", "validate", "evolve"] as const;
</script>

<template>
  <div class="vos-home">
    <section class="hero home-rail">
      <div class="hero-copy">
        <p class="kicker"><span class="pulse"></span>{{ t("home.heroKicker") }}</p>
        <h1>
          {{ t("home.heroTitleA") }}<br />
          <span>{{ t("home.heroTitleB") }}</span>
        </h1>
        <p class="hero-lede">{{ t("home.heroLede") }}</p>
        <div class="hero-actions">
          <CtaLink to="/playground">{{ t("home.openPlayground") }}<span class="arrow">↗</span></CtaLink>
          <CtaLink variant="ghost" to="/model">{{ t("home.readModel") }}</CtaLink>
        </div>
        <div class="hero-meta">
          <span class="meta-key">VOS / 0.1</span>
          <span>{{ t("home.heroMeta") }}</span>
        </div>
      </div>

      <div class="model-board" aria-label="VOS model relationship preview">
        <div class="board-topline">
          <span><i class="window-dot orange"></i><i class="window-dot cyan"></i><i class="window-dot violet"></i></span>
          <span class="board-file">identity.vos</span>
          <span class="board-version">SCHEMA / V2</span>
        </div>
        <div class="board-body">
          <div class="board-label">{{ t("home.boardLabel") }}</div>
          <div class="map">
            <span class="connector connector-main"></span>
            <span class="connector connector-left"></span>
            <span class="connector connector-right"></span>
            <span class="connector connector-bottom"></span>
            <div class="map-node node-root">
              <span class="node-type">TABLE</span>
              <strong>User</strong>
              <small>identity · durable</small>
            </div>
            <div class="map-node node-profile">
              <span class="node-type">CLASS</span>
              <strong>Profile</strong>
              <small>inline value</small>
            </div>
            <div class="map-node node-session">
              <span class="node-type">TABLE</span>
              <strong>Session</strong>
              <small>reference</small>
            </div>
            <div class="map-node node-service">
              <span class="node-type">SERVICE</span>
              <strong>Identity</strong>
              <small>named params</small>
            </div>
          </div>
          <div class="board-footer">
            <span><b class="legend-dot cyan"></b>{{ t("home.boardRelation") }}</span>
            <span><b class="legend-dot orange"></b>{{ t("home.boardPersistent") }}</span>
            <span class="board-status">✓ {{ t("home.boardReady") }}</span>
          </div>
        </div>
      </div>
    </section>

    <section class="system-strip">
      <div class="home-rail strip-inner">
        <p class="strip-lede">{{ t("home.stripTitle") }}</p>
        <div class="strip-stats">
          <span><b>01</b>{{ t("home.stripNamespace") }}</span>
          <span><b>02</b>{{ t("home.stripRelations") }}</span>
          <span><b>03</b>{{ t("home.stripLifecycle") }}</span>
        </div>
      </div>
    </section>

    <section class="anatomy home-rail section-block">
      <div class="section-intro">
        <p class="eyebrow">{{ t("home.anatomyEyebrow") }}</p>
        <h2>{{ t("home.anatomyTitle") }}</h2>
        <p>{{ t("home.anatomyLede") }}</p>
        <div class="intro-note"><span>→</span>{{ t("home.anatomyNote") }}</div>
      </div>
      <div class="schema-card">
        <div class="schema-card-head">
          <span class="mono">model.vos</span>
          <span class="schema-chip">{{ t("home.schemaChip") }}</span>
        </div>
        <pre aria-label="VOS schema example"><code>{{ SAMPLE_HERO }}</code></pre>
        <div class="schema-card-foot">
          <span><b class="legend-dot orange"></b>table</span>
          <span><b class="legend-dot cyan"></b>reference</span>
          <span><b class="legend-dot violet"></b>namespace</span>
        </div>
      </div>
    </section>

    <section class="surfaces home-rail section-block">
      <div class="section-intro wide-intro">
        <p class="eyebrow">{{ t("home.surfaceEyebrow") }}</p>
        <h2>{{ t("home.surfaceTitle") }}</h2>
        <p>{{ t("home.surfaceLede") }}</p>
      </div>
      <div class="surface-grid">
        <article v-for="item in surfaceItems" :key="item.key" class="surface-card" :data-tone="item.tone">
          <div class="surface-card-top"><span class="surface-mark">{{ item.mark }}</span><span class="surface-index">0{{ surfaceItems.indexOf(item) + 1 }}</span></div>
          <h3>{{ t(`home.surfaces.${item.key}Title`) }}</h3>
          <p>{{ t(`home.surfaces.${item.key}Body`) }}</p>
          <code>{{ t(`home.surfaces.${item.key}Code`) }}</code>
        </article>
      </div>
    </section>

    <section class="lifecycle home-rail section-block">
      <div class="lifecycle-head">
        <div class="section-intro">
          <p class="eyebrow">{{ t("home.lifecycleEyebrow") }}</p>
          <h2>{{ t("home.lifecycleTitle") }}</h2>
        </div>
        <p class="lifecycle-note">{{ t("home.lifecycleLede") }}</p>
      </div>
      <ol class="lifecycle-track">
        <li v-for="(step, index) in lifecycleItems" :key="step">
          <span class="step-number">0{{ index + 1 }}</span>
          <span class="step-line"></span>
          <strong>{{ t(`home.lifecycle.${step}Title`) }}</strong>
          <p>{{ t(`home.lifecycle.${step}Body`) }}</p>
        </li>
      </ol>
    </section>

    <section class="toolchain home-rail section-block">
      <div class="toolchain-copy">
        <p class="eyebrow">{{ t("home.toolchainEyebrow") }}</p>
        <h2>{{ t("home.toolchainTitle") }}</h2>
        <p>{{ t("home.toolchainLede") }}</p>
      </div>
      <div class="package-stack">
        <article>
          <div><span class="package-icon">TS</span><code>@game-gpt/vos</code></div>
          <p>{{ t("home.pkgVos") }}</p>
          <span class="package-role">TYPE CHECKS · RUNTIME</span>
        </article>
        <article>
          <div><span class="package-icon rust">Rs</span><code>vos</code></div>
          <p>{{ t("home.pkgRs") }}</p>
          <span class="package-role">AST · NATIVE</span>
        </article>
      </div>
    </section>

    <section class="final-cta home-rail">
      <div>
        <p class="eyebrow">{{ t("home.finalEyebrow") }}</p>
        <h2>{{ t("home.finalTitle") }}</h2>
        <p>{{ t("home.finalLede") }}</p>
      </div>
      <CtaLink to="/playground">{{ t("home.finalCta") }}<span class="arrow">↗</span></CtaLink>
    </section>
  </div>
</template>

<style scoped>
.vos-home {
  overflow: hidden;
  background: var(--page);
}

.home-rail {
  width: min(1240px, calc(100% - 3rem));
  margin-inline: auto;
}

.hero {
  display: grid;
  grid-template-columns: minmax(0, 0.88fr) minmax(32rem, 1.12fr);
  gap: clamp(2rem, 5vw, 5.5rem);
  align-items: center;
  min-height: min(720px, calc(100vh - 4.2rem));
  padding-block: clamp(3.5rem, 8vw, 7rem);
}

.hero-copy {
  max-width: 38rem;
}

.kicker,
.eyebrow,
.meta-key,
.board-label,
.board-version,
.surface-index,
.package-role {
  margin: 0;
  font-family: var(--font-mono);
  font-size: 0.72rem;
  letter-spacing: 0.11em;
  text-transform: uppercase;
}

.kicker {
  display: flex;
  align-items: center;
  gap: 0.55rem;
  color: var(--accent);
}

.pulse {
  width: 0.5rem;
  height: 0.5rem;
  border-radius: 999px;
  background: var(--accent);
  box-shadow: 0 0 0 0.25rem rgba(255, 139, 77, 0.15);
}

h1 {
  max-width: 35rem;
  margin: 1.25rem 0 1.4rem;
  font-size: clamp(3.2rem, 6.5vw, 6.5rem);
  font-weight: 700;
  letter-spacing: -0.055em;
  line-height: 0.98;
  color: var(--ink);
}

h1 span {
  color: var(--accent);
}

.hero-lede {
  max-width: 32rem;
  margin: 0;
  color: var(--muted);
  font-size: clamp(1rem, 1.35vw, 1.18rem);
  line-height: 1.7;
}

.hero-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 0.65rem;
  margin-top: 2rem;
}

.arrow {
  margin-left: 0.45rem;
  font-size: 1.05em;
}

.hero-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 0.8rem 1.1rem;
  align-items: center;
  margin-top: 2.4rem;
  color: var(--muted);
  font-size: 0.78rem;
}

.meta-key {
  color: var(--cyan);
}

.model-board {
  position: relative;
  min-height: 31rem;
  border: 1px solid var(--border);
  background: var(--surface);
  box-shadow: 0 1.5rem 4rem rgba(3, 7, 14, 0.22);
}

.board-topline,
.board-footer,
.schema-card-head,
.schema-card-foot {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
}

.board-topline {
  min-height: 2.75rem;
  padding: 0 1rem;
  border-bottom: 1px solid var(--border);
  background: var(--surface-2);
  color: var(--muted);
  font-family: var(--font-mono);
  font-size: 0.72rem;
}

.window-dot {
  display: inline-block;
  width: 0.48rem;
  height: 0.48rem;
  margin-right: 0.32rem;
  border-radius: 999px;
  background: var(--muted);
}

.window-dot.orange { background: var(--accent); }
.window-dot.cyan { background: var(--cyan); }
.window-dot.violet { background: var(--violet); }

.board-file { color: var(--ink); }
.board-version { color: var(--muted); font-size: 0.64rem; }

.board-body {
  padding: 1.25rem 1.3rem 1rem;
}

.board-label {
  color: var(--muted);
  font-size: 0.66rem;
}

.map {
  position: relative;
  min-height: 22.5rem;
  margin-top: 0.8rem;
}

.map-node {
  position: absolute;
  z-index: 2;
  display: grid;
  gap: 0.18rem;
  width: 9.8rem;
  padding: 0.8rem 0.9rem 0.85rem;
  border: 1px solid var(--border-strong);
  background: var(--surface);
  box-shadow: 0 0.7rem 1.5rem rgba(3, 7, 14, 0.12);
}

.map-node strong { color: var(--ink); font-size: 1.03rem; font-weight: 650; }
.map-node small { color: var(--muted); font-size: 0.72rem; }
.node-type { color: var(--accent); font-family: var(--font-mono); font-size: 0.62rem; letter-spacing: 0.1em; }
.node-root { top: 6.2rem; left: 50%; transform: translateX(-50%); border-color: rgba(255, 139, 77, 0.68); }
.node-profile { top: 1.7rem; left: 0.8rem; }
.node-session { top: 1.7rem; right: 0.8rem; border-color: rgba(89, 217, 221, 0.62); }
.node-service { bottom: 0.8rem; left: 50%; transform: translateX(-50%); border-color: rgba(156, 140, 255, 0.62); }

.connector { position: absolute; z-index: 1; display: block; background: var(--cyan); opacity: 0.68; }
.connector-main { top: 5.4rem; left: 50%; width: 1px; height: 1rem; }
.connector-left { top: 4.9rem; left: 25%; width: 25%; height: 1px; transform: rotate(18deg); transform-origin: right; }
.connector-right { top: 4.9rem; right: 25%; width: 25%; height: 1px; transform: rotate(-18deg); transform-origin: left; }
.connector-bottom { top: 15rem; left: 50%; width: 1px; height: 6rem; }

.board-footer {
  flex-wrap: wrap;
  justify-content: flex-start;
  padding-top: 0.75rem;
  border-top: 1px solid var(--border);
  color: var(--muted);
  font-family: var(--font-mono);
  font-size: 0.68rem;
}

.board-footer > span { display: inline-flex; align-items: center; gap: 0.35rem; }
.board-status { margin-left: auto; color: var(--ok); }
.legend-dot { display: inline-block; width: 0.42rem; height: 0.42rem; border-radius: 999px; background: var(--muted); }
.legend-dot.orange { background: var(--accent); }
.legend-dot.cyan { background: var(--cyan); }
.legend-dot.violet { background: var(--violet); }

.system-strip {
  border-block: 1px solid var(--border);
  background: var(--surface-2);
}

.strip-inner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 2rem;
  padding-block: 1.05rem;
}

.strip-lede { max-width: 25rem; margin: 0; color: var(--ink); font-size: 0.94rem; font-weight: 600; }
.strip-stats { display: flex; gap: clamp(1rem, 4vw, 3.25rem); color: var(--muted); font-family: var(--font-mono); font-size: 0.7rem; text-transform: uppercase; letter-spacing: 0.06em; }
.strip-stats span { display: inline-flex; align-items: center; gap: 0.5rem; white-space: nowrap; }
.strip-stats b { color: var(--accent); font-weight: 500; }

.section-block { padding-block: clamp(4rem, 8vw, 7rem); }
.section-intro { max-width: 31rem; }
.wide-intro { max-width: 42rem; }
.eyebrow { margin-bottom: 0.7rem; color: var(--accent); }
.section-intro h2,
.toolchain h2,
.final-cta h2 { margin: 0; color: var(--ink); font-size: clamp(2rem, 4vw, 3.8rem); font-weight: 680; letter-spacing: -0.045em; line-height: 1.03; }
.section-intro > p:not(.eyebrow), .toolchain-copy > p:not(.eyebrow), .final-cta p { margin: 1rem 0 0; color: var(--muted); font-size: 1rem; line-height: 1.7; }

.anatomy { display: grid; grid-template-columns: minmax(0, 0.8fr) minmax(0, 1.2fr); gap: clamp(2.5rem, 8vw, 8rem); align-items: center; border-bottom: 1px solid var(--border); }
.intro-note { display: flex; gap: 0.55rem; margin-top: 1.8rem; color: var(--cyan); font-family: var(--font-mono); font-size: 0.74rem; line-height: 1.5; }
.intro-note span { color: var(--accent); font-size: 1rem; }
.schema-card { overflow: hidden; border: 1px solid var(--border); background: var(--surface); box-shadow: 0 1.2rem 3rem rgba(3, 7, 14, 0.16); }
.schema-card-head { padding: 0.8rem 1rem; border-bottom: 1px solid var(--border); background: var(--surface-2); }
.mono { color: var(--ink); font-family: var(--font-mono); font-size: 0.76rem; }
.schema-chip { padding: 0.2rem 0.45rem; color: var(--cyan); border: 1px solid rgba(89, 217, 221, 0.38); font-family: var(--font-mono); font-size: 0.62rem; letter-spacing: 0.06em; text-transform: uppercase; }
.schema-card pre { min-height: 15rem; margin: 0; padding: 1.5rem 1.4rem; overflow: auto; color: var(--ink); background: linear-gradient(90deg, rgba(89, 217, 221, 0.06), transparent 48%), var(--surface); font-family: var(--font-mono); font-size: clamp(0.78rem, 1.15vw, 0.92rem); line-height: 1.75; }
.schema-card code { font: inherit; }
.schema-card-foot { justify-content: flex-start; flex-wrap: wrap; padding: 0.7rem 1rem; border-top: 1px solid var(--border); color: var(--muted); font-family: var(--font-mono); font-size: 0.66rem; text-transform: uppercase; }
.schema-card-foot span { display: inline-flex; align-items: center; gap: 0.35rem; }

.surfaces { border-bottom: 1px solid var(--border); }
.surface-grid { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 0.8rem; margin-top: 2.2rem; }
.surface-card { min-height: 17.5rem; padding: 1.25rem; border: 1px solid var(--border); background: var(--surface); transition: transform 180ms ease, border-color 180ms ease; }
.surface-card:hover { transform: translateY(-4px); border-color: var(--accent); }
.surface-card-top { display: flex; align-items: center; justify-content: space-between; }
.surface-mark { display: grid; place-items: center; width: 2.5rem; height: 2.5rem; border: 1px solid var(--border-strong); color: var(--ink); font-family: var(--font-mono); font-size: 0.68rem; }
.surface-index { color: var(--muted); }
.surface-card h3 { margin: 3.1rem 0 0.55rem; color: var(--ink); font-size: 1.28rem; font-weight: 650; }
.surface-card p { min-height: 3.3rem; margin: 0; color: var(--muted); font-size: 0.9rem; line-height: 1.55; }
.surface-card code { display: block; margin-top: 1.8rem; color: var(--cyan); font-family: var(--font-mono); font-size: 0.76rem; }
.surface-card[data-tone="cyan"] .surface-mark { border-color: rgba(89, 217, 221, 0.58); color: var(--cyan); }
.surface-card[data-tone="violet"] .surface-mark { border-color: rgba(156, 140, 255, 0.58); color: var(--violet); }
.surface-card[data-tone="orange"] .surface-mark { border-color: rgba(255, 139, 77, 0.58); color: var(--accent); }

.lifecycle { border-bottom: 1px solid var(--border); }
.lifecycle-head { display: flex; align-items: end; justify-content: space-between; gap: 2rem; }
.lifecycle-note { max-width: 23rem; margin: 0; color: var(--muted); line-height: 1.65; }
.lifecycle-track { display: grid; grid-template-columns: repeat(4, minmax(0, 1fr)); gap: 0; margin: 2.7rem 0 0; padding: 0; list-style: none; border-top: 1px solid var(--border-strong); }
.lifecycle-track li { position: relative; min-height: 9rem; padding: 1rem 1.2rem 0 0; }
.lifecycle-track li + li { padding-left: 1.2rem; border-left: 1px solid var(--border); }
.step-number { color: var(--accent); font-family: var(--font-mono); font-size: 0.7rem; }
.step-line { position: absolute; top: -0.32rem; left: 0; width: 0.6rem; height: 0.6rem; border: 2px solid var(--accent); background: var(--page); transform: rotate(45deg); }
.lifecycle-track li:first-child .step-line { left: -0.02rem; }
.lifecycle-track strong { display: block; margin-top: 1.55rem; color: var(--ink); font-size: 1.05rem; }
.lifecycle-track p { max-width: 12rem; margin: 0.4rem 0 0; color: var(--muted); font-size: 0.82rem; line-height: 1.5; }

.toolchain { display: grid; grid-template-columns: minmax(0, 0.75fr) minmax(0, 1.25fr); gap: clamp(2rem, 8vw, 8rem); align-items: center; border-bottom: 1px solid var(--border); }
.package-stack { display: grid; gap: 0.7rem; }
.package-stack article { padding: 1.15rem 1.25rem; border: 1px solid var(--border); background: var(--surface); }
.package-stack article > div { display: flex; align-items: center; gap: 0.7rem; }
.package-stack code { color: var(--ink); font-family: var(--font-mono); font-size: 0.9rem; }
.package-icon { display: grid; place-items: center; width: 2rem; height: 2rem; color: var(--page); background: var(--accent); font-family: var(--font-mono); font-size: 0.7rem; font-weight: 600; }
.package-icon.rust { background: var(--violet); }
.package-stack p { margin: 0.8rem 0 0.75rem 2.7rem; color: var(--muted); font-size: 0.87rem; }
.package-role { display: block; margin-left: 2.7rem; color: var(--cyan); font-size: 0.62rem; }

.final-cta { display: flex; align-items: center; justify-content: space-between; gap: 2rem; padding-block: clamp(3.5rem, 8vw, 6rem); }
.final-cta h2 { max-width: 38rem; }
.final-cta .cta { flex: 0 0 auto; }

@media (max-width: 1000px) {
  .hero { grid-template-columns: 1fr; min-height: auto; }
  .hero-copy { max-width: 44rem; }
  .model-board { max-width: 54rem; }
  .anatomy, .toolchain { grid-template-columns: 1fr; gap: 2.5rem; }
}

@media (max-width: 760px) {
  .home-rail { width: min(100% - 2rem, 42rem); }
  .strip-inner, .lifecycle-head, .final-cta { align-items: flex-start; flex-direction: column; }
  .strip-stats { flex-wrap: wrap; gap: 0.8rem 1.4rem; }
  .surface-grid, .lifecycle-track { grid-template-columns: 1fr; }
  .surface-card { min-height: 0; }
  .surface-card h3 { margin-top: 2rem; }
  .lifecycle-track li, .lifecycle-track li + li { min-height: 0; padding: 1rem 0 1.2rem 1.2rem; border-left: 1px solid var(--border); border-bottom: 1px solid var(--border); }
  .lifecycle-track li:first-child { border-left: 1px solid var(--border); }
  .lifecycle-track li:last-child { border-bottom: 0; }
  .step-line, .lifecycle-track li:first-child .step-line { top: 1rem; left: -0.32rem; }
  .lifecycle-track strong { margin-top: 0.7rem; }
  .lifecycle-track p { max-width: none; }
}

@media (max-width: 520px) {
  h1 { font-size: clamp(2.65rem, 14vw, 4rem); }
  .hero { padding-block: 3rem 3.5rem; }
  .model-board { min-height: 28rem; }
  .board-body { padding-inline: 0.8rem; }
  .map { min-height: 20.5rem; }
  .map-node { width: 8rem; padding: 0.65rem 0.7rem; }
  .node-profile { left: 0; }
  .node-session { right: 0; }
  .node-root { top: 6rem; }
  .node-service { bottom: 0; }
  .connector-left { left: 26%; width: 24%; }
  .connector-right { right: 26%; width: 24%; }
  .board-status { margin-left: 0; }
  .schema-card pre { min-height: 13rem; padding: 1.1rem 0.9rem; font-size: 0.72rem; }
}

@media (prefers-reduced-motion: reduce) {
  .surface-card { transition: none; }
}
</style>
