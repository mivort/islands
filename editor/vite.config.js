import { defineConfig } from 'vite';
import { viteSingleFile } from 'vite-plugin-singlefile';
import includeHtml from 'vite-plugin-include-html';

export default defineConfig({
  base: '',
  plugins: [viteSingleFile(), includeHtml()],
  server: {
    strictPort: true,
  },
});
