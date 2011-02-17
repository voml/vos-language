# scripts/

Root automation for formatting and npm publish.

## Layout

```text
scripts/
  format.mjs           Biome format wrapper
  ci/
    publish-npm.mjs    Real release (OIDC via publish-npm.yml)
    publish-placeholder.mjs   0.0.0 stubs + npm trust setup
```

## npm placeholder (0.0.0)

Reserve package names before Trusted Publisher real releases:

```bash
pnpm placeholder          # status
pnpm placeholder:publish  # publish @game-gpt/vos + @game-gpt/vos-skills @0.0.0
pnpm placeholder:trust    # configure Trusted Publisher (needs NPM_TOTP_SECRET in .env.placeholder.local)
```

Local secrets (gitignored): `.env.placeholder.local` at repo root.

Real versions: push tag `vX.Y.Z` or `workflow_dispatch` on `publish-npm.yml` (environment `NPM_PUBLISH`).
