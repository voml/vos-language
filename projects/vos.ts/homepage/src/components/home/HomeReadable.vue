<script setup lang="ts">
import { useI18n } from "vue-i18n";

const { t } = useI18n();
</script>

<template>
  <section class="band">
    <div class="inner">
      <h2>{{ t("home.readableTitle") }}</h2>
      <p>{{ t("home.readableBody") }}</p>
      <p class="keywords">{{ t("home.keywords") }}</p>
    </div>
  </section>
</template>

<style scoped>
.band {
  border-top: 1px solid var(--border);
  border-bottom: 1px solid var(--border);
  background: rgba(28, 49, 87, 0.035);
}

.inner {
  max-width: 40rem;
  margin: 0 auto;
  padding: 3.25rem 1.25rem;
}

h2 {
  margin: 0 0 0.7rem;
  font-family: var(--font-display);
  font-size: clamp(1.7rem, 3vw, 2.2rem);
  font-weight: 600;
  letter-spacing: -0.015em;
  color: var(--navy);
}

p {
  margin: 0;
  color: var(--muted);
  font-size: 1.08rem;
}

.keywords {
  margin-top: 1.25rem;
  font-family: var(--font-mono);
  font-size: 0.8rem;
  letter-spacing: 0.05em;
  color: var(--indigo);
}
</style>
