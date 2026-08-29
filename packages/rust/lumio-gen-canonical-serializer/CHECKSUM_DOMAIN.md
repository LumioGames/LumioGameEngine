# Snapshot Header Checksum Domain (SnapshotHeaderV1, the B profile)

Generated with the CanonicalSerializer artifact. Do not hand-edit.
Authority: ADR-047 section 4; form: `CanonicalJsonV1` (ADR-041).

## The two digests are not the same digest

- `hash` covers the **payload bytes** the header describes: `SHA-256(payload)`,
  where the payload is the uncompressed domain bytes (ADR-047: encoded under
  `LumioBinV1` when the payload is binary). It says nothing about the header.
- `checksum` covers the **header** with the `checksum`, `hash` members removed:
  `SHA-256(CanonicalJsonV1({"digestDomain":"SnapshotHeaderV1","header":<header minus those two>}))`.
  Omitting both is what makes the value computable at all — `checksum` cannot
  cover itself, and `hash` is omitted so a re-hash of the payload does not
  force a header rewrite.

Domain tag: `SnapshotHeaderV1`. The tag is a member of the digest input, exactly as in
ADR-041 section 2, so a B-profile digest can never collide with an A-profile one.

## Golden

Input (`fixtures/valid/snapshot-active.json`, the registered positive fixture):

```json
{"digestDomain":"SnapshotHeaderV1","header":{"activationState":"Active","compression":"None","createdAt":"2026-08-27T01:10:00Z","encryption":"None","gameReleaseId":"A-1.1.0","magic":"LUMIOSNP1","payload":"snapshot-payload-v1","payloadLength":19,"productId":"A","revisions":{"chunkRevisionSet":{"c:0:0:0":9},"configRevision":4,"gameRevision":18,"replicationRevision":13,"schemaEpoch":1,"tickId":42,"voxelWorldRevision":24},"schemaVersion":1,"sessionId":"session-001","snapshotId":"snapshot-100","tickId":42}}
```

```text
checksum = ea67f6fec5a15e33c3873a6769bc1102942455b7da68cb9bd2bb8742ddb6597d
```

The architecture gate recomputes this value from the fixture on every run, so
the Golden cannot drift away from the rule it documents.
