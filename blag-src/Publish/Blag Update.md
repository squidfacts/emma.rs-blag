---
staticPath: blagupdate
Title: Blag Update
Description: An update on open sourcing my blag
Date: 2025-05-02
tags:
  - blag
---
Hi Y'all,

I've now open sourced my blag! It's on [github](https://github.com/squidfacts/emma.rs-blag). All I have to do now is run `cargo run` and it transforms my obsidian vault located in `blag-src` into `.mdx` pages ready to be consumed by [code hike](https://codehike.org/) which then gets generated into html and react js. The build tool then cleans my s3 bucket (no more unused files hanging around). It then uploads `emma.rs/out` to the s3 bucket. Then a cloudfront invalidation is created to make the cdn purge its cache.

Overall, very happy with my blag pipeline. It has very little friction. I just paste images and text into obsidian and then pipeline take it from there. I might one day make it all run in github actions. But, to be honest github actions are [hell](https://www.youtube.com/watch?v=9qljpi5jiMQ). And there's not really an upside vs running locally. Also my aws creds are scoped tightly to just the s3 bucket and cloudfront invalidations (limiting blast radius if my malware analysis host gets compromised).

My blag setup probably isn't usable by other people (yet). But, I hope it can be used as inspiration to others. It's all fairly straightforward (except interacting with npm LOL). 

Happy Blagging,
Emma