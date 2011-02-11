<script setup lang="ts">
import { useI18n } from "vue-i18n";
import SectionHead from "../ui/SectionHead.vue";

const { t } = useI18n();

const steps = ["define", "connect", "validate", "evolve"] as const;
</script>

<template>
  <section class="section">
    <SectionHead
      :title="t('home.lifecycleTitle')"
      :lede="t('home.lifecycleLede')"
    />
    <ol class="flow">
      <li v-for="(step, index) in steps" :key="step">
        <span class="index">0{{ index + 1 }}</span>
        <strong>{{ t(`home.lifecycle.${step}Title`) }}</strong>
        <p>{{ t(`home.lifecycle.${step}Body`) }}</p>
      </li>
    </ol>
  </section>
</template>

<style scoped>
.section {
  max-width: 1120px;
  margin: 0 auto;
  padding: 4rem 1.25rem 1rem;
}

.flow {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 0;
  margin: 0;
  padding: 0;
  list-style: none;
  border-top: 1px solid var(--border);
}

li {
  position: relative;
  padding: 1.4rem 1.1rem 1.5rem 0;
  border-bottom: 1px solid var(--border);
}

li + li {
  padding-left: 1.2rem;
  border-left: 1px solid var(--border);
}

li + li::before {
  content: "";
  position: absolute;
  top: 1.85rem;
  left: -0.35rem;
  width: 0.55rem;
  height: 0.55rem;
  border: 1px solid var(--amber);
  background: var(--page);
  transform: rotate(45deg);
}

.index {
  display: block;
  margin-bottom: 0.55rem;
  font-family: var(--font-mono);
  font-size: 0.75rem;
  letter-spacing: 0.08em;
  color: var(--amber);
}

strong {
  display: block;
  margin-bottom: 0.35rem;
  font-family: var(--font-display);
  font-size: 1.15rem;
  font-weight: 600;
  color: var(--navy);
}

p {
  margin: 0;
  color: var(--muted);
  font-size: 0.94rem;
}

@media (max-width: 900px) {
  .flow {
    grid-template-columns: 1fr 1fr;
  }

  li:nth-child(2n + 1) {
    padding-left: 0;
    border-left: 0;
  }

  li:nth-child(2n + 1)::before {
    display: none;
  }
}

@media (max-width: 560px) {
  .flow {
    grid-template-columns: 1fr;
  }

  li + li {
    padding-left: 0;
    border-left: 0;
  }

  li + li::before {
    display: none;
  }
}
</style>
