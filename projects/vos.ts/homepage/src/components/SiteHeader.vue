<script setup lang="ts">
import {useI18n} from "vue-i18n";
import {RouterLink} from "vue-router";
import LocaleSwitch from "./LocaleSwitch.vue";

const {t} = useI18n();
</script>

<template>
  <header class="site-header">
    <div class="inner">
      <RouterLink class="logo" to="/" :aria-label="`${t('brand.short')} home`">
        <span class="mark">{{ t("brand.short") }}</span>
        <span class="tag">{{ t("brand.full") }}</span>
      </RouterLink>

      <nav class="nav" :aria-label="t('nav.primary')">
        <RouterLink to="/">{{ t("nav.home") }}</RouterLink>
        <RouterLink to="/playground">{{ t("nav.playground") }}</RouterLink>
      </nav>

      <div class="aside">
        <LocaleSwitch/>
        <code class="pkg">@game-gpt/vos</code>
        <RouterLink class="cta" to="/playground">{{ t("nav.try") }}</RouterLink>
      </div>
    </div>
  </header>
</template>

<style scoped>
.site-header {
  position: sticky;
  top: 0;
  z-index: 20;
  border-bottom: 1px solid var(--mist);
  background: rgba(247, 249, 251, 0.88);
  backdrop-filter: blur(12px);
}

.inner {
  display: grid;
  grid-template-columns: auto 1fr auto;
  align-items: center;
  gap: 1.25rem;
  max-width: 1120px;
  margin: 0 auto;
  padding: 0.85rem 1.25rem;
}

.logo {
  display: inline-flex;
  align-items: baseline;
  gap: 0.55rem;
  min-width: 0;
  text-decoration: none;
}

.mark {
  font-family: var(--font-display);
  font-size: 1.7rem;
  font-weight: 400;
  letter-spacing: -0.03em;
  color: var(--sea-deep);
}

.tag {
  color: var(--ink-soft);
  font-size: 0.72rem;
  letter-spacing: 0.02em;
  white-space: nowrap;
}

.nav {
  display: flex;
  justify-content: center;
  gap: 1.35rem;
}

.nav a {
  position: relative;
  text-decoration: none;
  color: var(--ink-soft);
  font-weight: 600;
  font-size: 0.95rem;
}

.nav a:hover,
.nav a.router-link-active {
  color: var(--sea-deep);
}

.nav a.router-link-active::after {
  content: "";
  position: absolute;
  left: 0;
  right: 0;
  bottom: -0.55rem;
  height: 2px;
  background: var(--sea);
}

.aside {
  display: flex;
  align-items: center;
  gap: 0.65rem;
}

.pkg {
  display: none;
  padding: 0.2rem 0.45rem;
  border: 1px solid var(--mist);
  background: rgba(255, 255, 255, 0.55);
  font-family: var(--font-mono);
  font-size: 0.72rem;
  color: var(--ink-soft);
}

.cta {
  display: inline-flex;
  align-items: center;
  min-height: 2.15rem;
  padding: 0 0.9rem;
  background: var(--sea);
  color: #f4fbfb;
  text-decoration: none;
  font-weight: 600;
  font-size: 0.9rem;
}

.cta:hover {
  background: var(--sea-deep);
}

@media (min-width: 980px) {
  .pkg {
    display: inline-block;
  }
}

@media (max-width: 820px) {
  .inner {
    grid-template-columns: 1fr auto;
  }

  .nav {
    grid-column: 1 / -1;
    justify-content: flex-start;
    gap: 1rem;
  }

  .tag {
    display: none;
  }
}
</style>
