<script setup lang="ts">
import { useI18n } from "vue-i18n";
import HomeLifecycle from "../components/home/HomeLifecycle.vue";
import HomePrimitives from "../components/home/HomePrimitives.vue";
import HomeRelations from "../components/home/HomeRelations.vue";
import CtaLink from "../components/ui/CtaLink.vue";
import RelationDiagram from "../components/ui/RelationDiagram.vue";
import SchemaPanel from "../components/ui/SchemaPanel.vue";
import { SAMPLE_HERO } from "../samples";

const { t } = useI18n();
</script>

<template>
  <div class="page">
    <header class="page-head">
      <p class="eyebrow">{{ t("model.eyebrow") }}</p>
      <h1>{{ t("model.title") }}</h1>
      <p class="lede">{{ t("model.lede") }}</p>
    </header>

    <section class="overview">
      <RelationDiagram />
      <SchemaPanel :code="SAMPLE_HERO" :caption="t('home.schemaCaption')" />
    </section>

    <HomePrimitives />
    <HomeRelations />
    <HomeLifecycle />

    <div class="footer-cta">
      <CtaLink to="/playground">{{ t("model.openPlayground") }}</CtaLink>
      <CtaLink variant="ghost" to="/">{{ t("model.backHome") }}</CtaLink>
    </div>
  </div>
</template>

<style scoped>
.page {
  padding-bottom: 3rem;
}

.page-head {
  max-width: 1120px;
  margin: 0 auto;
  padding: 2.5rem 1.25rem 0.5rem;
}

.eyebrow {
  margin: 0 0 0.35rem;
  color: var(--indigo);
  font-family: var(--font-mono);
  font-size: 0.78rem;
  letter-spacing: 0.07em;
  text-transform: uppercase;
}

h1 {
  margin: 0 0 0.5rem;
  font-family: var(--font-display);
  font-size: clamp(2rem, 4vw, 2.8rem);
  font-weight: 600;
  letter-spacing: -0.015em;
  color: var(--navy);
}

.lede {
  margin: 0;
  max-width: 38rem;
  color: var(--muted);
}

.overview {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 1rem;
  max-width: 1120px;
  margin: 1.75rem auto 0;
  padding: 0 1.25rem;
}

.overview > :first-child {
  border: 1px solid var(--border);
  background: var(--surface);
}

.footer-cta {
  display: flex;
  flex-wrap: wrap;
  gap: 0.65rem;
  max-width: 1120px;
  margin: 2.5rem auto 0;
  padding: 0 1.25rem;
}

@media (max-width: 900px) {
  .overview {
    grid-template-columns: 1fr;
  }
}
</style>
