<script setup lang="ts">
import {useI18n} from "vue-i18n";
import {RouterLink} from "vue-router";
import {ref} from "vue";
import {useCodeHighlight} from "../composables/useCodeHighlight";
import {SAMPLE_PERSISTENCE} from "../samples";

const {t} = useI18n();
const sample = ref(SAMPLE_PERSISTENCE);
const {highlighted, highlightError} = useCodeHighlight(sample);
</script>

<template>
  <div class="home">
    <section class="hero">
      <div class="hero-copy">
        <p class="eyebrow">{{ t("home.eyebrow") }}</p>
        <h1 class="brand">{{ t("brand.short") }}</h1>
        <p class="full-name">{{ t("brand.full") }}</p>
        <p class="lede">{{ t("home.lede") }}</p>
        <div class="cta-row">
          <RouterLink class="cta primary" to="/playground">
            {{ t("home.openPlayground") }}
          </RouterLink>
          <a class="cta ghost" href="#surface">{{ t("home.viewPackages") }}</a>
        </div>
      </div>

      <div class="hero-visual" aria-hidden="true">
        <div class="orbit"></div>
        <div class="shard shard-a"></div>
        <div class="shard shard-b"></div>
        <pre v-if="highlightError || !highlighted" class="hero-code">{{ sample }}</pre>
        <div v-else class="hero-code" v-html="highlighted"></div>
      </div>
    </section>

    <section id="surface" class="section features">
      <div class="section-head">
        <h2>{{ t("home.whyTitle") }}</h2>
        <p>{{ t("home.whyLede") }}</p>
      </div>
      <div class="feature-grid">
        <article>
          <h3>{{ t("home.features.namespaceTitle") }}</h3>
          <p>{{ t("home.features.namespaceBody") }}</p>
        </article>
        <article>
          <h3>{{ t("home.features.tableTitle") }}</h3>
          <p>{{ t("home.features.tableBody") }}</p>
        </article>
        <article>
          <h3>{{ t("home.features.refTitle") }}</h3>
          <p>{{ t("home.features.refBody") }}</p>
        </article>
        <article>
          <h3>{{ t("home.features.attrTitle") }}</h3>
          <p>{{ t("home.features.attrBody") }}</p>
        </article>
        <article>
          <h3>{{ t("home.features.enumsTitle") }}</h3>
          <p>{{ t("home.features.enumsBody") }}</p>
        </article>
        <article>
          <h3>{{ t("home.features.serviceTitle") }}</h3>
          <p>{{ t("home.features.serviceBody") }}</p>
        </article>
      </div>
    </section>

    <section id="packages" class="section packages">
      <div class="section-head">
        <h2>{{ t("home.packagesTitle") }}</h2>
        <p>{{ t("home.packagesLede") }}</p>
      </div>
      <div class="pkg-grid">
        <article>
          <code>@game-gpt/vos</code>
          <p>{{ t("home.pkgVos") }}</p>
        </article>
        <article>
          <code>projects/vos.rs</code>
          <p>{{ t("home.pkgRs") }}</p>
        </article>
      </div>
      <div class="cta-row narrow">
        <RouterLink class="cta primary" to="/playground">
          {{ t("home.tryPlayground") }}
        </RouterLink>
      </div>
    </section>
  </div>
</template>

<style scoped>
.home {
  max-width: 1120px;
  margin: 0 auto;
  padding: 0 1.25rem 2rem;
}

.hero {
  display: grid;
  grid-template-columns: minmax(0, 1.05fr) minmax(0, 0.95fr);
  gap: 2.5rem;
  align-items: center;
  min-height: calc(100vh - 4.5rem);
  padding: 3.5rem 0 2.5rem;
}

.eyebrow {
  margin: 0 0 0.75rem;
  color: var(--ink-soft);
  font-size: 0.78rem;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.brand {
  margin: 0;
  font-family: var(--font-display);
  font-size: clamp(5rem, 14vw, 8rem);
  font-weight: 400;
  letter-spacing: -0.04em;
  line-height: 0.85;
  color: var(--sea-deep);
}

.full-name {
  margin: 0.55rem 0 1rem;
  color: var(--ink-soft);
  font-family: var(--font-mono);
  font-size: 0.95rem;
}

.lede {
  margin: 0 0 1.75rem;
  max-width: 36rem;
  color: var(--ink-soft);
  font-size: 1.08rem;
}

.cta-row {
  display: flex;
  flex-wrap: wrap;
  gap: 0.75rem;
}

.cta-row.narrow {
  margin-top: 1.5rem;
}

.cta {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-height: 2.75rem;
  padding: 0 1.15rem;
  border: 1px solid transparent;
  text-decoration: none;
  font-weight: 600;
}

.cta.primary {
  background: var(--sea);
  color: #f4fbfb;
}

.cta.primary:hover {
  background: var(--sea-deep);
}

.cta.ghost {
  border-color: rgba(10, 77, 87, 0.28);
  background: rgba(255, 255, 255, 0.4);
}

.hero-visual {
  position: relative;
  min-height: 26rem;
  display: grid;
  place-items: center;
}

.orbit {
  position: absolute;
  inset: 10% 8%;
  border: 1px solid var(--glow-line);
  border-radius: 46% 54% 42% 58% / 52% 44% 56% 48%;
  background: radial-gradient(circle at 30% 25%, rgba(255, 255, 255, 0.75), transparent 42%),
  linear-gradient(145deg, rgba(15, 111, 124, 0.14), rgba(196, 123, 58, 0.12));
  animation: morph 12s ease-in-out infinite;
}

.shard {
  position: absolute;
  width: 5rem;
  height: 5rem;
  border: 1px solid rgba(24, 33, 43, 0.12);
  background: rgba(255, 255, 255, 0.48);
  backdrop-filter: blur(6px);
}

.shard-a {
  top: 14%;
  left: 12%;
  rotate: 14deg;
  animation: drift 7s ease-in-out infinite;
}

.shard-b {
  right: 10%;
  bottom: 18%;
  width: 3.5rem;
  height: 3.5rem;
  rotate: -10deg;
  animation: drift 8.5s ease-in-out infinite reverse;
}

.hero-code {
  position: relative;
  z-index: 1;
  width: min(100%, 28rem);
  max-height: 22rem;
  margin: 0;
  padding: 1.25rem 1.35rem;
  overflow: auto;
  border: 1px solid rgba(24, 33, 43, 0.1);
  background: rgba(247, 249, 251, 0.9);
  box-shadow: 0 24px 60px rgba(24, 33, 43, 0.08);
  font-family: var(--font-mono);
  font-size: 0.8rem;
  line-height: 1.5;
  white-space: pre;
}

.hero-code :deep(pre) {
  margin: 0;
  background: transparent !important;
  font-family: inherit;
  font-size: inherit;
  white-space: pre;
}

.section {
  padding: 3.5rem 0 1rem;
}

.section-head h2 {
  margin: 0 0 0.4rem;
  font-family: var(--font-display);
  font-size: 2.2rem;
  font-weight: 400;
}

.section-head p {
  margin: 0 0 1.4rem;
  color: var(--ink-soft);
}

.feature-grid,
.pkg-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: 0.9rem;
}

.feature-grid article,
.pkg-grid article {
  padding: 1.15rem 1.2rem;
  border-left: 3px solid var(--sea);
  background: rgba(255, 255, 255, 0.45);
}

.feature-grid h3 {
  margin: 0 0 0.4rem;
  font-family: var(--font-display);
  font-size: 1.35rem;
  font-weight: 400;
}

.feature-grid p,
.pkg-grid p {
  margin: 0;
  color: var(--ink-soft);
}

.pkg-grid code {
  display: inline-block;
  margin-bottom: 0.45rem;
  font-family: var(--font-mono);
  color: var(--sea-deep);
}

@keyframes morph {
  0%,
  100% {
    border-radius: 46% 54% 42% 58% / 52% 44% 56% 48%;
    transform: rotate(0deg) scale(1);
  }
  50% {
    border-radius: 54% 46% 58% 42% / 44% 56% 44% 56%;
    transform: rotate(-3deg) scale(1.02);
  }
}

@keyframes drift {
  0%,
  100% {
    transform: translateY(0);
  }
  50% {
    transform: translateY(-10px);
  }
}

@media (prefers-reduced-motion: reduce) {
  .orbit,
  .shard-a,
  .shard-b {
    animation: none !important;
  }
}

@media (max-width: 900px) {
  .hero,
  .feature-grid,
  .pkg-grid {
    grid-template-columns: 1fr;
  }

  .hero {
    min-height: auto;
  }
}
</style>
