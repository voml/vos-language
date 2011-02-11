<script setup lang="ts">
import { useI18n } from "vue-i18n";
import SectionHead from "../ui/SectionHead.vue";

const { t } = useI18n();
</script>

<template>
  <section class="section">
    <SectionHead
      :title="t('home.packagesTitle')"
      :lede="t('home.packagesLede')"
    />
    <div class="row">
      <article>
        <code>@game-gpt/vos</code>
        <p>{{ t("home.pkgVos") }}</p>
      </article>
      <article>
        <code>vos</code>
        <p>{{ t("home.pkgRs") }}</p>
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

.row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0;
  border-top: 1px solid var(--border);
  border-bottom: 1px solid var(--border);
}

article {
  padding: 1.5rem 0;
}

article + article {
  padding-left: 1.5rem;
  border-left: 1px solid var(--border);
}

code {
  display: inline-block;
  margin-bottom: 0.5rem;
  font-family: var(--font-mono);
  font-size: 0.95rem;
  color: var(--indigo);
}

p {
  margin: 0;
  color: var(--muted);
}

@media (max-width: 760px) {
  .row {
    grid-template-columns: 1fr;
  }

  article + article {
    padding-left: 0;
    border-left: 0;
  }
}
</style>
