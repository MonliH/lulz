# lulz

lulz is a [LOLCODE](http://www.lolcode.org/) **interpreter** written in rust.

**design**

- **fast**, because it compiles to a bytecode (run on the lolvm)
- lulz won't give you random **segfaults** like [lci](https://github.com/justinmeza/lci/issues/55) 
  [does](https://github.com/justinmeza/lci/issues/54) 
  [a](https://github.com/justinmeza/lci/issues/47) 
  [lot](https://github.com/justinmeza/lci/issues/49)
- tries very hard to conform to the [**LOLCODE spec**](https://github.com/justinmeza/lolcode-spec), with some extra features
- has **additional extensions** (`RANDOM`, `ARGV`, and more)
- has **good rust/clang-like-errors**
