import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// https://vite.dev/config/
export default defineConfig({
  plugins: [react()],
  server: {
    port: 5173,      // force Vite to always use 5173
    strictPort: true // if 5173 is busy, fail instead of switching
  }
})
