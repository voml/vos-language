<script setup lang="ts">
withDefaults(
    defineProps<{
        compact?: boolean;
    }>(),
    { compact: false },
);
</script>

<template>
  <div class="diagram" :class="{ compact }" aria-hidden="true">
    <div class="node root">User</div>
    <div class="spine spine-a"></div>
    <div class="branch">
      <div class="node indigo">Profile</div>
      <div class="node indigo">Session</div>
    </div>
    <div class="spine spine-b"></div>
    <div class="node cyan">Service</div>
  </div>
</template>

<style scoped>
.diagram {
  display: grid;
  justify-items: center;
  gap: 0;
  width: 100%;
  padding: 1.5rem 1rem;
  font-family: var(--font-mono);
  font-size: 0.82rem;
  letter-spacing: 0.02em;
}

.node {
  position: relative;
  z-index: 1;
  min-width: 7.5rem;
  padding: 0.7rem 1rem;
  border: 1px solid var(--border);
  background: var(--surface);
  color: var(--navy);
  text-align: center;
  font-weight: 500;
}

.node.root {
  border-color: rgba(28, 49, 87, 0.35);
  background: var(--navy);
  color: #f4f7fc;
}

.node.indigo {
  border-color: rgba(83, 104, 181, 0.4);
  background: rgba(83, 104, 181, 0.1);
  color: #2f3f7a;
}

.node.cyan {
  border-color: rgba(75, 169, 184, 0.45);
  background: rgba(75, 169, 184, 0.12);
  color: #1d5d68;
}

.spine {
  width: 1px;
  height: 1.5rem;
  background: linear-gradient(180deg, var(--cyan), rgba(75, 169, 184, 0.2));
}

.branch {
  position: relative;
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 1.75rem;
  width: min(100%, 22rem);
  padding-top: 1.35rem;
}

.branch::before {
  content: "";
  position: absolute;
  top: 0;
  left: 25%;
  right: 25%;
  height: 1px;
  background: var(--cyan);
}

.branch::after {
  content: "";
  position: absolute;
  top: 0;
  left: 50%;
  width: 1px;
  height: 1.35rem;
  background: var(--cyan);
  transform: translateX(-50%);
}

.branch .node::before {
  content: "";
  position: absolute;
  top: -1.35rem;
  left: 50%;
  width: 1px;
  height: 1.35rem;
  background: var(--cyan);
  transform: translateX(-50%);
}

.compact {
  padding: 1rem 0.5rem;
  font-size: 0.76rem;
}

.compact .node {
  min-width: 6.2rem;
  padding: 0.55rem 0.75rem;
}

@media (max-width: 520px) {
  .branch {
    gap: 0.9rem;
  }

  .node {
    min-width: 0;
    width: 100%;
  }
}
</style>
