<script setup lang="ts">
import { RouterLink } from "vue-router";

withDefaults(
    defineProps<{
        to?: string;
        href?: string;
        variant?: "primary" | "ghost";
    }>(),
    { variant: "primary" },
);
</script>

<template>
  <RouterLink v-if="to" class="cta" :class="variant" :to="to">
    <slot />
  </RouterLink>
  <a v-else class="cta" :class="variant" :href="href">
    <slot />
  </a>
</template>

<style scoped>
.cta {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-height: 2.8rem;
  padding: 0 1.15rem;
  border: 1px solid transparent;
  text-decoration: none;
  font-weight: 600;
  font-size: 0.88rem;
  letter-spacing: 0;
  transition:
    background 160ms ease,
    border-color 160ms ease,
    color 160ms ease;
}

.cta.primary {
  background: var(--accent);
  color: #18120e;
}

.cta.primary:hover {
  background: #ffa16e;
}

.cta.ghost {
  border-color: var(--border);
  background: transparent;
  color: var(--ink);
}

.cta.ghost:hover {
  border-color: var(--cyan);
  background: rgba(89, 217, 221, 0.08);
}
</style>
