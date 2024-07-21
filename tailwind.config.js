/** @type {import('tailwindcss').Config} */
const colors = require('tailwindcss/colors')
export default {
  purge: ['./index.html', './src/**/*.{vue,js,ts,jsx,tsx}'],
  content: ['./index.html', './src/**/*.{vue,js,ts,jsx,tsx}'],
  theme: {
    extend: {
      colors: {
        primary: {
					DEFAULT: '#10C5A4',
					900: '#080D11',
					850: '#111C22',
					800: '#0A1116',
					700: '#0D151B',
					600: '#1D2C34',
					500: '#12191E',
					400: '#10AF93',
				},
        // primary is teal-600
        // primary: colors.teal[500],
        error: colors.red[400],
        success: colors.green[500]
      }
    }
  },
  plugins: []
}
