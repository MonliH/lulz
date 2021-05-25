<div align=center>
    <h1>lulz <kbd>🐱</kbd></h1>
    a <b><a href="http://www.lolcode.org/">LOLCODE</a> compiler</b> written in rust.
</div>

<br>
<br>

## design

* Compiles to assembly, using C as a compiler backend

<br>

## installation 📦

Currently, building form source is the only way to install the compiler.

### requirements

- [cmake](https://cmake.org/)
- [rust (cargo)](https://www.rust-lang.org/tools/install)
- `make` and a c compiler, should come with most unix systems

### building

```bash
git clone https://github.com/MonliH/lulz.git
cd lulz
cmake .
make
make install  # may need sudo
```

<br>

### usage

```bash
lulz tests/io/input.lol  # compiles `test/io/input.lol`
./lol.out  # run the compiled executable
```

For more options,

```bash
lulz --help
```

<br>

### changing the install directory

```bash
cmake . -DCMAKE_INSTALL_PREFIX=<output_directory>
make
make install
```

Then, when running the `<output_directory>/bin/lulz` executable:

```bash
<output_directory>/bin/lulz filename -- -I<output_directory>/include -I<output_directory>/lib
```
