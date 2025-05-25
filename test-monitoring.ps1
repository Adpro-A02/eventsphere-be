Write-Host "🔍 Testing EventSphere Monitoring Endpoints..." -ForegroundColor Green
Write-Host ""

$healthCheckUrl = "http://localhost:8000/health"
Write-Host "Checking if application is running at $healthCheckUrl..." -ForegroundColor Yellow

try {
    $healthCheck = Invoke-RestMethod -Uri $healthCheckUrl -Method Get -ErrorAction Stop
    Write-Host "✅ Application is running!" -ForegroundColor Green
    Write-Host "   Version: $($healthCheck.version)" -ForegroundColor Cyan
    Write-Host "   Status:  $($healthCheck.status)" -ForegroundColor Cyan
    Write-Host "   Uptime:  $($healthCheck.uptime) seconds" -ForegroundColor Cyan
    Write-Host ""
} catch {
    Write-Host "❌ Application doesn't appear to be running. Start it with ./start-dev.ps1" -ForegroundColor Red
    exit 1
}

Write-Host "Testing detailed health endpoint..." -ForegroundColor Yellow
try {
    $detailedHealth = Invoke-RestMethod -Uri "http://localhost:8000/health/detailed" -Method Get -ErrorAction Stop
    Write-Host "✅ Detailed health check successful" -ForegroundColor Green
    Write-Host "   Status: $($detailedHealth.status)" -ForegroundColor Cyan
    foreach ($service in $detailedHealth.services) {
        $statusColor = if ($service.status -eq "ok") { "Green" } else { "Red" }
        Write-Host "   - $($service.name): " -NoNewline
        Write-Host "$($service.status)" -ForegroundColor $statusColor
    }
    Write-Host ""
} catch {
    Write-Host "❌ Detailed health check failed: $_" -ForegroundColor Red
    Write-Host ""
}

Write-Host "Testing metrics endpoint..." -ForegroundColor Yellow
try {
    $metrics = Invoke-RestMethod -Uri "http://localhost:8000/metrics" -Method Get -ErrorAction Stop
    Write-Host "✅ Metrics endpoint responding" -ForegroundColor Green
    
    $metricCount = ($metrics -split "`n" | Where-Object { $_ -match "^[a-zA-Z]" }).Count
    Write-Host "   Found $metricCount metrics" -ForegroundColor Cyan
    
    Write-Host ""
    Write-Host "Sample metrics:" -ForegroundColor Yellow
    $httpRequestsLine = $metrics -split "`n" | Where-Object { $_ -match "^http_requests_total" } | Select-Object -First 1
    if ($httpRequestsLine) {
        Write-Host "   $httpRequestsLine" -ForegroundColor White
    }
    Write-Host ""
} catch {
    Write-Host "❌ Metrics endpoint failed: $_" -ForegroundColor Red
    Write-Host ""
}

Write-Host "Testing Prometheus connection..." -ForegroundColor Yellow
try {
    $prometheus = Invoke-RestMethod -Uri "http://localhost:9090/-/healthy" -Method Get -ErrorAction Stop
    Write-Host "✅ Prometheus is healthy" -ForegroundColor Green
    Write-Host ""
} catch {
    Write-Host "❌ Prometheus connection failed. Is it running?" -ForegroundColor Red
    Write-Host ""
}

Write-Host "Testing Grafana connection..." -ForegroundColor Yellow
try {
    $grafana = Invoke-RestMethod -Uri "http://localhost:3001/api/health" -Method Get -ErrorAction Stop
    Write-Host "✅ Grafana is healthy: $($grafana.database)" -ForegroundColor Green
    Write-Host ""
} catch {
    Write-Host "❌ Grafana connection failed. Is it running?" -ForegroundColor Red
    Write-Host ""
}

Write-Host "Testing AlertManager connection..." -ForegroundColor Yellow
try {
    $alertmanager = Invoke-RestMethod -Uri "http://localhost:9093/-/healthy" -Method Get -ErrorAction Stop
    Write-Host "✅ AlertManager is healthy" -ForegroundColor Green
    Write-Host ""
} catch {
    Write-Host "❌ AlertManager connection failed. Is it running?" -ForegroundColor Red
    Write-Host ""
}

Write-Host "🎉 Monitoring endpoints test completed!" -ForegroundColor Green
Write-Host ""
Write-Host "📈 Access your monitoring stack:" -ForegroundColor Cyan
Write-Host "   • Grafana:      http://localhost:3001 (admin/admin123)" -ForegroundColor White
Write-Host "   • Prometheus:   http://localhost:9090" -ForegroundColor White
Write-Host "   • AlertManager: http://localhost:9093" -ForegroundColor White
Write-Host "   • App Health:   http://localhost:8000/health" -ForegroundColor White
Write-Host "   • App Metrics:  http://localhost:8000/metrics" -ForegroundColor White
