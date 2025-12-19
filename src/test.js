// %%%%% arithmetic %%%%%
let a = 1 + 2 * 3; // expect 7
let b = (1 + 2) * 3; // expect 9
let c = -a; // expect -7
let d = a + b + c; // expect 7 + 9 - 7 = 9
let e = "ha" * 3; // expect "hahaha"
let s = "hi"; // expect "hi"
let f = "hello " + "world"; // expect "hello world"

// %%%% comparisions %%%
let a = true;
let b = false;
let c = a == "2"; // expect false
let d = a == b; // expect false

// %%%%%% if statements %%%%%
let a = 13;

if (a <= 10) {
  a = a + 1;
} elif (15 > a) {
  a = a - 1;
} elif (a == 15) {
  a = 0;
} else {
  a = a + 20;
}

// %%%%%%% while %%%%%%%%%%
let a = 0;
while (a <= 10) {
  a = a + 1;
  if (a > 5) {
    break;
  }
}


// %%%%% for %%%%%%%%%
let a = 20;

for (let i = 0; i < 10; i = i + 1) {
  a = a + 1;
}

// not implemented 
print();
