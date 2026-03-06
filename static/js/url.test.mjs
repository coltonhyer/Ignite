import assert from 'node:assert/strict';
import { buildShareUrl, parseShareUrl } from './url.mjs';

function runTests() {
  console.log('Running url.mjs tests...');

  // Test 1: buildShareUrl
  const url = buildShareUrl('abc-123', 'keydata');
  assert.equal(url, '/s/abc-123#keydata', 'buildShareUrl should produce the correct relative path');

  // Test 2: parseShareUrl with valid string
  const parsed1 = parseShareUrl('/s/abc-123#keydata');
  assert.deepEqual(parsed1, { id: 'abc-123', key: 'keydata' }, 'parseShareUrl should extract id and key correctly');

  // Test 3: key with special base64url characters
  const specialKeyUrl = buildShareUrl('uuid-456', 'aBcDe_Fg-HiJ-Kl_Mn');
  const parsed2 = parseShareUrl(specialKeyUrl);
  assert.deepEqual(parsed2, { id: 'uuid-456', key: 'aBcDe_Fg-HiJ-Kl_Mn' }, 'parseShareUrl should preserve base64url characters');

  // Test 4: parseShareUrl with absolute URL
  const parsed3 = parseShareUrl('https://example.com/s/789-xyz#someKeyData');
  assert.deepEqual(parsed3, { id: '789-xyz', key: 'someKeyData' }, 'parseShareUrl should work with absolute URLs');

  // Test 5: parseShareUrl invalid path
  const parsedInvalid1 = parseShareUrl('/p/abc-123#keydata');
  assert.equal(parsedInvalid1, null, 'parseShareUrl should return null for invalid path');

  // Test 6: parseShareUrl missing hash
  const parsedInvalid2 = parseShareUrl('/s/abc-123');
  assert.equal(parsedInvalid2, null, 'parseShareUrl should return null when missing hash');

  // Test 7: parseShareUrl empty hash
  const parsedInvalid3 = parseShareUrl('/s/abc-123#');
  assert.equal(parsedInvalid3, null, 'parseShareUrl should return null when hash is empty');

  // Test 8: parseShareUrl too many path segments
  const parsedInvalid4 = parseShareUrl('/s/abc-123/extra#key');
  assert.equal(parsedInvalid4, null, 'parseShareUrl should return null for extra path segments');

  // Test 9: default without window throws
  const parsedInvalid5 = parseShareUrl();
  assert.equal(parsedInvalid5, null, 'parseShareUrl should return null when no URL and no window are provided');

  console.log('All tests passed!');
}

runTests();
