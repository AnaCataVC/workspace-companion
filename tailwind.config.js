/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{svelte,js,ts,jsx,tsx}",
  ],
  darkMode: 'media',
  theme: {
    extend: {
      colors: {
        background: 'var(--background)',
        surface: 'var(--surface)',
        border: 'var(--border)',
        accent: 'var(--accent)',
        neutral: {
          750: '#333338',
          850: '#1f1f23',
        },
      },
    },
  },
  plugins: [],
}
