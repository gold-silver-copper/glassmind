# Complete graph QA calibration: rejected promotion

The complete independently acquired graph answers 554 of 1,949 calibration
questions correctly under the frozen strict entity/relation/answer metric,
compared with two for the TRAIN999 baseline. Both old correct answers remain.
It refuses 1,365 and answers 30 incorrectly. Answer-only scoring is 558 correct,
1,365 refused and 26 wrong; the stricter metric remains the decision criterion.

The promotion gate fails because 29 previously refused cases become wrong,
despite the 552 net correct gain. The predeclared requirement was at least ten
net gains and zero newly wrong cases from prior correct/refused questions. No
DEV run, default promotion, threshold search or output repair followed. The
single-shard rejection remains separate. This is reused TRAIN calibration,
not held-out evidence or a new trained-model result.

All 1,949 rows are accounted for: 2 right-to-right; 538 refused-to-right;
1,296 refused-to-refused; 29 refused-to-wrong; 14 wrong-to-right;
69 wrong-to-refused; one wrong-to-wrong. There are no lost correct answers.
The complete supervised run finished in 36.008 seconds, peaking at 103,604 KiB
process RSS and 32,587,776 added-run bytes, below the frozen bounds.

The broader source changes claim coverage, not linking: 1,013 questions link,
843 links match the annotated entity and 792 match both entity and relation,
the same as the single-shard experiment. Of 1,946 present annotated anchors,
1,213 have the unique annotated local answer, 663 have multiple answers including
it, 52 have a nonempty set without it and 18 have no claim. This is an offline
oracle diagnostic; it never supplies runtime features or claims.

Of the 29 newly wrong cases, 12 select the wrong entity, seven mismatch the
annotated relation and ten disagree with the annotated answer despite matching
entity and relation. These categories do not establish that a current public
source assertion is false: source date, granularity and benchmark wording may
differ. The original metric and labels remain unchanged. Several wrong links
select generic words such as “celestial object” inside the request rather than
the named object. That motivates a separate hypothesis about locating entity
mentions, instead of another coverage-only acquisition or gate adjustment.

Plan6fa9581 and runnerc22d730 preceded inference. The CLI and all inputs were
rehashed, all predictions frozen before the annotation join, and the original
joint-audit scorer was hash-pinned unchanged. Original predictions, full report
and mismatch details remain in the retained run; artifacts.json records hashes.
No model calls, API authentication, downloads or changes to production code.
