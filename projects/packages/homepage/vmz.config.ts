import { defineConfig } from '@vmz/vmz';
import shiki from '@vmz/plugin-shiki';

export default defineConfig({
    plugins: [shiki({ textmate: '@game-gpt/vos-textmate/shiki' })],
    engines: { code: 'shiki' },
});
