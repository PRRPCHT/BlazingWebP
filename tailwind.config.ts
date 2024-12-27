import type { Config } from "tailwindcss";

export default {
  content: ["./src/**/*.{html,js,svelte,ts}"],

  theme: {
    extend: {}
  },
  daisyui: {
    themes: [
      {
        mytheme: {
          "primary": "#477A91",
          "primary-content": "#100b03",
          "secondary": "#D6AA58",
          "secondary-content": "#100b03",
          "accent": "#A9CDD4",
          "accent-content": "#0a0f10",
          "neutral": "#374151",
          "neutral-content": "#9ca3af",
          "base-100": "#1d232a",
          "base-200": "#181d23",
          "base-300": "#13171c",
          "base-content": "#ccced0",
          "info": "#7dd3fc",
          "info-content": "#051016",
          "success": "#4ade80",
          "success-content": "#021206",
          "warning": "#fde047",
          "warning-content": "#161202",
          "error": "#F26457",
          "error-content": "#140303",
          },
        },
      ],
    },

  plugins: [
    require('daisyui'),
  ],
} as Config;
