<script setup lang="ts">
import { useI18n } from "vue-i18n";
import RelationDiagram from "../ui/RelationDiagram.vue";
import SectionHead from "../ui/SectionHead.vue";

const { t } = useI18n();
</script>

<template>
  <section class="section">
    <div class="layout">
      <div>
        <SectionHead
          :title="t('home.relationsTitle')"
          :lede="t('home.relationsLede')"
        />
        <p class="note">{{ t("home.relationsNote") }}</p>
      </div>
      <div class="frame">
        <RelationDiagram />
      </div>
    </div>
  </section>
</template>

<style scoped>
.section {
  max-width: 1120px;
  margin: 0 auto;
  padding: 3.5rem 1.25rem 1rem;
}

.layout {
  display: grid;
  grid-template-columns: minmax(0, 1fr) minmax(0, 1fr);
  gap: 2rem;
  align-items: center;
  padding: 1.75rem 0;
  border-top: 1px solid var(--border);
  border-bottom: 1px solid var(--border);
}

.note {
  margin: 0;
  max-width: 28rem;
  color: var(--muted);
}

.frame {
  border: 1px solid var(--border);
  background:
    linear-gradient(180deg, rgba(75, 169, 184, 0.08), transparent 55%),
    var(--surface);
}

@media (max-width: 900px) {
  .layout {
    grid-template-columns: 1fr;
  }
}
</style>
