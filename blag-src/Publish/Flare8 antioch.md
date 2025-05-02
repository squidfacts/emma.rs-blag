---
staticPath: flare8antioch
Title: "Flare8: Antioch"
Description: My write up for the Flare On 8 challenge called Antioch
Date: 2025-04-27
tags:
  - CTF
  - Flare
---
Attempt time: 340 mins  
Flag: N/a

So, I didn't actually solve this challenge. I ended up taking a look at writeups, I got stuck trying to brute force a crc32 check. Overall, I learned a ton from this challenge and I don't view it as a failure even though I didn't solve it. It was definitely a learning experience.

The challenge began with a tar file called `Antioch.tar`. Once I unpacked the tar file, I realized it was a docker container. Eventually I realized I could just load the docker container using the `docker load --input antioch.tar` command. Once I ran the docker container there was a prompt of:

```
AntiochOS, version 1.32 (build 1975)
Type help for help
>
```

It wasn't clear what commands I could run. By inspecting the binary I identified the following commands:
- quit
- approach
- consult

The approach command prompted me for my name. The consult command just printed a ton of VVVs and then prompted for another command again. What felt like a good place to start was understanding how the name check was working.

![[Pasted image 20250502100023.png]]

Above is the disassembly. After reading some write-ups, I now know that this is a crc32 hash function. Not having a clear way to reverse the function. I simply implemented it in python and then brute forced it with the names I found in the docker layers. Each docker layer had an author with a Monty Python character associated with it. I was successfully able to find the hash of `Bridge Keeper`. I failed to realize that all of the Monty Python names were valid. I also got stuck on brute forcing the correct color for each name.

## Correct Solution (from writeups)

The steps after where I got stuck was
1. Brute force the color for each name
2. Note the number outputted for each character
3. Layer the docker images in the order of the numbers
4. Run the `consult` command with the correctly ordered layers
5. Get flag

I suggest [0xdf's writeup](https://0xdf.gitlab.io/flare-on-2021/antioch) if you want more details!

## Lessons Learned
- I now know what a crc32 hash looks like
- I think hunting more for where data was coming from would be helpful
- This was my first challenge using Cutter so I spent a lot of time googling instructions
- Not relying on a decompiler helped me learn a lot more about reverse engineering
	- It also led to some confusion when tracing data flow