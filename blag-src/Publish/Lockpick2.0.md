---
staticPath: lockpick2
Title: "Hack the Box: LockPick 2.0"
Description: Analysis of a UPX protected ransomware sample
Date: 2024-11-27
tags:
  - Malware
  - HackTheBox
---
This is an analysis of the lockpick 2.0 sherlock from Hack the Box. The provided binary is called update. Running file on the binary returns:

```
update: ELF 64-bit LSB shared object, x86-64, version 1 (SYSV), statically linked, no section header
```

Which is odd because usually it tells you a bit more information. When opening the file in binary ninja I can see the string UPX. 

![[Pasted image 20241128073924.png]]


It also detects a few functions which also suggests that the sample is packed.
![[Pasted image 20241128073839.png]]

To unpack upx you can download the [upx binary](https://github.com/upx/upx/releases/download/v4.2.4/upx-4.2.4-amd64_linux.tar.xz) and run the following command.

```
upx -d updater
```

Now I have an unpacked version of the binary. Opening the binary again in binary ninja shows more functions!

![[Pasted image 20241128074223.png]]

Examining the encrypt_file function it looks like they are using aes 256 cbc encryption.

![[Pasted image 20241127141342.png]]


 Running the sample in gdb and breaking on the get_key_from_url gives me the url of the key used for encryption.
```
b  get_key_from_url
```

![[Pasted image 20241127141604.png]]

Finally we can decrypt the encrypted files using the key and iv from the downloaded file. The below screenshot shows how the first 32 bytes of the file are memcopied into a variable and the other half is used as an IV.


![[Pasted image 20241127143642.png]]

The following script decrypt the files.

```python
import os

# AES 256 cbc

from Crypto.Cipher import AES
from Crypto.Random import get_random_bytes
from Crypto.Util.Padding import pad, unpad


def decrypt(ciphertext, key, iv):
    cipher = AES.new(key, AES.MODE_CBC, iv)
    decrypted_text = cipher.decrypt(ciphertext)
    return decrypted_text

key_material = open("updater","rb").read()

key = key_material[0:32]
iv = key_material[32:]

for filename in os.listdir("."):
    if "24bes" in filename:
        print("decrypting", filename)
        ciphertext = open(filename,"rb").read()
        out = decrypt(ciphertext, key,iv)
        open(filename[:-6],"wb").write(out)
```

# Final Thoughts

If a sample is using unmodified upx it is straightforward to unpack. Once unpacked, the sample was a simple ransomeware malware that used the same key and IV for every file. The key and IV were downloaded from the domain found in the binary.