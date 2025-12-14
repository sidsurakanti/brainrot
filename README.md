# Overview
An interpreted language written in Rust. (active development)

### Currently supported
- Variables (`let`, assignment)
- Arithmetic expressions with precedence & unary
- String concatination and reptition
- Boolean literals (`true`, `false`)
- Comparisions
- Comments (`//`)
- `if / elif / else`
- `while` loops
- `break`, `continue`, `return`
- `for` loops (soon)
- Functions (parsing)

### Example
```javascript
let d = 9;

if (d < 10) {
  d = d + 1;
} elif (d >= 10) {
  d = d - 1;
} else {
  d = d + 20;
}
```
```javascript
// output
a: Bool(true)
b: Bool(true)
c: Bool(false)
d: Int(12)
````

```js
let a = 0;
while (a <= 10) {
  a = a + 1;
}
```

```js
a: Int(11)
```

```js
let e = "ha" * 3; // expect "hahaha"
let s = "hi";
let f = "hello " + "world"; // expect "hello world"
```

```js
e: Str("hahaha")
s: Str("hi")
f: Str("hello world")
```
