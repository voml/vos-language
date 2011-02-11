<script setup lang="ts">
import { checkSource } from "@game-gpt/vos";
import { useI18n } from "vue-i18n";
import { computed, ref, watch } from "vue";
import { useCodeHighlight } from "../composables/useCodeHighlight";
import { SAMPLES, type SampleKind } from "../samples";

const { t } = useI18n();
const sampleKind = ref<SampleKind>("persistence");
const source = ref(SAMPLES.persistence);
const { highlighted, highlightError } = useCodeHighlight(source);

const result = computed(() => checkSource(source.value));

watch(sampleKind, (kind) => {
    source.value = SAMPLES[kind];
});

function resetSample() {
    source.value = SAMPLES[sampleKind.value];
}
</script>

<template>
  <div class="playground">
    <header class="page-head">
      <div>
        <p class="eyebrow">{{ t("playground.eyebrow") }}</p>
        <h1>{{ t("playground.title") }}</h1>
        <p class="lede">{{ t("playground.lede") }}</p>
      </div>
      <button type="button" class="reset" @click="resetSample">
        {{ t("playground.reset") }}
      </button>
    </header>

    <div class="tabs" role="tablist">
      <button
        type="button"
        role="tab"
        :data-active="sampleKind === 'persistence'"
        @click="sampleKind = 'persistence'"
      >
        {{ t("playground.samplePersistence") }}
      </button>
      <button
        type="button"
        role="tab"
        :data-active="sampleKind === 'service'"
        @click="sampleKind = 'service'"
      >
        {{ t("playground.sampleService") }}
      </button>
    </div>

    <p class="hint">{{ t("playground.hint") }}</p>

    <div class="play-grid">
      <label class="editor">
        <span class="panel-label">{{ t("playground.source") }}</span>
        <textarea v-model="source" spellcheck="false" />
      </label>

      <div class="preview">
        <span class="panel-label">{{ t("playground.preview") }}</span>
        <pre v-if="highlightError || !highlighted" class="code-frame">{{ source }}</pre>
        <div v-else class="code-frame" v-html="highlighted" />
      </div>
    </div>

    <div class="status" :data-ok="result.ok">
      <strong>{{ result.ok ? t("playground.ok") : t("playground.failed") }}</strong>
      <ul v-if="!result.ok">
        <li v-for="(item, index) in result.diagnostics" :key="index">
          L{{ item.line }} — {{ item.message }}
        </li>
      </ul>
    </div>
  </div>
</template>

<style scoped>
.playground {
  max-width: 1120px;
  margin: 0 auto;
  padding: 2.25rem 1.25rem 3rem;
}

.page-head {
  display: flex;
  flex-wrap: wrap;
  justify-content: space-between;
  gap: 1rem;
  align-items: end;
  margin-bottom: 0.85rem;
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
  margin: 0 0 0.45rem;
  font-family: var(--font-display);
  font-size: clamp(2rem, 4vw, 2.7rem);
  font-weight: 600;
  letter-spacing: -0.015em;
  color: var(--navy);
}

.lede,
.hint {
  margin: 0;
  color: var(--muted);
}

.hint {
  margin: 0.85rem 0 1rem;
  font-size: 0.9rem;
}

.reset {
  min-height: 2.4rem;
  padding: 0 0.95rem;
  border: 1px solid var(--border);
  background: var(--surface);
  color: var(--ink);
  cursor: pointer;
  font-weight: 600;
}

.reset:hover {
  border-color: rgba(83, 104, 181, 0.45);
  background: rgba(83, 104, 181, 0.08);
}

.tabs {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0.35rem;
  padding: 0.3rem;
  border: 1px solid var(--border);
  background: var(--surface);
}

.tabs button {
  min-height: 2.5rem;
  border: 0;
  background: transparent;
  color: var(--muted);
  cursor: pointer;
  font-weight: 600;
}

.tabs button[data-active="true"] {
  background: var(--navy);
  color: #f4f7fc;
}

.play-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 1rem;
}

.editor,
.preview {
  display: flex;
  flex-direction: column;
  gap: 0.55rem;
  min-height: 24rem;
}

.panel-label {
  font-family: var(--font-mono);
  font-size: 0.78rem;
  letter-spacing: 0.04em;
  text-transform: uppercase;
  color: var(--muted);
}

textarea,
.code-frame,
.status {
  border: 1px solid var(--border);
  background: var(--surface);
}

textarea {
  flex: 1;
  min-height: 22rem;
  resize: vertical;
  padding: 1rem 1.1rem;
  color: var(--ink);
  font-family: var(--font-mono);
  font-size: 0.86rem;
  line-height: 1.5;
}

textarea:focus {
  outline: 2px solid rgba(83, 104, 181, 0.35);
  outline-offset: 1px;
}

.code-frame {
  flex: 1;
  min-height: 22rem;
  margin: 0;
  padding: 1rem 1.1rem;
  overflow: auto;
  font-family: var(--font-mono);
  font-size: 0.82rem;
  line-height: 1.5;
  white-space: pre;
}

.code-frame :deep(pre) {
  margin: 0;
  background: transparent !important;
  font-family: inherit;
  white-space: pre;
}

.status {
  margin-top: 1rem;
  padding: 1rem 1.15rem;
}

.status[data-ok="true"] {
  border-color: rgba(31, 122, 77, 0.35);
  background: rgba(31, 122, 77, 0.06);
}

.status[data-ok="false"] {
  border-color: rgba(210, 154, 74, 0.45);
  background: rgba(210, 154, 74, 0.1);
}

.status strong {
  display: block;
  margin-bottom: 0.35rem;
  font-family: var(--font-display);
  font-size: 1.25rem;
  font-weight: 600;
  color: var(--navy);
}

.status[data-ok="true"] strong {
  color: var(--ok);
}

.status[data-ok="false"] strong {
  color: var(--warn);
}

.status ul {
  margin: 0;
  padding-left: 1.1rem;
  color: var(--muted);
}

@media (max-width: 900px) {
  .play-grid,
  .tabs {
    grid-template-columns: 1fr;
  }
}
</style>
