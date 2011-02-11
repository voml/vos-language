<script setup lang="ts">
import { toRef } from "vue";
import { useCodeHighlight } from "../../composables/useCodeHighlight";

const props = defineProps<{
    code: string;
    caption?: string;
}>();

const { highlighted, highlightError } = useCodeHighlight(toRef(props, "code"));
</script>

<template>
  <figure class="panel">
    <figcaption v-if="caption">{{ caption }}</figcaption>
    <pre v-if="highlightError || !highlighted" class="code">{{ code }}</pre>
    <div v-else class="code" v-html="highlighted"></div>
  </figure>
</template>

<style scoped>
.panel {
  margin: 0;
  padding: 1.1rem 1.15rem 1.2rem;
  border: 1px solid var(--border);
  background: var(--surface);
}

figcaption {
  margin-bottom: 0.75rem;
  font-family: var(--font-mono);
  font-size: 0.72rem;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--indigo);
}

.code {
  margin: 0;
  overflow: auto;
  font-family: var(--font-mono);
  font-size: 0.84rem;
  line-height: 1.55;
  white-space: pre;
  color: var(--ink);
}

.code :deep(pre) {
  margin: 0;
  background: transparent !important;
  font-family: inherit;
  font-size: inherit;
  line-height: inherit;
  white-space: pre;
}

.code :deep(code),
.code :deep(.shiki),
.code :deep(.shiki span) {
  font-family: inherit;
  background: transparent !important;
}
</style>
