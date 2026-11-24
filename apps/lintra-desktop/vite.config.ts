// Tailwind CSS 4 (DEC-MIG-08) runs at build time through its Vite plugin; the
// output is one local stylesheet, so the CSP keeps style-src 'self'.
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vite";

export default defineConfig({ plugins: [tailwindcss()] });
