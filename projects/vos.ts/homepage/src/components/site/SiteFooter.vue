<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { RouterLink } from "vue-router";

const { t } = useI18n();
</script>

<template>
  <footer class="site-footer">
    <div class="footer-inner">
      <div class="footer-brand">
        <p class="footer-logo">VOS<span>.</span></p>
        <p>{{ t("footer.blurb") }}</p>
      </div>
      <div class="footer-links">
        <div>
          <p class="footer-label">{{ t("footer.explore") }}</p>
          <RouterLink to="/model">{{ t("nav.model") }}</RouterLink>
          <RouterLink to="/playground">{{ t("nav.playground") }}</RouterLink>
        </div>
        <div>
          <p class="footer-label">{{ t("footer.runtime") }}</p>
          <span>@game-gpt/vos</span>
          <span>vos</span>
        </div>
        <div>
          <p class="footer-label">{{ t("footer.related") }}</p>
          <span>{{ t("footer.von") }}</span>
        </div>
      </div>
    </div>
    <div class="footer-bar">
      <span>© {{ new Date().getFullYear() }} {{ t("footer.copyright") }}</span>
      <span class="footer-mono">VOS / schema language</span>
    </div>
  </footer>
</template>

<style scoped>
.site-footer { border-top: 1px solid var(--border); background: #0b0e13; }
.footer-inner { display: grid; grid-template-columns: minmax(0, 1fr) minmax(0, 1.25fr); gap: 4rem; width: min(1240px, calc(100% - 3rem)); margin-inline: auto; padding-block: 3.5rem 2.5rem; }
.footer-brand { max-width: 24rem; }
.footer-logo { margin: 0 0 0.7rem; color: var(--ink); font-family: var(--font-mono); font-size: 1.4rem; font-weight: 600; letter-spacing: 0.08em; }
.footer-logo span { color: var(--accent); }
.footer-brand > p:last-child { margin: 0; color: var(--muted); font-size: 0.9rem; line-height: 1.65; }
.footer-links { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 1.4rem; }
.footer-links > div { display: grid; align-content: start; gap: 0.5rem; }
.footer-label { margin: 0 0 0.35rem; color: var(--accent); font-family: var(--font-mono); font-size: 0.66rem; letter-spacing: 0.1em; text-transform: uppercase; }
.footer-links a, .footer-links span { color: var(--muted); font-size: 0.8rem; text-decoration: none; }
.footer-links a:hover { color: var(--ink); }
.footer-bar { display: flex; justify-content: space-between; gap: 1rem; width: min(1240px, calc(100% - 3rem)); margin-inline: auto; padding-block: 1rem 1.25rem; border-top: 1px solid var(--border); color: #667181; font-size: 0.72rem; }
.footer-mono { font-family: var(--font-mono); font-size: 0.66rem; }
@media (max-width: 760px) { .footer-inner { grid-template-columns: 1fr; gap: 2.5rem; width: min(100% - 2rem, 42rem); } .footer-links { grid-template-columns: repeat(2, minmax(0, 1fr)); } .footer-bar { flex-direction: column; width: min(100% - 2rem, 42rem); } }
@media (max-width: 430px) { .footer-links { grid-template-columns: 1fr; } }
</style>
