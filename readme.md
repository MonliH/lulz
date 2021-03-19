# lulz

lulz is a [LOLCODE](http://www.lolcode.org/) **compiler** written in rust.

**It...**

- compiles to **assembly** (via llvm) meaning your code will run faster than [lci](https://github.com/justinmeza/lci)!
- tries very hard to conform to the [**LOLCODE spec**](https://github.com/justinmeza/lolcode-spec)
    - however, we added some additional features
- has **additional extensions** (`RANDOM`, `ARGV`, and more)
- has **good rust/clang-like-errors**
    - typechecking at compile-time
