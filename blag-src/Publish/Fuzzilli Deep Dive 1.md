---
staticPath: fuzzillipart1
Title: "Fuzzilli Deep Dive: part 1"
Description: A Look into Fuzzilli Internals
Date: 2024-12-13
tags:
  - Fuzzing
  - Fuzzilli
---
Fuzzilli is a JavaScript engine fuzzer written in Swift that is designed to generate semantically correct javascript. This blog series will be a deep dive into how Fuzzilli works!

# Fuzzilli Internals

Fuzzilli works by using an intermediate language called FuzzIL. This language is used to represent javascript operations. Mutations are then defined on fuzzIL that ensure that it can only generate semantically correct code. This approach works better than generating a context free javascript grammar which would generate code that has mismatched types.

# Data structures Overview

#### Opcodes

The most basic building blocks of FuzzIL are Opcodes.

```swift
num Opcode {
    case nop(Nop)
    case loadInteger(LoadInteger)
	<snip..>
    case probe(Probe)
    case fixup(Fixup)
}
```

These Opcodes represent different fundamental actions in FuzzIL.

#### Operations

The operations datatype contains opcodes. Operations also contain attributes that have values like:

```swift
struct Attributes: OptionSet {
	static let isMutable = Attributes(rawValue: 1 << 1)
	static let isCall = Attributes(rawValue: 1 << 2)
	static let isBlockStart = Attributes(rawValue: 1 << 3)
	static let isBlockEnd = Attributes(rawValue: 1 << 4)
	static let isInternal = Attributes(rawValue: 1 << 5)
	static let isJump = Attributes(rawValue: 1 << 6)
	static let isVariadic = Attributes(rawValue: 1 << 7)
	<snip..>
}
```

OptionSets in Swift are a datatype where you can set different attributes as true or false. Attributes keep track of what certain Operations do. Operations also keep track of the number of inputs and outputs an operation has.


#### Code

The code datatype contains a list of Operations. It also has a check method that tests if the program is semantically correct.


# Lift

Fuzzilli lifts FuzzIL into two formats: the first is a human-readable format. It looks like this:

```
v0 <- LoadInt '0'
v1 <- LoadInt '10'
v2 <- LoadInt '1'
v3 <- Phi v0
BeginFor v0, '<', v1, '+', v2 -> v4
	v6 <- BinaryOperation v3, '+', v4
	Copy v3, v6
EndFor
```

The second format is javascript:

```js
const v0 = 0;
const v1 = 10;
const v2 = 1;
let v3 = v0;
for (let v4 = v0; v4 < v1; v4 = v4 + v2) {
	const v6 = v3 + v4;
	v3 = v6;
}
```
# Sources

Thanks to Saelo for making an awesome learning resource for JavaScript fuzzing.

[Saelo Offensivcon slides 2019](https://saelo.github.io/presentations/offensivecon_19_fuzzilli.pdf)  
[Fuzzilli Source code](https://github.com/googleprojectzero/fuzzilli)   
[HowFuzzilliworks](https://github.com/googleprojectzero/fuzzilli/blob/main/Docs/HowFuzzilliWorks.md)   
