/** @type {import('tailwindcss').Config} */
module.exports = {
  darkMode: 'class',
  content: [
    "./index.html",
    "./src/**/*.rs",  // <--- TO JEST KLUCZOWE! Skanuj wszystkie pliki .rs w folderze src
  ],
  theme: {
    extend: {},
  },
  plugins: [],
}