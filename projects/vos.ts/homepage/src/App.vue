<script setup lang="ts">
import {RouterView} from "vue-router";
import SiteFooter from "./components/SiteFooter.vue";
import SiteHeader from "./components/SiteHeader.vue";
</script>

<template>
  <div class="shell">
    <SiteHeader/>
    <main class="main">
      <RouterView/>
    </main>
    <SiteFooter/>
  </div>
</template>

<style scoped>
.shell {
  min-height: 100vh;
  display: flex;
  flex-direction: column;
}

.main {
  flex: 1;
  width: 100%;
}
</style>
