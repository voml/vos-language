<script setup lang="ts">
import {useI18n} from "vue-i18n";
import {RouterLink} from "vue-router";

const {t} = useI18n();
</script>

<template>
  <footer class="site-footer">
    <div class="inner">
      <div>
        <p class="brand">{{ t("brand.short") }}</p>
        <p class="full">{{ t("brand.full") }}</p>
        <p class="blurb">{{ t("footer.blurb") }}</p>
      </div>
      <div class="cols">
        <div>
          <h2>{{ t("footer.explore") }}</h2>
          <ul>
            <li>
              <RouterLink to="/">{{ t("nav.home") }}</RouterLink>
            </li>
            <li>
              <RouterLink to="/playground">{{ t("nav.playground") }}</RouterLink>
            </li>
          </ul>
        </div>
        <div>
          <h2>{{ t("footer.packages") }}</h2>
          <ul>
            <li><code>@game-gpt/vos</code></li>
            <li><code>vos</code> / <code>vos-ast</code> / <code>vos-parser</code></li>
          </ul>
        </div>
        <div>
          <h2>{{ t("footer.related") }}</h2>
          <ul>
            <li>{{ t("footer.von") }}</li>
          </ul>
        </div>
      </div>
    </div>
    <div class="bar">
      <p>© {{ new Date().getFullYear() }} {{ t("footer.copyright") }}</p>
      <p class="mono">{{ t("footer.mono") }}</p>
    </div>
  </footer>
</template>

<style scoped>
.site-footer {
  margin-top: 4rem;
  border-top: 1px solid var(--mist);
  background: rgba(255, 255, 255, 0.35);
}

.inner {
  display: grid;
  grid-template-columns: minmax(0, 1.1fr) minmax(0, 1.4fr);
  gap: 2.5rem;
  max-width: 1120px;
  margin: 0 auto;
  padding: 2.75rem 1.25rem 2rem;
}

.brand {
  margin: 0;
  font-family: var(--font-display);
  font-size: 2.2rem;
  color: var(--sea-deep);
}

.full {
  margin: 0.2rem 0 0.65rem;
  color: var(--ink-soft);
  font-size: 0.92rem;
}

.blurb {
  margin: 0;
  max-width: 28rem;
  color: var(--ink-soft);
}

.cols {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: 1.25rem;
}

.cols h2 {
  margin: 0 0 0.7rem;
  font-size: 0.78rem;
  font-weight: 600;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--ink-soft);
}

.cols ul {
  margin: 0;
  padding: 0;
  list-style: none;
  display: grid;
  gap: 0.45rem;
}

.cols a {
  text-decoration: none;
  font-weight: 500;
}

.cols a:hover {
  color: var(--sea-deep);
}

.cols code {
  font-family: var(--font-mono);
  font-size: 0.82rem;
}

.bar {
  display: flex;
  flex-wrap: wrap;
  justify-content: space-between;
  gap: 0.75rem;
  max-width: 1120px;
  margin: 0 auto;
  padding: 1rem 1.25rem 1.4rem;
  border-top: 1px solid var(--mist);
  color: var(--ink-soft);
  font-size: 0.85rem;
}

.mono {
  margin: 0;
  font-family: var(--font-mono);
  font-size: 0.78rem;
}

@media (max-width: 820px) {
  .inner,
  .cols {
    grid-template-columns: 1fr;
  }
}
</style>
