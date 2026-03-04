/** @type {import('tailwindcss').Config} */
export default {
  // Use the "class" strategy so we control dark mode via a `.dark` class
  // on <html>, allowing System / Light / Dark switching.
  darkMode: "class",
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx}"],
  theme: {
    extend: {
      colors: {
        // Brand accent — a subtle anthropic-orange-ish amber.
        accent: {
          DEFAULT: "#D97706",
          hover: "#B45309",
          subtle: "#FEF3C7",
          "subtle-dark": "#292524",
        },
      },
      boxShadow: {
        popover:
          "0 8px 32px rgba(0,0,0,0.18), 0 2px 8px rgba(0,0,0,0.12), 0 0 0 0.5px rgba(0,0,0,0.08)",
        "popover-dark":
          "0 8px 32px rgba(0,0,0,0.55), 0 2px 8px rgba(0,0,0,0.4), 0 0 0 0.5px rgba(255,255,255,0.06)",
      },
      borderRadius: {
        popover: "14px",
      },
      fontFamily: {
        sans: [
          "-apple-system",
          "BlinkMacSystemFont",
          "SF Pro Display",
          "Segoe UI",
          "Helvetica Neue",
          "sans-serif",
        ],
        mono: ["SF Mono", "JetBrains Mono", "Fira Code", "monospace"],
      },
      animation: {
        "fade-in": "fadeIn 0.15s ease-out",
        spin: "spin 1s linear infinite",
      },
      keyframes: {
        fadeIn: {
          "0%": { opacity: "0", transform: "translateY(-4px)" },
          "100%": { opacity: "1", transform: "translateY(0)" },
        },
      },
    },
  },
  plugins: [],
};
