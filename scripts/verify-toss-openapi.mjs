#!/usr/bin/env node

const SPEC_URL = 'https://openapi.tossinvest.com/openapi-docs/latest/openapi.json';
const EXPECTED_TITLE = '토스증권 Open API';
const EXPECTED_BASE_URL = 'https://openapi.tossinvest.com';
const EXPECTED_PATHS = [
  '/oauth2/token',
  '/api/v1/orderbook',
  '/api/v1/prices',
  '/api/v1/trades',
  '/api/v1/price-limits',
  '/api/v1/candles',
  '/api/v1/stocks',
  '/api/v1/stocks/{symbol}/warnings',
  '/api/v1/exchange-rate',
  '/api/v1/market-calendar/KR',
  '/api/v1/market-calendar/US',
  '/api/v1/accounts',
  '/api/v1/holdings',
  '/api/v1/orders',
  '/api/v1/orders/{orderId}',
  '/api/v1/orders/{orderId}/modify',
  '/api/v1/orders/{orderId}/cancel',
  '/api/v1/buying-power',
  '/api/v1/sellable-quantity',
  '/api/v1/commissions',
];

function fail(message) {
  console.error(`Toss OpenAPI verification failed: ${message}`);
  process.exitCode = 1;
}

const response = await fetch(SPEC_URL, {
  headers: { accept: 'application/json' },
});

if (!response.ok) {
  fail(`HTTP ${response.status} while fetching ${SPEC_URL}`);
  process.exit();
}

const spec = await response.json();
const paths = Object.keys(spec.paths ?? {});
const servers = (spec.servers ?? []).map((server) => server.url);
const missingPaths = EXPECTED_PATHS.filter((path) => !paths.includes(path));
const accountHeaderRefs = JSON.stringify(spec).match(/X-Tossinvest-Account/g)?.length ?? 0;
const hasRateLimitHeaders = ['Retry-After', 'X-RateLimit-Limit', 'X-RateLimit-Remaining'].every(
  (header) => JSON.stringify(spec).includes(header),
);
const schemas = spec.components?.schemas ?? {};
const errorResponse = schemas.ErrorResponse;
const apiError = schemas.ApiError;
const oauthError = schemas.OAuth2ErrorResponse;

if (spec.info?.title !== EXPECTED_TITLE) {
  fail(`unexpected title: ${spec.info?.title ?? '(missing)'}`);
}
if (!servers.includes(EXPECTED_BASE_URL)) {
  fail(`missing official server: ${EXPECTED_BASE_URL}`);
}
if (paths.length !== EXPECTED_PATHS.length) {
  fail(`expected ${EXPECTED_PATHS.length} paths, found ${paths.length}`);
}
if (missingPaths.length > 0) {
  fail(`missing paths: ${missingPaths.join(', ')}`);
}
if (accountHeaderRefs === 0) {
  fail('missing X-Tossinvest-Account references');
}
if (!hasRateLimitHeaders) {
  fail('missing expected rate-limit headers');
}
if (errorResponse?.required?.includes('error') !== true) {
  fail('ErrorResponse must require error');
}
if (errorResponse?.properties?.error?.$ref !== '#/components/schemas/ApiError') {
  fail('ErrorResponse.error must reference ApiError');
}
if (!apiError?.properties?.code || !apiError?.properties?.message) {
  fail('ApiError must expose code and message');
}
if (!oauthError?.properties?.error || !oauthError?.properties?.error_description) {
  fail('OAuth2ErrorResponse must expose error and error_description');
}

const endpointInventory = paths.map((path) => {
  const methods = Object.keys(spec.paths[path]).filter((key) =>
    ['get', 'post', 'put', 'patch', 'delete'].includes(key),
  );
  return `${methods.join(',').toUpperCase().padEnd(8)} ${path}`;
});

console.log(`Toss OpenAPI: ${spec.info.title} version ${spec.info.version}`);
console.log(`Server: ${servers.join(', ')}`);
console.log(`Paths: ${paths.length}`);
console.log(`X-Tossinvest-Account refs: ${accountHeaderRefs}`);
console.log('Error schemas: ErrorResponse, ApiError, OAuth2ErrorResponse OK');
console.log('Endpoint inventory:');
for (const line of endpointInventory) {
  console.log(`- ${line}`);
}
