# Decisions

## D1: Fix monthly cost tracking race condition with flock + atomic write

**Decision:** Use file locking (`fs2`/`flock`) + atomic write (temp file + fsync + rename) to fix the concurrent read-modify-write race on `monthly_cost.json`.

**Alternatives considered:**

| Approach | Verdict | Why rejected/chosen |
|---|---|---|
| **flock + atomic write** | **Chosen** | Minimal code change, tiny dependency (`fs2` ~20KB), correct for single-user local filesystem. Lock held <1ms so contention is negligible even with 100+ sessions |
| SQLite (WAL + UPSERT) | Rejected | Strongest correctness guarantees (transactions, crash recovery, monotonic MAX), but adds ~1-2MB binary bloat via bundled C library. Overkill for a status bar tool writing one number per session |
| Per-session files | Rejected | Eliminates shared state entirely (zero contention), but creates many small files (100+/month), needs cleanup logic, bigger redesign than necessary |
| Append-only log | Rejected | Lock-free writes via O_APPEND, but needs replay/dedup/compaction — too database-like for this tool |
| Atomic rename only | Rejected | Prevents torn writes but does NOT prevent lost updates from stale reads — the actual bug |
| Hybrid (per-session + consolidation) | Rejected | Complexity jump with no practical gain at this scale |

**Additional hardening applied:**
- Monotonic max (`session_cost.max(existing)`) so a session's cost can never decrease, even from stale/late writes
- Month computed inside lock to prevent midnight boundary races
- Fsync parent directory after rename for full crash durability
- Write errors propagated (return `None`) instead of silently ignored

**Evidence:** Two rounds of deep analysis (codex_thinkdeep) + code review (codex_review) confirmed flock as best fit-to-effort ratio. SQLite was the "more correct" choice but the correctness gap is small for local single-user single-tool access patterns.

**Commit:** `019eae2` (2026-03-07)
