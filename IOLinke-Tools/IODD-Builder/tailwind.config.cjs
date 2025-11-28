/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./src/**/*.{html,js,svelte,ts}",
    "./node_modules/flowbite-svelte/**/*.{html,js,svelte,ts}",
    "./node_modules/flowbite/**/*.js"
  ],
  theme: {
    extend: {
      colors: {
        surface: {
          DEFAULT: "#0f172a",
          foreground: "#f8fafc"
        }
      }
    }
  },
  plugins: [require("flowbite/plugin")]
};