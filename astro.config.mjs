// @ts-check
import { defineConfig } from "astro/config";
import starlight from "@astrojs/starlight";
import starlightLlmsTxt from "starlight-llms-txt";

// https://astro.build/config
export default defineConfig({
  site: "https://machich.co",
  integrations: [
    starlight({
      title: "mach",
      favicon: "/favicon.svg",
      plugins: [starlightLlmsTxt({ projectName: "mach" })],
      social: [
        {
          icon: "github",
          label: "GitHub",
          href: "https://github.com/rvcas/mach",
        },
        {
          icon: "heart",
          label: "Sponsor",
          href: "https://github.com/sponsors/rvcas",
        },
      ],
      sidebar: [
        {
          label: "Getting Started",
          items: [
            { label: "Installation", slug: "installation" },
            { label: "Quick Start", slug: "quick-start" },
          ],
        },
        {
          label: "Reference",
          items: [
            { label: "CLI Reference", slug: "reference/cli" },
            {
              label: "Keyboard Shortcuts",
              slug: "reference/keyboard-shortcuts",
            },
            { label: "How It Works", slug: "reference/how-it-works" },
          ],
        },
      ],
    }),
  ],
});
