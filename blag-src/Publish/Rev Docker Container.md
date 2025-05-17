---
staticPath: dockercontainer
Title: Rev Docker Container
Description: " A linux container for dynamic anslysis"
Date: 2025-05-04
tags: []
---
I frequently run into the issue of having to reinstall dynamic analysis tools into a docker container. So I created a Dockerfile to fix this.

Dockerfile:
```Dockerfile
from ubuntu

RUN apt update
RUN apt install -y file curl python3 build-essential gdb vim strace
RUN bash -c "$(curl -fsSL https://gef.blah.cat/sh)"
RUN echo "alias gdb='LC_CTYPE=C.UTF-8 gdb'" >> ~/.bashrc
```

Features:
- gdb 
- gef (with correct locale)
- strace

Usage:
```
# build container
> docker build -t rev .
# run container with pwd mounted at /chall
> docker run -it -v .:/chall rev
```