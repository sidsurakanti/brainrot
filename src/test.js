let a = 1 + 2 * 3; // expect 7
let b = (1 + 2) * 3; // expect 9
let c = -a; // expect -7
let d = a + b + c; // expect 7 + 9 - 7 = 9
let e = "ha" * 3; // expect "hahaha"
let s = "hi";
let f = "hello " + "world"; // expect "hello world"

if (d < 10) {
  d = d + 1;
}

while (a <= 10) {
  a = a + 1;
  break;
}

for (let i = 0; i < 10; i = i + 1) {
  a = a + 1;
}

print();
