"""The word bridge: words the request-category tagger never saw, read as
words it knows that mean the same.

Two sources, both outside data:
- Wikidata: programming concepts (subroutine, member variable, function
  call, record, ...) with their English labels and aliases (CC0). Each
  concept stands for one of glassmind's words (CONCEPTS below: which
  concept is which word is the judgment made here, for the Rust world:
  Rust has no classes, and people who come from languages that do call
  structs classes).
- Open English WordNet 2025 (CC BY 4.0): words sharing a computing sense or
  one of the commonest senses.

Only words outside the tagger's vocabulary are bridged, and only to words
inside it; an inflected form goes to the same inflection of its word when
the vocabulary has that ("invoking" -> "calling", "attributes" -> "fields").

    python3 tools/datagen/wordnet_bridge.py --wordnet english-wordnet-2025.xml.gz \\
        --wikidata wikidata-programming.json [--wikidata ...] \\
        --model crates/english/models/intents.tsv --out crates/english/models/synonyms.tsv

Regenerate after retraining the tagger: its vocabulary decides the table.
The Wikidata files are `wbgetentities` results (props labels|aliases|sitelinks,
languages en, sitefilter enwiki) for the English Wikipedia titles of the
concepts below, for example:

    curl -G https://www.wikidata.org/w/api.php --data-urlencode action=wbgetentities \
        --data-urlencode sites=enwiki --data-urlencode "titles=Function (computer programming)|Member variable" \
        --data-urlencode "props=labels|aliases|sitelinks" --data-urlencode languages=en --data-urlencode format=json
"""

import argparse
import gzip
import json
import re
import xml.etree.ElementTree as ET
from collections import defaultdict

# Senses of a word counted, commonest first (WordNet lists them by
# frequency), besides those in computing.
SENSES = 3

# Wikidata concept -> the word glassmind uses for it.
CONCEPTS = {
    "Q190686": "function",     # subroutine: routine, procedure, method, ...
    "Q815844": "function",     # method (a Rust method is a function)
    "Q110940720": "call",      # subroutine call: invocation, method call, ...
    "Q2470517": "struct",      # record: struct, compound data
    "Q4479242": "struct",      # class (Rust has none; its structs are what people mean)
    "Q1172412": "field",       # field
    "Q6814247": "field",       # member variable: attribute, data member, ...
    "Q2123335": "field",       # property
    "Q760170": "enum",         # enumerated type
    "Q1971873": "trait",       # trait
    "Q7251920": "trait",       # interface: protocol, trait
    "Q1362142": "build",       # software build
    "Q47506": "build",         # compiler (what compiles is what builds)
    "Q1568346": "test",        # test case
    "Q907375": "test",         # unit testing
    "Q188522": "test",         # software testing
    "Q179550": "error",        # software bug
    "Q190087": "type",         # data type
}

# Words of a many-word alias that say nothing of the concept.
FILLER = {"a", "an", "the", "of", "in", "on", "to", "for", "computer", "computing", "programming",
          "program", "software", "data", "code", "object", "class", "instance", "unit", "process"}


def forms(lemma, pos):
    """(suffix tag, form) for the regular inflections of a lemma."""
    out = [("", lemma)]
    if " " in lemma or not lemma.isalpha():
        return out
    if pos in ("n", "any"):
        if re.search(r"[^aeiou]y$", lemma):
            out.append(("s", lemma[:-1] + "ies"))
        elif re.search(r"(s|x|z|ch|sh)$", lemma):
            out.append(("s", lemma + "es"))
        else:
            out.append(("s", lemma + "s"))
    if pos in ("v", "any"):
        if re.search(r"[^aeiou]y$", lemma):
            out += [("s", lemma[:-1] + "ies"), ("ed", lemma[:-1] + "ied"), ("ing", lemma + "ing")]
        elif lemma.endswith("e"):
            out += [("s", lemma + "s"), ("ed", lemma + "d"), ("ing", lemma[:-1] + "ing")]
        else:
            out += [("s", lemma + ("es" if re.search(r"(s|x|z|ch|sh)$", lemma) else "s")),
                    ("ed", lemma + "ed"), ("ing", lemma + "ing")]
    return out


def lemmas_of(word):
    """The lemmas a word may be a form of: itself, or without an -ing,
    -ed or -s ending (with an -e put back)."""
    out = {word}
    for suffix in ("ing", "ed", "es", "s"):
        if word.endswith(suffix) and len(word) > len(suffix) + 2:
            stem = word[: -len(suffix)]
            out |= {stem, stem + "e"}
    return out


def wordnet_pairs(path, counted):
    """(word, pos) -> the (word, pos) sharing one of its senses that count."""
    senses = defaultdict(list)
    members = defaultdict(set)
    topics = defaultdict(set)
    words_of = {}
    with gzip.open(path) as fh:
        for _, el in ET.iterparse(fh):
            if el.tag == "LexicalEntry":
                lemma = el.find("Lemma")
                word, pos = lemma.get("writtenForm").lower(), lemma.get("partOfSpeech")
                pos = "a" if pos == "s" else pos
                for sense in el.findall("Sense"):
                    senses[(word, pos)].append(sense.get("synset"))
                    members[sense.get("synset")].add((word, pos))
                el.clear()
            elif el.tag == "Synset":
                words_of[el.get("id")] = el.get("members", "")
                for r in el.findall("SynsetRelation"):
                    if r.get("relType") == "domain_topic":
                        topics[el.get("id")].add(r.get("target"))
                el.clear()
    computing = {s for s, ws in words_of.items()
                 if re.search(r"oewn-(computer_science|computing|computer_programming|programming)-n", ws)}
    near = {}
    for key, synsets in senses.items():
        chosen = [s for s in synsets if topics[s] & computing] + synsets[:counted]
        near[key] = {m for s in chosen for m in members[s] if m != key}
    return near


def wikidata_words(paths):
    """word -> the glassmind words of the concepts it names."""
    single, heads = defaultdict(set), defaultdict(set)
    for path in paths:
        for q, e in json.load(open(path)).get("entities", {}).items():
            word = CONCEPTS.get(q)
            if not word or "missing" in e:
                continue
            names = [e.get("labels", {}).get("en", {}).get("value", "")]
            names += [a["value"] for a in e.get("aliases", {}).get("en", [])]
            for name in names:
                parts = re.findall(r"[a-z]+", name.lower())
                if len(parts) == 1:
                    single[parts[0]].add(word)
                elif parts and parts[0].endswith("ing"):
                    # "invoking a function": the act, named by its verb.
                    heads[parts[0]].add(word)
                else:
                    # The head of a many-word name: before "of", else last.
                    cut = parts[: parts.index("of")] if "of" in parts else parts
                    kept = [p for p in cut if p not in FILLER]
                    if kept:
                        heads[kept[-1]].add(word)
    # A word that is a concept's own name means that concept only.
    out = defaultdict(set, single)
    for w, words in heads.items():
        if w not in single:
            out[w] |= words
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--wordnet", required=True)
    ap.add_argument("--wikidata", action="append", default=[])
    ap.add_argument("--model", required=True)
    ap.add_argument("--out", required=True)
    ap.add_argument("--senses", type=int, default=SENSES, help="WordNet senses counted besides computing ones")
    a = ap.parse_args()
    vocab = set()
    for line in open(a.model):
        f = line.rstrip("\n").split("\t")
        if f[0] == "seen" and " " not in f[1]:
            vocab.add(f[1])
    bridge = defaultdict(set)

    def add(form, tag, other, opos):
        if form in vocab:
            return
        theirs = dict(forms(other, opos))
        if theirs.get(tag) in vocab:
            bridge[form].add(theirs[tag])
        elif other in vocab:
            bridge[form].add(other)

    for (word, pos), near in wordnet_pairs(a.wordnet, a.senses).items():
        for tag, form in forms(word, pos):
            for other, opos in near:
                add(form, tag, other, opos)
    for word, targets in wikidata_words(a.wikidata).items():
        # Every form of the lemma the word is a form of ("invoking": invoke,
        # invokes, invoked, invoking).
        for lemma in lemmas_of(word):
            fs = forms(lemma, "any")
            if any(form == word for _, form in fs):
                for tag, form in fs:
                    for t in targets:
                        add(form, tag, t, "any")
    with open(a.out, "w") as fo:
        fo.write("# programming concepts from Wikidata (CC0) and words sharing a computing sense or one of "
                 f"the first {a.senses} senses in Open English WordNet 2025 (CC BY 4.0), for the tagger's "
                 "vocabulary (tools/datagen/wordnet_bridge.py)\n")
        for form in sorted(bridge):
            fo.write(f"{form}\t{','.join(sorted(bridge[form]))}\n")
    print(f"{len(bridge)} words bridged to {len(set().union(*bridge.values())) if bridge else 0} of {len(vocab)} words")


if __name__ == "__main__":
    main()
