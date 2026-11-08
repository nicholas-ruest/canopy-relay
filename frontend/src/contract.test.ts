import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import { API_PATHS } from './client';

describe('typed client and OpenAPI drift', () => {
  const contract = readFileSync(new URL('../../openapi.yaml', import.meta.url), 'utf8');

  it('keeps every client route in the versioned contract', () => {
    for (const path of Object.values(API_PATHS)) expect(contract).toContain(`  ${path}:`);
  });

  it('declares denial-before-write and durable read-back operations', () => {
    expect(contract).toContain('operationId: createIntervention');
    expect(contract).toContain("'403':");
    expect(contract).toContain('operationId: listAudit');
    expect(contract).toContain('/api/v1/audit/{id}:');
  });
});
