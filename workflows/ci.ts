import { getAction, Job, Workflow } from "../generated/index.js";

const checkout = getAction("actions/checkout@v5");
const rustToolchain = getAction("dtolnay/rust-toolchain@stable");
const rustCache = getAction("swatinem/rust-cache@v2");

function rustJob(jobName: string, command: string): Job {
  return new Job("ubuntu-latest").steps((s) =>
    s
      .add(checkout({ name: "Checkout" }))
      .add(rustToolchain({ with: { toolchain: "stable" } }))
      .add(rustCache({ name: "Cache Rust artifacts" }))
      .add({ name: jobName, run: command }),
  );
}

new Workflow({
  name: "CI",
  on: {
    push: { branches: ["master", "dev"] },
    pull_request: { branches: ["master", "dev"] },
  },
})
  .jobs((j) =>
    j
      .add("fmt", rustJob("Check formatting", "cargo fmt --all -- --check"))
      .add(
        "clippy",
        rustJob(
          "Lint",
          "cargo clippy --workspace --all-targets --all-features -- -D warnings",
        ),
      )
      .add("test", rustJob("Test", "cargo test --workspace --all-features")),
  )
  .build("ci");
