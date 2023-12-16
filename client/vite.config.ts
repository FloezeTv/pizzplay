import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react-swc'
import { resolve } from 'path'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  build: {
    rollupOptions: {
      input: {
        display: resolve(__dirname, 'index.html'),
      },
    },
  },
  server: {
    proxy: {
      '/orders/': 'http://localhost:8080/',
      '/events/subscribe': 'http://localhost:8080/',
    },
  },
})
