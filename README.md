# glassmind

A system you talk to in plain English about data it can see, that answers in
English, with no neural network inside it. Everything it knows is Rust source
you can read: its types are the concepts it can think about, its functions are
how it reasons, and every answer can show the trace that produced it.

LLM coding agents build it, but none runs inside it. They write its grammar,
words and concepts, generate the conversations it learns from, and improve it
in a loop that keeps a change only when an automatic scorer says it helps.

The first world is Rust source code; the first language is English.

See `prompts/symbolic-language-model.md` for the brief and
`references/README.md` for related work.
