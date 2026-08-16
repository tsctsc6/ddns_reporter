$token = ConvertTo-SecureString "xx" -AsPlainText -Force

$params = @{
    Uri         = "https://api.cloudflare.com/client/v4/accounts/xx/tokens/verify"
    Method      = 'GET'
    Authentication = 'Bearer'
    Token = $token
    ContentType = 'application/json'
}

$response = Invoke-WebRequest @params

Write-Host $response | ConvertTo-Json

Pause