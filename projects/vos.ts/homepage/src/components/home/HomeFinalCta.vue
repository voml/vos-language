<script setup lang="ts">
import { useI18n } from "vue-i18n";
import CtaLink from "../ui/CtaLink.vue";

const { t } = useI18n();
</script>

<template>
  <section class="final">
    <div class="inner">
      <h2>{{ t("home.finalTitle") }}</h2>
      <p>{{ t("home.finalLede") }}</p>
      <CtaLink to="/playground">{{ t("home.finalCta") }}</CtaLink>
    </div>
  </section>
</template>

<style scoped>
.final {
  margin-top: 3rem;
  border-top: 1px solid var(--border);
  background:
    linear-gradient(180deg, rgba(28, 49, 87, 0.08), transparent 70%),
    var(--surface);
}

.inner {
  max-width: 1120px;
  margin: 0 auto;
  padding: 3.5rem 1.25rem 4rem;
}

h2 {
  margin: 0 0 0.5rem;
  font-family: var(--font-display);
  font-size: clamp(1.7rem, 3vw, 2.25rem);
  font-weight: 600;
  letter-spacing: -0.015em;
  color: var(--navy);
}

p {
  margin: 0 0 1.35rem;
  max-width: 30rem;
  color: var(--muted);
}
</style>
