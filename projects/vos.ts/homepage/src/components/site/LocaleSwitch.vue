<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { setLocale, type AppLocale } from "../../i18n";

const { t, locale } = useI18n();

function onChange(event: Event) {
    const value = (event.target as HTMLSelectElement).value as AppLocale;
    setLocale(value);
}
</script>

<template>
  <label class="locale">
    <span class="sr-only">{{ t("locale.label") }}</span>
    <select :value="locale" @change="onChange">
      <option value="zh">{{ t("locale.zh") }}</option>
      <option value="en">{{ t("locale.en") }}</option>
    </select>
  </label>
</template>

<style scoped>
.locale select {
  min-height: 2.15rem;
  padding: 0 0.55rem;
  border: 1px solid var(--border);
  background: var(--surface-2);
  color: var(--muted);
  font-size: 0.82rem;
  font-weight: 500;
}

.locale select:hover {
  border-color: var(--border-strong);
  color: var(--ink);
}

.sr-only {
  position: absolute;
  width: 1px;
  height: 1px;
  padding: 0;
  margin: -1px;
  overflow: hidden;
  clip: rect(0, 0, 0, 0);
  white-space: nowrap;
  border: 0;
}
</style>
