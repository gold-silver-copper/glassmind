# Corpus provenance and licenses

Training source is the already mounted **BabyLM 2025 strict-small 10M**
`train_10M.zip`, not the updated 2026 collection. Exact local archive/member
versions are pinned by SHA-256 in `provenance.json`; no upstream revision ID
is supplied by the mounted README. No new corpus was downloaded. All six
members (54,351,399 raw bytes) are included. The mounted README is 3,589 bytes,
SHA-256 `20d4993f0f36a965add35cbf0554cbb42b131f60f29bd80db16ab56a3f40c9c0`.
Source README says documents are newline separated and minimally processed.
The [BabyLM site](https://babylm.github.io/) now describes 2026 data, so its
current download is not a substitute for the pinned 2025 archive.

This is mixed-source research data, **not a blanket MIT/Apache corpus**.
The code's repository license does not relicense the corpus or count artifact.
The corpus and artifact remain in the private worker run cache.

| Training member | Mounted README statement | Primary source checked 2026-10-10 (local date) |
| --- | --- | --- |
| bnc_spoken | BNC User Licence; distribution under fair dealing, term 2g | [BNC licence](https://www.natcorp.ox.ac.uk/docs/licence.html) restricts redistribution and reproduction; not an unrestricted open license. |
| childes | Links TalkBank ground rules | Old `/share/rules.html` URL is gone; [current rules](https://talkbank.org/0share/rules.html) specify CC BY-NC-SA 3.0 except otherwise indicated, citations, and restrictions on commercial incorporation. |
| gutenberg | Project Gutenberg license | [Project Gutenberg license](https://www.gutenberg.org/policy/license.html) distinguishes US-unrestricted text, copyrighted books, and trademark/license conditions. No book-level relicensing claim made. |
| open_subtitles | Says “Open source”, cites Lison & Tiedemann (2016), v2018 URL | [OPUS v2018 primary page](https://opus.nlpl.eu/legacy/OpenSubtitles-v2018.php) requests source attribution and paper citation, but does not supply a precise blanket SPDX grant. [OpenSubtitles](https://www.opensubtitles.org/) is the originating provider. License specificity remains unresolved. |
| simple_wiki | Links Wikimedia dump legal terms; source snapshot 2022-12-01 | [Current Wikimedia dump terms](https://dumps.wikimedia.org/legal.html) describe CC BY-SA 4.0/GFDL with exceptions. The exact older snapshot's per-document license/version was not independently audited; do not infer it from today's page. |
| switchboard | Links Potts's SwDA page; Godfrey et al. (1992), Stolcke et al. (2000) | [SwDA primary distribution page](https://compprag.christopherpotts.net/swda.html) describes free transcript downloads, but no precise blanket SPDX license was confirmed there. |

Citations from the mounted README: BNC Consortium (2007), British National
Corpus XML Edition; MacWhinney (2000), The CHILDES Project; Gerlach &
Font-Clos (2020), A standardized Project Gutenberg corpus for statistical
analysis of natural language and quantitative linguistics; Lison &
Tiedemann (2016), OpenSubtitles2016: Extracting Large Parallel Corpora from
Movie and TV Subtitles; Godfrey et al. (1992), SWITCHBOARD; Stolcke et al.
(2000), Dialogue act modeling for automatic tagging and recognition of
conversational speech.

Dev uses only the explicitly supplied `grammar.jsonl` and `reading.jsonl`.
Their exact bytes/hashes and complete denominators are in `provenance.json`.
The existing manifest identifies its source archives as BLiMP master zip
SHA-256 `8c043d41701fff01cabf22fb9a2fd8e15cecac5240b777ff41680f4eb7fffc17`
and BabyLM dev zip SHA-256
`b5af75aeb1ee5a36f65ee98f316fdae73b16a0ba3767cca3f3dfa5eaac2dc621`.
Neither source archive was opened. Dev text is not copied into the repository;
no official test, other BLiMP examples, or hidden evaluation code was read.
