<script setup lang="ts">
import { useI18n } from "vue-i18n";
import SectionHead from "../ui/SectionHead.vue";

const { t } = useI18n();

const items = [
    { key: "namespace", mark: "Ns", tone: "navy" },
    { key: "table", mark: "Tb", tone: "indigo" },
    { key: "reference", mark: "Rf", tone: "cyan" },
    { key: "service", mark: "Sv", tone: "amber" },
] as const;
</script>

<template>
  <section class="section">
    <SectionHead
      :title="t('home.primitivesTitle')"
      :lede="t('home.primitivesLede')"
    />
    <div class="grid">
      <article v-for="item in items" :key="item.key" :data-tone="item.tone">
        <span class="mark">{{ item.mark }}</span>
        <h3>{{ t(`home.primitives.${item.key}Title`) }}</h3>
        <p>{{ t(`home.primitives.${item.key}Body`) }}</p>
      </article>
    </div>
  </section>
</template>

<style scoped>
.section {
  max-width: 1120px;
  margin: 0 auto;
  padding: 4rem 1.25rem 1rem;
}

.grid {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 0;
  border-top: 1px solid var(--border);
}

article {
  padding: 1.4rem 1.1rem 1.5rem 0;
  border-bottom: 1px solid var(--border);
}

article + article {
  padding-left: 1.15rem;
  border-left: 1px solid var(--border);
}

.mark {
  display: inline-grid;
  place-items: center;
  width: 2.2rem;
  height: 2.2rem;
  margin-bottom: 0.85rem;
  border: 1px solid var(--border);
  font-family: var(--font-mono);
  font-size: 0.72rem;
  letter-spacing: 0.04em;
  color: var(--navy);
  background: var(--surface);
}

article[data-tone="navy"] .mark {
  background: var(--navy);
  border-color: var(--navy);
  color: #f4f7fc;
}

article[data-tone="indigo"] .mark {
  background: rgba(83, 104, 181, 0.14);
  border-color: rgba(83, 104, 181, 0.35);
  color: #2f3f7a;
}

article[data-tone="cyan"] .mark {
  background: rgba(75, 169, 184, 0.14);
  border-color: rgba(75, 169, 184, 0.4);
  color: #1d5d68;
}

article[data-tone="amber"] .mark {
  background: rgba(210, 154, 74, 0.16);
  border-color: rgba(210, 154, 74, 0.4);
  color: #7a5318;
}

h3 {
  margin: 0 0 0.4rem;
  font-family: var(--font-display);
  font-size: 1.2rem;
  font-weight: 600;
  letter-spacing: -0.02em;
  color: var(--navy);
}

p {
  margin: 0;
  color: var(--muted);
  font-size: 0.95rem;
}

@media (max-width: 960px) {
  .grid {
    grid-template-columns: 1fr 1fr;
  }

  article:nth-child(2n + 1) {
    padding-left: 0;
    border-left: 0;
  }

  article:nth-child(n + 3) {
    border-top: 0;
  }
}

@media (max-width: 640px) {
  .grid {
    grid-template-columns: 1fr;
  }

  article + article {
    padding-left: 0;
    border-left: 0;
  }
}
</style>
