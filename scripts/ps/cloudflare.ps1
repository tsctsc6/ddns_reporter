$ZONE_ID = "xx"
$DNS_RECORD_ID = "xx"
$uri = "https://api.cloudflare.com/client/v4/zones/$ZONE_ID/dns_records/$DNS_RECORD_ID"

$ip = (Get-NetIPAddress -AddressFamily IPv6 -InterfaceAlias "以太网" -PrefixOrigin RouterAdvertisement -SuffixOrigin Random).IPAddress

curl

$params = @{
    Uri            = $uri
    Method         = 'PATCH'
    ContentType    = 'application/json'
    Authentication = 'Bearer'
    Token          = $token
    Body           = "{
        `"name`":`"xx`",
        `"type`":`"AAAA`",
        `"content`":`"$ip`",
        `"proxiable`":true,
        `"proxied`":false,
        `"ttl`":1,
        `"settings`":{},
        `"meta`":{},
        `"comment`":null,
        `"tags`":[]
    }"
}

$response = Invoke-WebRequest @params

Write-Host $response | ConvertTo-Json

Pause