# Health check script for New Loka server
param(
    [string]$Url = "http://localhost:8080/health",
    [int]$TimeoutSeconds = 10
)

try {
    $response = Invoke-RestMethod -Uri $Url -TimeoutSec $TimeoutSeconds
    Write-Host "Status: $($response.status)"
    Write-Host "Version: $($response.version)"
    Write-Host "Node: $($response.node_id)"
    Write-Host "Tier: $($response.tier)"
    exit 0
} catch {
    Write-Error "Health check failed: $_"
    exit 1
}
