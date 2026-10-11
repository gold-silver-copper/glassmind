Pre-DEV implementation accounting: normalization uses the existing <=129 f32
score vector plus <=129 (action,f64) legal scores, each local to one parent.
PLAN's singular action-score-vector wording omitted this bounded temporary vector.
No expansion states beyond the4 retained parents/4 chosen successors, and no
change to preregistered objective, search width, steps or accepted input limits.
All129 scores are checked, including illegal scores. Original64-label limit applies.
