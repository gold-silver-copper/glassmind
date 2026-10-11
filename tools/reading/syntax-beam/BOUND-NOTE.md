Pre-DEV bounds clarification: the frozen PLAN used a conservative128-label ceiling.
Actual existing loader and trainer cap labels at64, so the tighter bound is129
actions and4*129*256=132096 examined transition scores. No algorithm change.
Four retained parent states plus at most four temporary successor states during
replacement; at most four descriptors, no materialized expansion-state pool.
