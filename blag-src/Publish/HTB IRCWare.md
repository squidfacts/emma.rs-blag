---
staticPath: htbircware
Title: "HTB: IRCWare"
Description: Reverse engineering a simple key verification function
Date: 2025-05-05
tags:
  - HackTheBox
---
Solve time: 146 mins
Flag: `HTB{m1N1m411st1C_fL4g_pR0v1d3r_b0T}`

IRCWare was a nice little reversing challenge. The challenge description was:

```
During a routine check on our servers we found this suspicious binary, although when analyzing it we couldn't get it to do anything. We assume it's dead malware, but maybe something interesting can still be extracted from it?
```

It's a linux binary:

```
% file ircware
ircware: ELF 64-bit LSB executable, x86-64, version 1 (SYSV), dynamically linked, interpreter /lib64/ld-linux-x86-64.so.2, stripped
```

After playing with the binary in a docker container for a bit I realized it needed a key.

![[Pasted image 20250505061253.png]]

Here's what the key verification function looks like:
![[Pasted image 20250505061316.png]]

0x601159 is 8, so we know it loops over 8 bytes. It then does an uppercase check 0x41-0x5a is the range of uppercase letters. Does some math to `al` and then compare the value of `al` to the value pointed to by `rdi` which is the string `RJJ3DSCP`. I wrote a little key brute script to find all the letters.

```python
def verify(c,i):
    val = ord(c)

    if 0x41 <= val and val <= 0x5a:
        val += 0x11
        if val - 0x5a <= 0:
            if val == ord(known_str[i]):
                print(c,end="")
        else:
            val -= 0x5a
            val += 0x40
            if val == ord(known_str[i]):
                print(c,end="")
```

Which outputted `ASSMBLY`. I then realized that it was impossible to reach the 4th rdi char `3` just using uppercase ascii letters. 

![[Pasted image 20250505061819.png]]

If you look closely at the range check the `cmp al, 0x41` and `cmp al, 0x5a` it doesn't actually bail out. It just goes directly to the keycheck. Which means if the char isn't uppercase ascii it just gets checked directly so the 4th char is `3` which maps directly to the target strings `3`.

Yielding:
![[Pasted image 20250505062019.png]]


# Takeaways
- `syscall 318` is getrandom
- `syscall 60` is exit