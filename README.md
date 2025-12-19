# Overview
An interpreted language written in Rust. (active development)

### Currently supported
- REPL
- Variables (`let`, assignment)
- Arithmetic expressions with precedence & unary
- String concatination and reptition
- Boolean literals (`true`, `false`)
- Comparisions
- Print
- Comments (`//`)
- `if / elif / else`
- `while` loops
- `break`, `continue`, `return`
- `for` loops 
- Functions (wip)

### Usage
```bash
git clone https://github.com/sidsurakanti/brainrot.git
cd brainrot
cargo build
cargo run
```
```bash
[BRAINROT] REPL
<Ctrl-C> to quit.
>>> print("hello, world!");
hello, world!
>>> let a = 10;
>>> print(a);
10
>>> print(10 * 30 + (2 / 2));
301
```
```bash
>>> let a = true;
>>> let b = false;
>>> print(a == b);
false
>>> print(b == false);
true
>>> print(a == "2");
false
```
```sh
>>> let a = 0;
>>> while (a <= 10) {
...     a = a + 1;
...     if (a > 5) {
...             break;
...     }
... }
...
...
>>> print(a);
6
```
```sh
>>> let a = 0;
>>> for (let i = 0; i < 10; i = i + 1) {
...     a = a + 1;
... }
...
...
>>> print(a);
10
```
```sh
>>> let a = 13;
>>> if (a <= 10) {
...     a = a + 1;
... } elif (15 > a) {
...     a = a - 1;
... } else {
...     a = 20;
... }
...
>>> print(a);
12
```
