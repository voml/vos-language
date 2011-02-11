<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { SAMPLE_HERO } from "../../samples";
import CtaLink from "../ui/CtaLink.vue";
import RelationDiagram from "../ui/RelationDiagram.vue";
import SchemaPanel from "../ui/SchemaPanel.vue";

const { t } = useI18n();
</script>

<template>
  <section class="hero">
    <div class="hero-copy">
      <h1 class="brand">{{ t("brand.short") }}</h1>
      <p class="claim">{{ t("home.claim") }}</p>
      <p class="lede">{{ t("home.lede") }}</p>
      <div class="cta-row">
        <CtaLink to="/playground">{{ t("home.openPlayground") }}</CtaLink>
        <CtaLink variant="ghost" to="/model">{{ t("home.readModel") }}</CtaLink>
      </div>
    </div>

    <aside class="hero-stage" aria-hidden="true">
      <div class="stage-frame">
        <RelationDiagram />
        <SchemaPanel :code="SAMPLE_HERO" :caption="t('home.schemaCaption')" />
      </div>
    </aside>
  </section>
</template>

<style scoped>
.hero {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1.05fr);
  gap: 2.75rem;
  align-items: center;
  max-width: 1120px;
  min-height: calc(100vh - 4.25rem);
  margin: 0 auto;
  padding: clamp(2.5rem, 6vw, 4.5rem) 1.25rem 3rem;
}

.hero-copy {
  animation: rise 640ms cubic-bezier(0.22, 1, 0.36, 1) both;
}

.brand {
  margin: 0;
  font-family: var(--font-display);
  font-size: clamp(3.8rem, 9vw, 5.5rem);
  font-weight: 600;
  letter-spacing: -0.02em;
  line-height: 1;
  color: var(--navy);
}

.claim {
  margin: 1.2rem 0 0;
  max-width: 26rem;
  font-family: var(--font-display);
  font-size: clamp(1.25rem, 2.2vw, 1.55rem);
  font-weight: 500;
  letter-spacing: -0.01em;
  line-height: 1.35;
  color: var(--ink);
}

.lede {
  margin: 1rem 0 1.85rem;
  max-width: 30rem;
  color: var(--muted);
  font-size: 1.05rem;
}

.cta-row {
  display: flex;
  flex-wrap: wrap;
  gap: 0.65rem;
}

.hero-stage {
  animation: rise 760ms 90ms cubic-bezier(0.22, 1, 0.36, 1) both;
}

.stage-frame {
  display: grid;
  gap: 1rem;
  padding: 1.25rem;
  border: 1px solid var(--border);
  background:
    linear-gradient(180deg, rgba(28, 49, 87, 0.04), transparent 40%),
    var(--surface);
  box-shadow: inset 0 0 0 1px rgba(255, 255, 255, 0.65);
}

@keyframes rise {
  from {
    opacity: 0;
    transform: translateY(16px);
  }
  to {
    opacity: 1;
    transform: none;
  }
}

@media (prefers-reduced-motion: reduce) {
  .hero-copy,
  .hero-stage {
    animation: none !important;
  }
}

@media (max-width: 960px) {
  .hero {
    grid-template-columns: 1fr;
    min-height: auto;
    gap: 1.75rem;
    padding-top: 2.5rem;
  }
}
</style>
