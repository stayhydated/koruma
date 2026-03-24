import sitemap from "@astrojs/sitemap";
import tailwindcss from "@tailwindcss/vite";
import react from "@astrojs/react";
import mdx from "@astrojs/mdx";
import { defineConfig } from "astro/config";

export default defineConfig({
  site: "https://stayhydated.github.io",
  base: "/koruma",
  vite: {
    plugins: [tailwindcss()],
  },
  build: {
    assets: "_assets",
  },
  integrations: [react(), sitemap(), mdx()],
});
