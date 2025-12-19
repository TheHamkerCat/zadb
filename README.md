## Why?

Android 11+ randomizes ADB wireless ports for "security" 

While the pairing-code provides actual security. The two random ports (pair & connection) 
just force developers to waste time on every connection while providing zero protection
against anyone who can run a 5-second port scan.

This is what happens when product managers mistake inconvenience for security.
I'm still trying to understand why google thought this is a good idea.

## Why are you so annoyed by it?

1. My keyboard doesn't have a numpad on the right
2. You have to enter 2 port numbers, not 1
3. You have to run 2 separate commands, adb pair and adb connect
4. Because it saves atleast 5 seconds.

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

Example:

```
❯ zadb x15 460835
Scanning ports on x15...
[OK] Found open ports: [37065, 44469]
Trying to pair on all opened ports...
[OK] Paired successfully on port 37065
Trying to connect on remaining ports...
[OK] Connected on port 44469
[OK] Done! Device connected on x15:44469
```