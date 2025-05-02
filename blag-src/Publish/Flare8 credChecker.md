---
staticPath: flare8credchecker
Title: "Flare8: Credchecker"
Description: A Look into Fuzzilli Internals
Date: 2025-04-26
tags:
  - flare
---
Solve time: 25 mins
Flag: enter_the_funhouse@flare-on.com

Credchecker was a straightforward javascript reversing chall. Definetly a warm up. After unpacking the chall zip, we are presented with an html file called `admin.html` and a folder called `img`. Inside the `img` directory are two images one is an image of a golden ticket and one is the Flare On logo.  When loading the html page in a browser is a form that asks for a username and password. Trying random creds presents me with an error message.


![[Pasted image 20250502072407.png]]

Love the snark ("If you continue to fail, please ask your parents if it is too late to change your major"). Looking at the admin.html, I see a simple string comparison for the password check function.

```javscript
username.value == "Admin" && atob(password.value) == "goldenticket"
```

Now I know the username! I just need to base64 encode the string "goldenticket" and we win. `Z29sZGVudGlja2V0` is the encoding of golden ticket. 

## Bling Bling

![[Pasted image 20250502072648.png]]