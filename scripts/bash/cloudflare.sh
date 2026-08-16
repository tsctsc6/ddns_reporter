# DNS_RECORD_ID from get-cloudflare-dns-info.sh, result -> id
curl -X PATCH "https://api.cloudflare.com/client/v4/zones/<ZONE_ID>/dns_records/<DNS_RECORD_ID>" \
     -H "Authorization: Bearer <YOUR_API_TOKEN>" \
     -H "Content-Type: application/json" \
     -d '{
       "type": "AAAA",
       "name": "ddns.yourdomain.com",
       "content": "<IP_ADDRESS>",
       "ttl": 60,
       "proxied": false
     }'
