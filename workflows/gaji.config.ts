import { defineConfig } from "./generated/index.js";

export default defineConfig({
    workflows: "src",
    output: "../.github",
    generated: "generated",
});
