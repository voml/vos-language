<script setup lang="ts">
defineProps<{
    title: string;
    lede?: string;
}>();
</script>

<template>
  <div class="section-head">
    <h2>{{ title }}</h2>
    <p v-if="lede">{{ lede }}</p>
  </div>
</template>

<style scoped>
.section-head {
  margin-bottom: 1.75rem;
  max-width: 38rem;
}

h2 {
  margin: 0 0 0.5rem;
  font-family: var(--font-display);
  font-size: clamp(1.55rem, 2.6vw, 1.95rem);
  font-weight: 600;
  letter-spacing: -0.015em;
  color: var(--navy);
}

p {
  margin: 0;
  color: var(--muted);
  font-size: 1.05rem;
}
</style>
