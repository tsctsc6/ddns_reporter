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

## DDNS Server Config

### Cloudflare

`token` : Open and login Cloudflare, top right corner, `My Profile`, left side, `API token`, `Create Token`, `Edit zone DNS` template, select your domain.

`zone_id` : Enter your domain page, overview, scoll down, at the down right corner, you will see the zone_id.

`dns_record_id` : Send a web request:

```bash
curl -s -X GET "https://api.cloudflare.com/client/v4/zones/<ZONE_ID>/dns_records?type=A&name=ddns.yourdomain.com" \
     -H "Authorization: Bearer <YOUR_API_TOKEN>" \
     -H "Content-Type: application/json"
```

You will get a json string. The `dns_record_id` is `result` -> `id`
