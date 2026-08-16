$ZONE_ID = "xx"
$uri = "https://api.cloudflare.com/client/v4/zones/$ZONE_ID/dns_records/";

$token = ConvertTo-SecureString "xx" -AsPlainText -Force

$params = @{
    Uri            = $uri
    Method         = 'GET'
    ContentType    = 'application/json'
    Authentication = 'Bearer'
    Token          = $token
}

$response = Invoke-WebRequest @params

Write-Host $response | ConvertTo-Json

Pause