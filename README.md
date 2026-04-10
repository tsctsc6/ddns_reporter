# ddns_reporter

A software that sends the local machine's IPv6 address (temporary address preferred) to a DDNS server.

## Supported

OS:

- Windows
- GNU/Linux

DDNS server:

- Cloudflare

## Working Principle

Monitor the operating system's network events, including new network added and ipv6 address changed. Then choose a temporary address and send it to DDNS server.
