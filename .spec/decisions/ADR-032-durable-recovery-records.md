# ADR-032: Durable Recovery Records

- **Status**: Historical · Accepted (Implementation Baseline `LGE-V1.3-2026-08-27`, accepted 2026-08-27)
- **Owner**: `LumioGameRuntime` (format), Host persistence (durability)
- **Baseline**: `LGE-V1.3-2026-08-27`
- **Relation**: Refines [ADR-010](ADR-010-persistence-config.md) and [ADR-011](ADR-011-observability.md). Those Accepted Decision texts are unchanged. A `LoggingEvent` is not a recovery record.

## Context

TxnJournal and CommandLog existed only as logging categories with free-form fields. Recovery needs sequenced, hashed, idempotent records.

## Decision

Publish `txn-journal-record.schema.json`, `command-log-record.schema.json` and `wal-record-envelope.schema.json`. Every record carries `recordVersion`, `recordSeq`, Session/Release/Tick/Txn or Command correlation, `recordKind`, `idempotencyKey`, `previousHash`, `payloadHash`, `length`, `commitState`, `durabilityState` and `checksum`. The WAL envelope wraps one journal or command record with the same hash chain.

`LoggingEvent` categories `TxnJournal` and `CommandLog` remain diagnostic/audit mirrors. They must not be used as the recovery input.

## Contract

The three schemas above. Game command payloads stay Game-generated inside the hashed payload.

## Failure semantics

A broken `previousHash` chain, a checksum mismatch, or a LoggingEvent presented as a recovery record is invalid.

## Alternatives

Reusing LoggingEvent as the journal was rejected because it has no hash chain or commit marker.

## Compatibility and migration

Additive in `LGE-V1.3-2026-08-27`.

## Verification

Fixtures `journal/committed`, `journal/broken-previous-hash`, `cmdlog/appended`, `cmdlog/checksum-mismatch`.
