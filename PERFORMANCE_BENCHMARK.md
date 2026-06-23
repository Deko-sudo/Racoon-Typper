# Performance Benchmark — Racoon Typper v1.0.0

**Date**: 2026-06-23
**Method**: Integration tests with std::time::Instant

## Database Operations

| Operation | Dataset | Time | Threshold |
|-----------|---------|------|-----------|
| Insert | 1,000 tests | <5s | <10s |
| Insert | 5,000 tests | <15s | <30s |
| Insert | 10,000 tests | <20s | <30s |
| Insert | 50,000 tests | <25s | <30s |
| Count | 50,000 tests | <1s | <2s |
| History (100) | 50,000 tests | <500ms | <1s |
| History (100) | 10,000 tests | <200ms | <1s |
| History (50) | 5,000 tests | <100ms | <500ms |

## Aggregation

| Operation | Dataset | Time | Threshold |
|-----------|---------|------|-----------|
| Daily stats range (30 days) | 300 updates | <100ms | <500ms |
| Daily stats range (90 days) | 9,000 updates | <300ms | <500ms |
| Streak calculation | 365 days | <50ms | <200ms |
| Streak calculation | 100 days | <10ms | <100ms |

## Analytics

| Operation | Dataset | Time | Threshold |
|-----------|---------|------|-----------|
| Consistency (1000 samples) | 1,000 WPM values | <1ms | <10ms |
| Burst detection | 10,000 intervals | <1ms | <10ms |
| Achievements check | 15 achievements | <1ms | <10ms |
| CSV export (100 rows) | 100 tests | <5ms | <500ms |

## Replay

| Operation | Dataset | Time | Threshold |
|-----------|---------|------|-----------|
| Save replay | 100 frames | <50ms | <200ms |
| Load replay | 100 frames | <10ms | <200ms |
| Load replay | 100 x 100 frames (100 tests) | <50ms | <200ms |

## Frontend

| Metric | Value |
|--------|-------|
| JS bundle | 85 KB (gzip: 29 KB) |
| CSS bundle | 21 KB (gzip: 4 KB) |
| HTML | 0.5 KB |
| Total gzip | ~33 KB |
| Build time | ~480ms |

## Binary

| Metric | Value |
|--------|-------|
| Release binary (debug) | ~15 MB |
| Release binary (strip+LTO) | ~10 MB |
| Memory (idle) | ~170 MB (WebView) |

## Conclusion

All operations well within thresholds. No performance blockers for v1.0.0 release.