$ZONE_ID = "xx"
$DNS_RECORD_ID = "xx"
$uri = "https://api.cloudflare.com/client/v4/zones/$ZONE_ID/dns_records/$DNS_RECORD_ID"

$token = ConvertTo-SecureString "xx" -AsPlainText -Force

$params = @{
    Uri            = $uri
    Method         = 'PATCH'
    ContentType    = 'application/json'
    Authentication = 'Bearer'
    Token          = $token
    Body           = '{
        "name":"xx",
        "type":"AAAA",
        "content":"xx",
        "proxiable":true,
        "proxied":false,
        "ttl":1,
        "settings":{},
        "meta":{},
        "comment":null,
        "tags":[]
    }'
}

$response = Invoke-WebRequest @params

Write-Host $response | ConvertTo-Json

Pause