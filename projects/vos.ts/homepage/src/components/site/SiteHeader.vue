<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { RouterLink } from "vue-router";
import LocaleSwitch from "./LocaleSwitch.vue";

const { t } = useI18n();
</script>

<template>
  <header class="site-header">
    <div class="header-inner">
      <RouterLink class="logo" to="/" :aria-label="`${t('brand.short')} home`">
        <span class="logo-mark">VOS</span>
        <span class="logo-divider"></span>
        <span class="logo-caption">{{ t("brand.full") }}</span>
      </RouterLink>

      <nav class="nav" :aria-label="t('nav.primary')">
        <RouterLink to="/">{{ t("nav.home") }}</RouterLink>
        <RouterLink to="/model">{{ t("nav.model") }}</RouterLink>
        <RouterLink to="/playground">{{ t("nav.playground") }}</RouterLink>
      </nav>

      <div class="header-aside">
        <span class="header-status"><i></i> schema language</span>
        <LocaleSwitch />
        <RouterLink class="header-cta" to="/playground">{{ t("nav.try") }}<span>↗</span></RouterLink>
      </div>
    </div>
  </header>
</template>

<style scoped>
.site-header {
  position: sticky;
  top: 0;
  z-index: 20;
  border-bottom: 1px solid var(--border);
  background: rgba(16, 20, 27, 0.88);
  backdrop-filter: blur(16px);
}

.header-inner {
  display: grid;
  grid-template-columns: minmax(13rem, 1fr) auto minmax(13rem, 1fr);
  align-items: center;
  gap: 1rem;
  width: min(1240px, calc(100% - 3rem));
  min-height: 4.2rem;
  margin-inline: auto;
}

.logo { display: inline-flex; align-items: center; gap: 0.7rem; min-width: 0; text-decoration: none; }
.logo-mark { color: var(--ink); font-family: var(--font-mono); font-size: 1rem; font-weight: 600; letter-spacing: 0.08em; }
.logo-divider { width: 1px; height: 1.1rem; background: var(--border-strong); }
.logo-caption { overflow: hidden; color: var(--muted); font-family: var(--font-mono); font-size: 0.68rem; letter-spacing: 0.04em; text-overflow: ellipsis; white-space: nowrap; }
.nav { display: flex; align-items: center; justify-content: center; gap: 1.45rem; }
.nav a { position: relative; padding: 1.45rem 0; color: var(--muted); font-size: 0.83rem; font-weight: 600; text-decoration: none; }
.nav a:hover, .nav a.router-link-active { color: var(--ink); }
.nav a.router-link-active::after { position: absolute; right: 0; bottom: -1px; left: 0; height: 2px; background: var(--accent); content: ""; }
.header-aside { display: flex; align-items: center; justify-content: flex-end; gap: 0.7rem; }
.header-status { display: inline-flex; align-items: center; gap: 0.35rem; color: var(--muted); font-family: var(--font-mono); font-size: 0.62rem; letter-spacing: 0.05em; text-transform: uppercase; }
.header-status i { width: 0.38rem; height: 0.38rem; border-radius: 999px; background: var(--ok); box-shadow: 0 0 0 0.22rem rgba(102, 211, 154, 0.12); }
.header-cta { display: inline-flex; align-items: center; gap: 0.35rem; min-height: 2.25rem; padding: 0 0.8rem; background: var(--accent); color: #18120e; font-size: 0.78rem; font-weight: 700; text-decoration: none; }
.header-cta:hover { background: #ffa16e; }

@media (max-width: 980px) {
  .header-inner { grid-template-columns: auto 1fr auto; }
  .header-status, .logo-caption, .logo-divider { display: none; }
  .nav { justify-content: flex-end; }
}

@media (max-width: 700px) {
  .header-inner { width: min(100% - 2rem, 42rem); grid-template-columns: auto 1fr; padding-block: 0.7rem; }
  .nav { grid-column: 1 / -1; grid-row: 2; justify-content: flex-start; gap: 1.1rem; overflow-x: auto; }
  .nav a { padding: 0.2rem 0 0.65rem; white-space: nowrap; }
  .header-aside { grid-column: 2; grid-row: 1; }
  .header-cta { min-height: 2.1rem; padding-inline: 0.65rem; }
}
</style>
