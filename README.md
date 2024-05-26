# GRA

GRA's not really anything right now, but a work in progress view of a different way to compute, and an opportunity to explore some ideas, and also do learn/practice some Rust.

Also expect a lot of ramblings. 

## Why?

Really, it's about stopping, reflecting on the last 50 years of computing, and asking, "What if we did it now, with what we know now?", with a heavy bias towards my enjoyment of development.

I'm not sure where it's going, but I'm excited to see where it ends up. In all honesty, I'm not too bothered, I think it's pretty nifty, and fun to just explore and tinker.

Something I've experienced in the past - and try to challenge myself on - is with incremental change, one's unlikely to end up at the same place, as restarting from scratch, which can be a good thing.

## What?
The goal is really exploration, but as it stands the design, into a unified access layer, that can be used to interact with a variety of data sources, and compute resources.

The core concept would be an auto-tiered storage system, where the bottom tier is the network, using a combination of supernodes, and gossip.

My Requirements:
* Data agnosticism
* Language agnosticism
* Resource agnosticism
* Everything in userspace
* Everything is Async
* Everything is a file, including directories.
  - This includes the ability to execute directories, and have them run as a program, via a runtime.

And trying to bring them together in to a cohesive system.

In my head it shares a lot of concepts, with that of IPFS, but nuanced, for example taking POSIX disallowed filename patterns '\' & '\0', and using them for namespace separation.
    - Hopefully, this will help both mapping and understanding the system, and also provide a way to interact with the system in a more human way.

A weird mix of ideas, but let's see where it goes.

## Status
Very much not working, but many of the core concepts are in place. Hopefully, they'll start to come together soon.

Libp2p became a bit of a nightmare/rabbit hole. I'm going to try and simplify the networking stack, and see if I can get it working.

### Requirements
* Rust
* Cargo
* A Unix-like system - for now.


### Getting Started
```bash
git clone https://github.com/nanoly-cloud/gra.git
cd gra
cargo install --path .
```


### Troubleshooting
If you're having issues with the build, try running the following command:
```bash
GRA_LOG=debug cargo build
```


### Notes
* This is a work in progress, and is very obviously not working, and not ready for production use.
* This is a personal project, but it's copyrighted to nanoly.

TODO:
* [ ] Implement autotiered storage
* [ ] Implement the network stack
* [ ] Implement the fuse filesystem
* [ ] Implement the wasi runtime integration
* [ ] Remove Daemon 
