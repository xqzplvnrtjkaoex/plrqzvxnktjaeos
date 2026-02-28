import { defineConfig } from "./workflows/generated/index.js";

export default defineConfig({
    workflows: "workflows",
    output: ".github",
    generated: "workflows/generated",
});
