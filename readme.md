# rust-backend-scratch

A TCP echo server written in Rust from scratch — no framework, no Tokio, 
just raw Linux syscalls via `libc`. Built to understand how a backend 
server actually works below the framework layer.

## What it does

Accepts multiple client connections at once on port 8080, echoes back 
whatever each client sends. Single-threaded, non-blocking, event-driven.

## How it works

Uses an **epoll** event loop instead of one-thread-per-connection:

- Creates a non-blocking listening socket (`socket` → `bind` → `listen`)
- Registers it with an epoll instance
- One thread sleeps in `epoll_wait` until any socket has activity
- On activity: accepts new clients (registering each into epoll) or 
  reads/echoes data from existing ones
- Idles at ~0% CPU when no clients are active (sleeps, doesn't poll)

This is the same core model behind nginx and Node.js — a single thread 
handling many connections by only touching the ones that are ready.

## Run it

```bash
cargo run
```

Then in another terminal:
```bash
nc localhost 8080
```
Type anything — it echoes back. Open multiple `nc` sessions to see 
concurrent clients handled by one thread.

## Status

Working: concurrent echo over TCP.  
Next: HTTP/1.1 request parsing (GET → response).

## Built with

Rust, raw `libc` syscalls (`socket`, `bind`, `listen`, `accept`, 
`epoll_*`, `read`, `write`), `unsafe` FFI. Linux only (epoll is 
Linux-specific).