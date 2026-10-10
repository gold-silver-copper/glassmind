# References

Research related to glassmind, downloaded October 2026. The PDFs are
gitignored, since most of them can't be redistributed. Each one can be
downloaded again from the link listed with it.

## Understanding English with hand-written code

- `1971-winograd-shrdlu.pdf`: Terry Winograd, *Procedures as a Representation
  for Data in a Computer Program for Understanding Natural Language* (MIT PhD
  thesis, SHRDLU). <https://dspace.mit.edu/handle/1721.1/7095>. Scanned, no
  text layer.
  The closest ancestor. It reads English commands and questions about a
  small world, acts, answers and explains itself, all with hand-written
  procedures. It is impressive inside its world, it was never tested on input
  its author hadn't seen, and it never grew beyond that world.
- `2000-copestake-flickinger-english-resource-grammar.pdf`: Copestake &
  Flickinger, *An open source grammar development environment and
  broad-coverage English grammar using HPSG* (LREC 2000).
  <https://aclanthology.org/L00-1276/>
  A large hand-written English grammar that both parses and generates through
  a structured meaning representation. Shows what broad symbolic coverage
  takes.
- `2023-lenat-marcus-what-llms-might-learn-from-cyc.pdf`: Lenat & Marcus,
  *Getting from Generative AI to Trustworthy AI: What LLMs might learn from
  Cyc*. <https://arxiv.org/abs/2308.04445>
  The Cyc project's own account of 40 years of hand-built knowledge, and what
  it thinks symbolic systems offer that LLMs don't.
- `2019-sutton-bitter-lesson.html`: Rich Sutton, *The Bitter Lesson*.
  <http://www.incompleteideas.net/IncIdeas/BitterLesson.html>
  The main argument against this project: general methods that scale with
  computing power have so far beaten hand-built knowledge.

## Grammars that grow from use

- `2015-wang-berant-liang-semantic-parser-overnight.pdf`: Wang, Berant &
  Liang, *Building a Semantic Parser Overnight* (ACL 2015).
  <https://aclanthology.org/P15-1129/>
  A small grammar plus paraphrases written for the task, used to learn a
  parser. Also a warning: paraphrases written for a task don't sound like real
  users.
- `2017-wang-et-al-naturalizing-programming-language-voxelurn.pdf`: Wang,
  Ginn, Liang & Manning, *Naturalizing a Programming Language via Interactive
  Learning* (ACL 2017). <https://aclanthology.org/P17-1086/>
  A command language whose grammar grows as users teach it new phrasings.
- `2023-wang-et-al-grammar-prompting.pdf`: Wang et al., *Grammar Prompting for
  Domain-Specific Language Generation with Large Language Models* (NeurIPS
  2023). <https://arxiv.org/abs/2305.19234>
  An LLM picks the minimal subset of an expert-written grammar for each
  example, then generates within it. It shows LLMs can work well with formal
  grammars, but they choose rules here; they don't write new ones.

## LLMs growing programs under an automatic scorer

- `2023-romera-paredes-et-al-funsearch.pdf`: Romera-Paredes et al.,
  *Mathematical discoveries from program search with large language models*
  (Nature 2023, FunSearch).
  <https://www.nature.com/articles/s41586-023-06924-6>
- `2025-novikov-et-al-alphaevolve.pdf`: Novikov et al., *AlphaEvolve: A coding
  agent for scientific and algorithmic discovery*.
  <https://arxiv.org/abs/2506.13131>. Open-source version: OpenEvolve,
  <https://github.com/codelion/openevolve>
  An LLM proposes code changes and a scorer keeps the better ones, in a loop
  with no human in between. The model for glassmind's improvement loop.
- `2023-wang-et-al-voyager.pdf`: Wang et al., *Voyager: An Open-Ended Embodied
  Agent with Large Language Models*. <https://arxiv.org/abs/2305.16291>
  An agent stores what it learns as a growing library of code skills, kept
  only after it checks that they work. The closest to "knowledge stored as a
  codebase that LLMs grow".
- `2021-ellis-et-al-dreamcoder.pdf`: Ellis et al., *DreamCoder: Growing
  generalizable, interpretable knowledge with wake-sleep Bayesian program
  learning* (PLDI 2021). <https://arxiv.org/abs/2006.08381>
- `2023-grand-et-al-lilo.pdf`: Grand et al., *LILO: Learning Interpretable
  Libraries by Compressing and Documenting Code* (ICLR 2024).
  <https://arxiv.org/abs/2310.19791>
  Learn reusable helper functions, keeping those that shrink the total of the
  library plus every solution rewritten to use it (the library itself grows).
  Also shows that unnamed helpers hurt LLMs and that auto-written docs fix
  this. The basis for glassmind's size-for-gain acceptance rule and its
  compression rounds.

## Turning LLM or neural knowledge into readable rules and programs

- `2024-wang-et-al-llms-are-interpretable-learners.pdf`: Wang et al., *Large
  Language Models are Interpretable Learners*.
  <https://arxiv.org/abs/2406.17224>
- `2026-distilling-asp-theories-from-llms.pdf`: Higuera Ruiz, Hofmarcher et
  al., *Distilling Answer Set Programming Theories from Large Language Models*
  (NeSy 2026). <https://arxiv.org/abs/2607.28086>
  An LLM's knowledge extracted as readable logic rules. The closest current
  work to "the LLM is the teacher, the code is what's learned", on small
  tasks.
- `2026-symtorch-symbolic-distillation.pdf`: Tan, Soubki & Cranmer, *SymTorch:
  Symbolic Distillation of Neural Networks*.
  <https://arxiv.org/abs/2602.21307>
- `2021-weiss-et-al-thinking-like-transformers-rasp.pdf`: Weiss, Goldberg &
  Yahav, *Thinking Like Transformers* (ICML 2021, RASP).
  <https://arxiv.org/abs/2106.06981>
- `2023-lindner-et-al-tracr.pdf`: Lindner et al., *Tracr: Compiled
  Transformers as a Laboratory for Interpretability*.
  <https://arxiv.org/abs/2301.05062>
- `2023-friedman-et-al-learning-transformer-programs.pdf`: Friedman, Wettig &
  Chen, *Learning Transformer Programs* (NeurIPS 2023).
  <https://arxiv.org/abs/2306.01128>
  Neural networks built so they can be converted into readable programs, or
  the reverse: the "decompile the model" path glassmind doesn't take.

## Universal grammar and cross-linguistic structure

- `2002-bender-flickinger-oepen-grammar-matrix.pdf`: Bender, Flickinger &
  Oepen, *The Grammar Matrix* (COLING Workshop on Grammar Engineering 2002).
  <https://aclanthology.org/W02-1502/>
  A shared core of grammar code, taken from the English Resource Grammar,
  that every new language's grammar starts from, adding only what is specific
  to that language. The direct model for per-language grammar crates over one
  shared base.
- `2004-ranta-grammatical-framework.pdf`: Aarne Ranta, *Grammatical
  Framework: A Type-Theoretical Grammar Formalism* (Journal of Functional
  Programming 2004). <https://www.cse.chalmers.se/~aarne/articles/gf-jfp.pdf>.
  No usable text layer.
  One typed abstract syntax holds the meaning, and each language has a
  concrete syntax mapping it to and from text, with the same grammar used to
  parse and to generate. The closest existing design to glassmind's typed
  meaning layer plus one grammar crate per language.
- `2009-ranta-gf-resource-grammar-library.pdf`: Aarne Ranta, *The GF Resource
  Grammar Library* (LiLT 2009).
  <https://journals.colorado.edu/index.php/lilt/article/view/1205>
  Grammars for many languages behind one shared interface, so application
  code is written once for all of them. Shows how much grammar can be shared
  across languages and what has to stay separate.
- `2009-angelov-incremental-parsing-pmcfg.pdf`: Krasimir Angelov,
  *Incremental Parsing with Parallel Multiple Context-Free Grammars* (EACL
  2009). <https://aclanthology.org/E09-1009/>
  GF's parser, which reads one word at a time and can list every word that
  may come next. The technique behind grammar-driven Tab completion. The
  later runtime-format paper (Angelov, Bringert & Ranta, *PGF*, JoLLI 2010,
  <https://doi.org/10.1007/s10849-009-9112-y>) has no open copy.
- `2021-de-marneffe-et-al-universal-dependencies.pdf`: de Marneffe, Manning,
  Nivre & Zeman, *Universal Dependencies* (Computational Linguistics 2021).
  <https://aclanthology.org/2021.cl-2.11/>
  One set of word classes and grammatical relations used across 100+
  languages. The main working example of a cross-linguistic inventory, and
  of where it has to give way to language-specific detail.
- `2010-haspelmath-comparative-concepts.pdf`: Martin Haspelmath,
  *Comparative concepts and descriptive categories in crosslinguistic
  studies* (Language 2010). <https://zenodo.org/records/1303024>
  A language's own categories (English "adjective") are a different kind of
  thing from the concepts used to compare languages. Supports keeping
  language-neutral meaning types separate from each grammar crate's
  categories.
- `2009-evans-levinson-myth-of-language-universals.pdf`: Evans & Levinson,
  *The myth of language universals* (Behavioral and Brain Sciences 2009,
  with peer commentaries). <https://doi.org/10.1017/S0140525X0999094X>
  The strongest case that very few grammatical features hold in every
  language. A warning against an English-shaped "neutral" layer.
- `2019-chomsky-gallego-ott-generative-grammar-faculty-of-language.pdf`:
  Chomsky, Gallego & Ott, *Generative Grammar and the Faculty of Language:
  Insights, Questions, and Challenges* (Catalan Journal of Linguistics 2019).
  <https://lingbuzz.net/lingbuzz/003507>
  A recent overview of the universal-grammar position: a small shared core of
  combining operations, with differences between languages placed in the
  lexicon and the sound system. Read it against Evans & Levinson.
- `2021-wierzbicka-semantic-primitives-fifty-years-later.pdf`: Anna
  Wierzbicka, *"Semantic Primitives", fifty years later* (Russian Journal of
  Linguistics 2021). <https://doi.org/10.22363/2687-0088-2021-25-2-317-342>
  About 65 "semantic primes" (SOMEONE, DO, WANT, BECAUSE…) claimed to exist as
  words in every language and able to define everything else. A tested
  candidate for the smallest building blocks of an interlingua.

## Language-neutral meaning representations

- `2013-banarescu-et-al-amr.pdf`: Banarescu et al., *Abstract Meaning
  Representation for Sembanking* (Linguistic Annotation Workshop 2013).
  <https://aclanthology.org/W13-2322/>
  Sentence meaning as a small graph of concepts and roles, independent of
  word order and syntax. Widely used, but English-centric.
- `2021-van-gysel-et-al-uniform-meaning-representation.pdf`: Van Gysel et
  al., *Designing a Uniform Meaning Representation for Natural Language
  Processing* (KI 2021). <https://par.nsf.gov/biblio/10288269>
  AMR extended to work across languages, adding tense, aspect, modality and
  links across a whole document, and stating where languages may differ. The
  most current design for a typed language-neutral meaning layer.
- `2005-copestake-et-al-minimal-recursion-semantics.pdf`: Copestake,
  Flickinger, Pollard & Sag, *Minimal Recursion Semantics: An Introduction*
  (Research on Language and Computation 2005).
  <https://www.cl.cam.ac.uk/~aac10/papers/mrs.pdf>
  The meaning format the English Resource Grammar and Grammar Matrix produce
  and generate from. It can leave choices such as quantifier scope open,
  which the meaning layer needs.
- `2017-abzianidze-et-al-parallel-meaning-bank.pdf`: Abzianidze et al., *The
  Parallel Meaning Bank* (EACL 2017). <https://aclanthology.org/E17-2039/>
  Translations in English, German, Italian and Dutch annotated with one
  shared formal meaning. A test of "same meaning, different language".
- `2020-vrandecic-multilingual-wikipedia.pdf`: Denny Vrandečić,
  *Architecture for a Multilingual Wikipedia* (2020).
  <https://arxiv.org/abs/2004.04733>
  Abstract Wikipedia: content stored once as language-neutral constructors
  over Wikidata items, with one renderer per language. A large current
  attempt at glassmind's architecture, in the generating direction.
- `1992-nyberg-mitamura-kant.pdf`: Nyberg & Mitamura, *The KANT System*
  (COLING 1992). <https://aclanthology.org/C92-3168/>
  Interlingua machine translation that shipped, because it was limited to
  one domain and a controlled input language. Evidence that a defined world
  makes a neutral layer much easier.
- `2013-bond-foster-open-multilingual-wordnet.pdf`: Bond & Foster, *Linking
  and Extending an Open Multilingual Wordnet* (ACL 2013).
  <https://aclanthology.org/P13-1133/>
  Wordnets in 150+ languages linked through one shared index of concepts. A
  model for word meanings each language's lexicon points to (research only:
  glassmind doesn't use WordNet data).
- `1969-schank-tesler-conceptual-dependency-parser.pdf`: Schank & Tesler, *A
  Conceptual Dependency Parser for Natural Language* (COLING 1969).
  <https://aclanthology.org/C69-0201/>
  Hand-written parsing straight into language-independent meaning, later
  reduced to a small set of action primitives. An ancestor of the meaning
  layer, showing both the appeal and the limits of a fixed primitive set.

## Ontologies

- `1993-gruber-translation-approach-ontologies.pdf`: Tom Gruber, *A
  Translation Approach to Portable Ontology Specifications* (Knowledge
  Acquisition 1993). <http://tomgruber.org/writing/ontolingua-kaj-1993.pdf>
  The standard definition of an ontology ("an explicit specification of a
  conceptualization"), and sharing one across systems by translation.
- `1998-guarino-formal-ontology.pdf`: Nicola Guarino, *Formal Ontology and
  Information Systems* (FOIS 1998).
  <http://www.loa.istc.cnr.it/old/Papers/FOIS98.pdf>
  Top-level, domain, task and application ontologies, and how each should
  depend on the others. Maps onto the meaning layer versus each world's kinds.
- `2001-niles-pease-sumo.pdf`: Niles & Pease, *Towards a Standard Upper
  Ontology* (FOIS 2001).
  <https://www.ontologyportal.org/professional/FOIS.pdf>
  SUMO: a large free top-level ontology in logic, mapped to WordNet. A worked
  example of general things, events and relations a lexicon can link into.
- `2002-gangemi-et-al-dolce.pdf`: Gangemi, Guarino, Masolo, Oltramari &
  Schneider, *Sweetening Ontologies with DOLCE* (EKAW 2002).
  <http://www.loa.istc.cnr.it/old/Papers/DOLCE-EKAW.pdf>
  A top-level ontology built around the categories of natural language and
  human thinking rather than physics. The closest one to a meaning layer
  behind language.
- `2022-otte-beverley-ruttenberg-bfo.pdf`: Otte, Beverley & Ruttenberg,
  *BFO: Basic Formal Ontology* (Applied Ontology 2022, authors' preprint).
  <https://philarchive.org/rec/OTTBBF>
  About 36 top-level classes, an ISO standard, extended by 350+ domain
  ontologies. Shows how small a shared top level can be.
- `2014-vrandecic-krotzsch-wikidata.pdf`: Vrandečić & Krötzsch, *Wikidata: A
  Free Collaborative Knowledgebase* (CACM 2014).
  <https://iccl.inf.tu-dresden.de/web/Article4002>
  Items are language-neutral IDs with a label in each language: things kept
  separate from the words for them. A likely future world.

Some of these files are the authors' preprints or accepted manuscripts
rather than the final published layout (Gruber, Guarino, Van Gysel, BFO,
Ranta 2004). The SUMO and BFO files come from the Internet Archive's saved
copies of the links above, which are behind bot checks.
