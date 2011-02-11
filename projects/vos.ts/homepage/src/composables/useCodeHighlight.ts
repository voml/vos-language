import { createHighlighter, type Highlighter } from "shiki";
import { onMounted, ref, unref, watch, type MaybeRef } from "vue";
import { vosLanguage } from "../vos-lang";

let shared: Highlighter | null = null;
let loading: Promise<Highlighter> | null = null;

async function getHighlighter(): Promise<Highlighter> {
    if (shared) return shared;
    if (!loading) {
        loading = createHighlighter({
            themes: ["everforest-light"],
            langs: [vosLanguage],
        }).then((instance) => {
            shared = instance;
            return instance;
        });
    }
    return loading;
}

export function useCodeHighlight(source: MaybeRef<string>) {
    const highlighted = ref("");
    const highlightError = ref<string | null>(null);

    async function render() {
        const code = unref(source);
        if (!code) {
            highlighted.value = "";
            highlightError.value = null;
            return;
        }
        try {
            const instance = await getHighlighter();
            highlighted.value = instance.codeToHtml(code, {
                lang: "vos",
                theme: "everforest-light",
            });
            highlightError.value = null;
        } catch (error) {
            highlighted.value = "";
            highlightError.value = error instanceof Error ? error.message : String(error);
        }
    }

    onMounted(() => {
        void render();
    });

    watch(
        () => unref(source),
        () => {
            void render();
        },
    );

    return { highlighted, highlightError };
}
