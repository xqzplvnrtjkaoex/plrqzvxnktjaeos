# auth service

HTTP authentication service. Handles passwordless login via one-time codes, JWT token issuance/refresh/revocation, and WebAuthn passkey registration and authentication.

## Configuration (environment variables)

| Variable | Required | Description |
|----------|----------|-------------|
| `DATABASE_URL` | Yes | PostgreSQL connection URL |
| `REDIS_URL` | Yes | Redis connection URL |
| `JWT_SECRET` | Yes | HMAC secret for signing access and refresh tokens |
| `WEBAUTHN_RP_ID` | Yes | WebAuthn relying-party ID (e.g. `example.com`) |
| `WEBAUTHN_ORIGIN` | Yes | WebAuthn relying-party origin URL (e.g. `https://example.com`) |
| `COOKIE_DOMAIN` | Yes | Cookie domain attribute (root domain, e.g. `example.com`) |
| `AUTH_PORT` | No | TCP port to listen on (default: `3112`) |

## Running migrations

```bash
DATABASE_URL=... cargo run -p madome-auth-migration
```

## Endpoints

| Method | Path | Auth | Description |
|--------|------|------|-------------|
| `POST` | `/auth/code` | None | Request a one-time auth code via email |
| `POST` | `/auth/token` | None | Exchange code for token pair (login) |
| `GET` | `/auth/token` | Cookie | Validate access token, optionally check role |
| `PATCH` | `/auth/token` | Cookie | Refresh token pair using refresh token |
| `DELETE` | `/auth/token` | Identity | Revoke tokens (clear cookies) |
| `GET` | `/auth/passkeys` | Identity | List registered passkeys |
| `DELETE` | `/auth/passkeys/{credential_id}` | Identity | Delete a passkey |
| `POST` | `/auth/passkey/registration` | Identity | Start WebAuthn passkey registration |
| `PATCH` | `/auth/passkey/registration` | Identity | Finish WebAuthn passkey registration |
| `POST` | `/auth/passkey/authentication` | None | Start WebAuthn passkey authentication |
| `PATCH` | `/auth/passkey/authentication` | None | Finish WebAuthn passkey authentication |
| `GET` | `/healthz` | None | Liveness probe |
| `GET` | `/readyz` | None | Readiness probe |

## Auth mechanisms

- **Cookie**: reads `madome_access_token` or `madome_refresh_token` cookie directly
- **Identity**: reads gateway-injected `x-madome-user-id` and `x-madome-user-role` headers

## Token details

- Access token: JWT HS256, exp = 14400 s (4 h), cookie Max-Age = 604800 s (7 d)
- Refresh token: JWT HS256, exp = 604800 s (7 d), cookie path `/auth/token`
