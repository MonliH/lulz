# lulz

`lulz` is a **[LOLCODE](http://www.lolcode.org/) interpreter** written in rust.
It is **much faster than [lci](https://github.com/justinmeza/lci)** because it compiles 
your LOLCODE code to `lolbc` (bytecode) that runs on the `lolvm`.

**also, it...**

- won't give you random **segfaults** like [lci](https://github.com/justinmeza/lci/issues/55)
  [does](https://github.com/justinmeza/lci/issues/54)
  [a](https://github.com/justinmeza/lci/issues/47)
  [lot](https://github.com/justinmeza/lci/issues/49)
- tries very hard to conform to the [**LOLCODE spec**](https://github.com/justinmeza/lolcode-spec), 
  with some additional (backwards compatible) features
- has **additional extensions** (e.g., `RANDOM`, `ARGV`, and more)
- has beautiful and informative **rust/clang-like-errors**
