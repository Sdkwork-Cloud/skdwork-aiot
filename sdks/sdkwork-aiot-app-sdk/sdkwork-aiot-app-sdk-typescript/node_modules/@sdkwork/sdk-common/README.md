# @sdkwork/sdk-common

Common foundation package for generated TypeScript SDKs.

## Install

```bash
npm install @sdkwork/sdk-common
```

## Authentication Modes

Choose one mode per client instance.

1. API Key mode
- `Authorization: Bearer {apiKey}`

2. Dual-token mode
- `Access-Token: {accessToken}`
- `Authorization: Bearer {authToken}`

## Quick Start

```typescript
import { createBaseHttpClient, createTokenManager } from '@sdkwork/sdk-common';

const tokenManager = createTokenManager({
  accessToken: 'your-access-token',
  authToken: 'your-auth-token',
});

const client = createBaseHttpClient({
  baseUrl: 'https://api.example.com',
  tokenManager,
});

const profile = await client.get<{ id: string; name: string }>('/v1/profile');
console.log(profile.name);
```

API key mode example:

```typescript
import { createBaseHttpClient } from '@sdkwork/sdk-common';

const client = createBaseHttpClient({
  baseUrl: 'https://api.example.com',
  apiKey: 'your-api-key',
});
```

## Exported Modules

- `core`: request/result types, constants, retry/cache/logger config types
- `auth`: token manager and auth header builder
- `http`: `BaseHttpClient` and `createBaseHttpClient`
- `errors`: SDK error hierarchy and type guards
- `utils`: retry, cache, logger, string/encoding/date/object helpers


## Publishing

This SDK includes cross-platform publish scripts in `bin/`:
- `bin/publish-core.mjs`
- `bin/publish.sh`
- `bin/publish.ps1`

### Check

```bash
./bin/publish.sh --action check
```

### Publish

```bash
./bin/publish.sh --action publish --channel release
```

```powershell
.\bin\publish.ps1 --action publish --channel test --dry-run
```

> Set `NPM_TOKEN` (and optional `NPM_REGISTRY_URL`) before release publish.

## License

MIT
