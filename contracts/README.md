# Contracts

Immutable external contract fixtures for the Compat phase.

These files are the source of truth for byte-for-byte API compatibility with the legacy system.
**Do not modify existing fixtures** — add new ones only when the legacy behavior is verified.

## Structure

```
contracts/
  http/             # HTTP request/response golden assertions (status, headers, body)
    auth/           # auth service: /auth/* endpoints
    library/        # library service: /books/* endpoints
    users/          # users service: /users/* endpoints
  cookies/          # Exact Set-Cookie attribute strings per environment
    dev.txt         # dev environment
    prod.txt        # prod environment
  spa-routing/      # SPA route parity assertions
    routes.txt      # paths that must return 200 from nginx
  openapi/          # Frozen public OpenAPI spec (Compat)
    public.yaml     # OpenAPI 3.0 — immutable during Compat phase
```

## Fixture Format (`contracts/http/**/*.json`)

Each file describes one HTTP assertion:

```json
{
  "service": "auth",
  "id": "check_access_token_no_auth",
  "description": "GET /auth/token without auth cookie → 401 Unauthorized",
  "request": {
    "method": "GET",
    "path": "/auth/token",
    "headers": {},
    "body": null
  },
  "expect": {
    "status": 401,
    "headers": {
      "content-type": "application/json"
    },
    "body": {
      "kind": "INVALID_TOKEN",
      "message": "invalid token"
    }
  }
}
```

Fields:
- `service` — service name, matches the subdirectory (`auth`, `library`, `users`)
- `id` — unique ID within the service; should match the filename stem
- `description` — human-readable description shown in harness output
- `request.method` — HTTP method (GET, POST, PATCH, PUT, DELETE)
- `request.path` — absolute path appended to `--base-url`
- `request.headers` — request headers to send (optional)
- `request.body` — JSON request body (optional)
- `expect.status` — expected HTTP status code
- `expect.headers` — expected response headers (subset match; optional)
- `expect.body` — expected JSON response body (exact match; optional)

**Fixture design rule:** assert everything that is part of the API contract; omit what is
irrelevant. Concretely:

- JSON error responses → assert `content-type: application/json` + the full `body`
  (`kind` + `message`); see `.claude/docs/error-kinds.md` for the list of kinds
- Success responses with a JSON body → assert `content-type: application/json` + the
  full `body`
- Cookie-setting responses → assert the `Set-Cookie` header attributes
- Responses with an empty body (e.g. 204, or axum `StatusCode` rejections) → status
  only; do not add a `body` field

## Cookie Contract Format (`contracts/cookies/*.txt`)

Lines starting with `#` are comments. Each non-comment line describes one
expected `Set-Cookie` header attribute string.

`{ROOT_DOMAIN}` is a placeholder substituted at harness runtime from the
`ROOT_DOMAIN` environment variable.

The format matches the canonical Set-Cookie output from
`previous/auth-madome-app/src/app.rs` (`set_token_pair` / `set_empty_token_pair`):

```
# Token set (POST /auth/token, PATCH /auth/token):
madome_access_token=<value>; Domain={ROOT_DOMAIN}; Max-Age=604800; Path=/; SameSite=Lax; HttpOnly; Secure
madome_refresh_token=<value>; Domain={ROOT_DOMAIN}; Max-Age=604800; Path=/auth/token; SameSite=Lax; HttpOnly; Secure

# Token clear (DELETE /auth/token):
madome_access_token=; Domain={ROOT_DOMAIN}; Max-Age=0; Path=/; SameSite=Lax; HttpOnly; Secure
madome_refresh_token=; Domain={ROOT_DOMAIN}; Max-Age=0; Path=/; SameSite=Lax; HttpOnly; Secure
```

Cookie verification is done attribute-by-attribute in the auth service
integration tests (Unit C onward). The harness uses these patterns as
the reference during E2E smoke tests (Unit J).

## SPA Routing Format (`contracts/spa-routing/routes.txt`)

Lines starting with `#` are comments. Each non-comment line is an absolute
path that nginx must serve `index.html` for (HTTP 200). Dynamic segments
use concrete example values so nginx wildcard rules are exercised.

## Running the Harness

```bash
# Build
cargo build -p contract-harness

# Run all fixtures against the gateway
cargo run -p contract-harness -- --base-url http://localhost:3000 --env dev

# Run only auth service fixtures (direct service port)
cargo run -p contract-harness -- --base-url http://localhost:3112 --service auth

# Run only library service fixtures
cargo run -p contract-harness -- --base-url http://localhost:3113 --service library

# Run only users service fixtures
cargo run -p contract-harness -- --base-url http://localhost:3114 --service users
```

Exit codes: `0` = all pass, `1` = any fail.

## Adding New Contracts

1. Verify the behavior in `previous/` — read the legacy source, not docs.
2. Add a new `.json` file to the appropriate `contracts/http/{service}/` directory.
3. Run `cargo build -p contract-harness` to confirm JSON is valid.
4. The new fixture must fail until the service is implemented — this is expected.

Do not modify or delete existing fixtures during the Compat phase.

## Expected Status (Unit B)

At this stage no services are implemented yet. All harness assertions **fail** —
this is intentional. The failing output proves that the contracts are correctly
capturing the gap between expected and actual behavior.

When services are implemented (Unit C onward), assertions begin to pass one
service at a time.
