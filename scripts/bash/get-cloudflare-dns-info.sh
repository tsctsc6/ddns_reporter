curl -s -X GET "https://api.cloudflare.com/client/v4/zones/<ZONE_ID>/dns_records?type=A&name=ddns.yourdomain.com" \
     -H "Authorization: Bearer <YOUR_API_TOKEN>" \
     -H "Content-Type: application/json"
