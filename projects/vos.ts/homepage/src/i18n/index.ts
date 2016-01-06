import {createI18n} from "vue-i18n";
import en from "./locales/en";
import zh from "./locales/zh";

export type AppLocale = "zh" | "en";

const STORAGE_KEY = "vos.locale";

function detectLocale(): AppLocale {
    if (typeof localStorage !== "undefined") {
        const saved = localStorage.getItem(STORAGE_KEY);
        if (saved === "zh" || saved === "en") return saved;
    }
    if (typeof navigator !== "undefined" && navigator.language.toLowerCase().startsWith("zh")) {
        return "zh";
    }
    return "en";
}

export const i18n = createI18n({
    legacy: false,
    locale: detectLocale(),
    fallbackLocale: "en",
    messages: {en, zh},
});

export function setLocale(locale: AppLocale) {
    i18n.global.locale.value = locale;
    if (typeof localStorage !== "undefined") {
        localStorage.setItem(STORAGE_KEY, locale);
    }
    if (typeof document !== "undefined") {
        document.documentElement.lang = locale === "zh" ? "zh-CN" : "en";
    }
}

setLocale(detectLocale());
