# lulz

`lulz` is a **JIT compiler for [LOLCODE](http://www.lolcode.org/)** written in RPython.
It is **much faster than [lci](https://github.com/justinmeza/lci)**. It even performs 
better than CPython in many microbenchmarks.

**also, it...**

- tries very hard to conform to the [**LOLCODE spec**](https://github.com/justinmeza/lolcode-spec), 
  with some additional (backwards compatible) features
- has **additional extensions** (e.g., `RANDOM`, `ARGV`, and more)
- has beautiful and informative **rust/clang-like-errors**

49 tests working
[good resource](https://tratt.net/laurie/blog/entries/fast_enough_vms_in_fast_enough_time.html)
