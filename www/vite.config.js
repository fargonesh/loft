import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from "@tailwindcss/vite"
import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";
import { readFileSync, readdirSync } from 'fs';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));

function loftExamplesPlugin() {
  const virtualModuleId = 'virtual:loft-examples';
  const resolvedId = '\0' + virtualModuleId;
  const examplesDir = resolve(__dirname, '../examples');

  return {
    name: 'loft-examples',
    resolveId(id) {
      if (id === virtualModuleId) return resolvedId;
    },
    load(id) {
      if (id !== resolvedId) return;

      const files = readdirSync(examplesDir)
        .filter(f => f.endsWith('.lf') && !f.startsWith('ffi_'))
        .sort();

      const examples = files.map(file => {
        const content = readFileSync(resolve(examplesDir, file), 'utf-8');
        const name = file
          .replace('.lf', '')
          .replace(/_/g, ' ')
          .replace(/\b\w/g, c => c.toUpperCase());
        const firstLine = content.split('\n')[0];
        const description = firstLine.startsWith('// ')
          ? firstLine.slice(3)
          : `${name} example.`;
        return { name, description, code: content.trimEnd(), file };
      });

      // Watch each example file so dev-server HMR picks up changes
      files.forEach(f => this.addWatchFile(resolve(examplesDir, f)));

      return `export default ${JSON.stringify(examples, null, 2)};`;
    },
  };
}

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    react(),
    tailwindcss(),
    wasm(),
    topLevelAwait(),
    loftExamplesPlugin(),
  ],
  worker: {
    format: 'es',
    plugins: () => [wasm(), topLevelAwait()],
  },
  server: {
    proxy: {
      '/packages': 'http://localhost:5050',
      '/auth/github': 'http://localhost:5050',
      '/auth/me': 'http://localhost:5050',
      '/tokens': 'http://localhost:5050',
      '/pkg-docs': 'http://localhost:5050',
      '/api': 'http://localhost:5050',
    },
    host: '127.0.0.1',
    port: 9916,
    allowedHosts: ['loft.fargone.sh', 'localhost', '127.0.0.1'],
    strictPort: true,
  },
  preview: {
    allowedHosts: ['loft.fargone.sh'],
  }
})
