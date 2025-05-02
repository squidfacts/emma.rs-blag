---
staticPath: flare8known
Title: "Flare8: Known"
Description: A Look into Fuzzilli Internals
Date: 2025-04-26
tags:
  - flare
---
Solve time: 91 mins
Flag: You_Have_Awakened_Me_Too_Soon_EXE@flare-on.com

This challenge was a nice little ransomware decryption challenge. I completed this challenge statically which I am proud of (too lazy to install a windows VM). 

Once I unpacked the chall zip, I was presented with a folder called `Files` and a file called `UnlockYourFiles.exe`.

Running file on the binary showed me it was a windows binary.
```
UnlockYourFiles.exe: PE32 executable (console) Intel 80386, for MS Windows
```

The crux of the chall was realizing we could guess the xor key by realizing that `latin_alphabet.txt.encrypted` is 26 bytes so probably one byte for each letter of the alphabet. I was able to confirm this guess by using it the key to decrypt the rest of the file. The xor decryption routine only operated on 8 bytes at a time. Here's the xor decryption routine pseudcode:

```c
004011f0    void do_xor(int32_t arg1, int32_t arg2)

004011f0    {
004011f0        int32_t ebx;
004011f3        int32_t var_8 = ebx;
004011fc        char* ecx = nullptr;
004011fc        
00401201        while (ecx < 8)
00401201        {
00401201            ebx = ecx[arg2];
0040120f            ecx[arg1] = ROLB(ecx[arg1] ^ ebx, ecx) - ecx;
00401212            ecx += 1;
00401201        }
004011f0    }
```
 Where arg1 is the encrypted buf and arg2 is the key.

I rewrote the algorithm in python:
```python
def decrypt(buf,key):
    ecx = 0
    ebx = 0

    while ecx < 8:
        ebx = key[ecx]
        buf[ecx] = rol(buf[ecx] ^ ebx,ecx) - ecx
        ecx += 1
```

I then wrote a key search function based on the idea that we knew the original contents of `latin_alphabet.txt.encrypted`.

```python
with open("Files/latin_alphabet.txt.encrypted","rb") as file:
    bvar = file.read()
    
known_cipher = "ABCDEFGH"

def solve_for_key(buf):
    key = []
    for ecx in range(8):
        for k in range(255):
            ebx = k
            out = rol(buf[ecx] ^ ebx,ecx) - ecx
            if out == ord(known_cipher[ecx]):
                key.append(k)

    return key
key = solve_for_key(bvar)
print(key)
```

I started with lowercase "abcdefgh" but it didn't work so I swapped it for "ABCDEFGH" and then it worked. It took me a while to correctly implement rol in python. Got confused between hex and dec conversions and how to handle the high bits.

My decryption routine worked on text until it got to the encrypted images and then it would totally break. I realized it was a bug in my sub instruction implementation. The assembly sub instructions wraps around when it goes negative my python implementation did not. So I added:
```python
can =  l - ecx
if can < 0:
	can = 255 + (l - ecx)
out.append(can)
```

Which was still wrong! It should be 256.

```python
can =  l - ecx
if can < 0:
	can = 256 + (l - ecx)
out.append(can)
```

And then I could decrypt the images as well!

## Final solve script

```python
import glob
with open("Files/latin_alphabet.txt.encrypted","rb") as file:
    bvar = file.read()

print(len(bvar))

known_cipher = "ABCDEFGH"



def rol(val,shift):
    return (val << shift | ((val >> (8 - shift)) )) & 0xFF

def decrypt(buf,key,length=8):
    out = []
    ecx = 0
    ebx = 0

    while ecx < length:
        ebx = key[ecx]
        l = rol(buf[ecx] ^ ebx,ecx)
        can =  l - ecx
        if can < 0:
            can = 256 + (l - ecx)
        out.append(can)

        ecx += 1

    return out


assert rol(0x26,4) == 0x62


def solve_for_key(buf):
    key = []
    for ecx in range(8):
        for k in range(255):
            ebx = k
            out = rol(buf[ecx] ^ ebx,ecx) - ecx
            if out == ord(known_cipher[ecx]):
                key.append(k)

    return key

def apply_key(buf,key):
    no_bufs = len(buf) // 8
    out = []
    for i in range(no_bufs):
        start = i*8
        end = i*8+8
        s = buf[start:end]
        p = decrypt(s,key)
        out.extend(p)
    

    r_len = (len(buf) // 8) *8
    l = len(buf)
    # decrypt last bit
    if r_len != l:
        diff = l- r_len
        s = buf[r_len:]
        p = decrypt(s,key,diff)
        out.extend(p)

    return out

key = solve_for_key(bvar)
print("keylen:",len(key))

def decrypt_file(path):
    with open(path,"rb") as file:
        bvar = file.read()

    decrypted_path = path[:-10]

    pvar = apply_key(bvar,key)

  

    with open(decrypted_path,"wb") as file:
        file.write(bytes(pvar))



for file in glob.glob("Files/*.encrypted"):
    print(file)
    decrypt_file(file)
```

## Bling Bling

```
> python3 solve.py
26
keylen: 8
Files/critical_data.txt.encrypted
Files/cicero.txt.encrypted
Files/commandovm.gif.encrypted
Files/capa.png.encrypted
Files/flarevm.jpg.encrypted
Files/latin_alphabet.txt.encrypted
> cat critical_data.txt
(>0_0)> You_Have_Awakened_Me_Too_Soon_EXE@flare-on.com <(0_0<)
```
