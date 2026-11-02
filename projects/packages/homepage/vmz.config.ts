import { defineConfig } from '@vmz/plugin';
import shiki from '@vmz/plugin-shiki';

export default defineConfig({
    plugins: [shiki({ textmate: '@game-gpt/vos-textmate/shiki' })],
    engines: { code: 'shiki' },
    delivery: {
        default: 'static',
        profiles: {
            static: { host: 'browser', assembly: 'web-static' },
        },
    },
});
