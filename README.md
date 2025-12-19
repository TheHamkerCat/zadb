## Why?

Android 11+ randomizes ADB wireless ports for "security" 

While the pairing-code provides actual security. The two random ports (pair & connection) 
just force developers to waste time on every connection while providing zero protection
against anyone who can run a 5-second port scan.

This is what happens when product managers mistake inconvenience for security.

## Build & Install

```bash
git clone https://github.com/thehamkercat/zadb && cd zadb
cargo build --release
sudo cp target/release/zadb /usr/local/bin/
```

## Usage

```bash
zadb <ip|hostname> <pairing-code>
```