import {watch} from "vue";
import {createRouter, createWebHistory} from "vue-router";
import {i18n} from "./i18n";
import HomePage from "./views/HomePage.vue";
import PlaygroundPage from "./views/PlaygroundPage.vue";

export const router = createRouter({
    history: createWebHistory(),
    routes: [
        {
            path: "/",
            name: "home",
            component: HomePage,
            meta: {titleKey: "titles.home"},
        },
        {
            path: "/playground",
            name: "playground",
            component: PlaygroundPage,
            meta: {titleKey: "titles.playground"},
        },
    ],
    scrollBehavior() {
        return {top: 0};
    },
});

function applyTitle(titleKey: unknown) {
    if (typeof titleKey !== "string") {
        document.title = "VOS - Virtual Object Schema";
        return;
    }
    document.title = String(i18n.global.t(titleKey));
}

router.afterEach((to) => {
    applyTitle(to.meta.titleKey);
});

watch(i18n.global.locale, () => {
    applyTitle(router.currentRoute.value.meta.titleKey);
});
