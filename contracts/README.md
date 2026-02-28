# Contracts

Immutable external contract fixtures for the Compat phase.

These files are the source of truth for byte-for-byte API compatibility with the legacy system.
Do not modify existing fixtures — add new ones only when the legacy behavior is verified.

## Structure

```
contracts/
  http/        # HTTP request/response golden assertions (status, headers, bodies)
  cookies/     # Exact Set-Cookie strings per environment (dev/prod)
  spa-routing/ # SPA route parity assertions
  openapi/     # Frozen public OpenAPI spec (Compat)
```

Content is added incrementally as each service is implemented (Unit B onward).
