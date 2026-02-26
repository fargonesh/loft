import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from "@tailwindcss/vite"
import wasm from "vite-plugin-wasm";
import topLevelAwait from "vite-plugin-top-level-await";

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    react(), 
    tailwindcss(),
    wasm(),
    topLevelAwait()
  ],
  server: {
    proxy: {
      '/packages': 'http://localhost:5050',
      '/auth/github': 'http://localhost:5050',
      '/auth/me': 'http://localhost:5050',
      '/tokens': 'http://localhost:5050',
      '/pkg-docs': 'http://localhost:5050',
      '/api': 'http://localhost:5050',
    }
  },
  preview: {
    allowedHosts: ['loft.fargone.sh'],
  }
})
