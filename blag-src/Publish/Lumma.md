---
staticPath: lumma
Title: Github Scanner Malware Analysis
Date: 2024-12-13
Description: Using dynamic analysis to analyze a Lumma sample.
tags:
  - Malware
---

# Github Scanner Malware

## Malware Origin

Someone in a Discord server shared the following email.

![[lumma1.png]]

The message encourages users to visit `hxxps[://]github-scanner[.]com`.

## Site

When someone visits the website, they are prompted to run a command to "prove they're human".

![[lumma2.png]]

When clicked, the site prompts the user to run the contents of their clipboard.

![[lumma3.png]]

The site uses the following JavaScript to copy commands to the clipboard.

![[lumma4.png]]

## PowerShell script

When the executable runs, it downloads another executable to temp.

![[lumma5.png]]


## The first binary


Opening the executable in Ghidra and IDA Free yielded little information. Examining it further, I realized it jumps to a DLL called `MSCOREE.DLL::_CorExeMain` which is a .NET runtime DLL. I then opened it in [DnSpyEx](https://github.com/dnSpyEx/dnSpy). The .NET program contained these large buffers which I suspected were encrypted payloads. 

![[lumma6.png]]

This was later confirmed when I found the decryption routines.


![[lumma7.png]]

Examining the main method, it decrypts a buffer and then calls `CallWindowProcW` on it. The W at the end indicates the method takes a wide character string. From the [Microsoft Docs](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-callwindowprocw) the function "Passes message information to the specified window procedure." Here it is used as a sneaky way to execute the shellcode which was allowed to be executed because of the previous call to `VirtualProtect`.

![[lumma8.png]]

### Malware authors hate this one weird trick

I then inserted a call to `File.WriteAllBytes` before the shellcode was executed, so I could do further analysis of the sample. DnSpy makes this pretty simple to do via their `edit code` functionality.

![[lumma9.png]]

## The second binary

Opening the second binary in Ghidra and Ida, it looked heavily obfuscated.

![[lumma10.png]]

So I opted to use [x64dbg](https://x64dbg.com/) to do dynamic analysis of the sample.

![[lumm11.png]]

When running the sample, I got the following window. This clued me in to the fact that I was dealing with a Lumma sample. I then spent a couple of hours manually stepping over the assembly, setting breakpoints and labeling functions. I was able to see some of the Lumma sample's info-stealing behaviour.


#### Info stealing

The malware queries system drives.

![[systemdrives.png]]

The malware queries hardware configuration.

![[idk.png]]


The malware queries BIOS information.

![[sys32.png]]

The malware retrieves Edge user data.

![[edge.png]]



#### C2 parameter

The malware makes a POST request to `/api` with the parameters `act=life`.

![[c2params.png]]

#### Disclaimer

I definitely missed a lot when doing my dynamic analysis, so this is only a subset of the Lumma functionality.

## Big-Brained Dynamic Analysis

Thanks to [Herrcore](https://x.com/herrcore) who gave me some tips, I was able to reverse more of the Command and Control (C2) communications of the sample. He showed me how to set a breakpoint on all `winhttp.dll` functions and then log the contents of the stack when it's called. 

![[lumma18.png]]

The first breakpoint the sample hits is in the loader, so I skip over it using the play button. 

![[lumma19.png]]

The second breakpoint actually breaks in the malware sample. This sample contains some anti-debugging features. So I set a breakpoint where it attempts to exit.

![[lumma20.png]]

Let's set the `z` flag (which controls jump conditions) to 0 so it doesn't take the jump and exit.

![[lumma23.png]]


I then set x64dbg to break on user and system DLL loads by going to Options -> Preferences.

![[lumma28.png]]



The next load breaks on `winhttp.dll`. Excellent. I can then set a breakpoint on all methods in that module. 


![[lumma26.png]]

I can then disable breaking on DLL loads because I've set breakpoints on the target DLL.

![[lumma27.png]]

The next breakpoint is the entrypoint of `winhttp.dll`, so I know it's working.

![[lumma29.png]]

I can then set a logging breakpoint on `WinHTTPCrackUrl`.

![[lumma30.png]]

Using the command `Log "Url: {s:[ESP+4]}` I log as a string the contents of the stack pointer plus 4, which is the location of the C2 URL.

![[lumma31.png]]

I get a list of C2s to use as indicators of compromise pretty painlessly.

![[lummaurls.png]]


```
Log "URL: L"hxxps[://]keennylrwmqlw[.]shop/""
Log "URL: L"hxxps[://]licenseodqwmqn[.]shop/""
Log "URL: L"hxxps[://]tendencctywop[.]shop/""
Log "URL: L"hxxps[://]tesecuuweqo[.]shop/""
Log "URL: L"hxxps[://]relaxatinownio[.]shop/""
Log "URL: L"hxxps[://]reggwardssdqw[.]shop/""
Log "URL: L"hxxps[://]eemmbryequo[.]shop/""
Log "URL: L"hxxps[://]tryyudjasudqo[.]shop/""
Log "URL: L"hxxps[://]steamcommunity[.]com/profiles/76561199724331900""
Log "URL: L"hxxps[://]steamcommunity[.]com/""
```