import { defineConfig } from "./workflows/generated/index.js";

export default defineConfig({
    workflows: "workflows/src",
    output: ".github",
    generated: "workflows/generated",
});
