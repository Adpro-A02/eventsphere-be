Write-Host "🚀 Starting EventSphere Development Environment..." -ForegroundColor Green

if (-not (Test-Path ".env")) {
    Write-Host "📄 Creating .env file from .env.example..." -ForegroundColor Yellow
    Copy-Item ".env.example" ".env"
    Write-Host "✅ Please review and update .env file with your configurations" -ForegroundColor Green
}

Write-Host "🐳 Starting Docker containers..." -ForegroundColor Blue
docker-compose up -d

Write-Host "⏳ Waiting for services to start..." -ForegroundColor Yellow
Start-Sleep -Seconds 10

Write-Host "🔍 Checking service status..." -ForegroundColor Blue
docker-compose ps

Write-Host ""
Write-Host "🎉 EventSphere Development Environment is ready!" -ForegroundColor Green
Write-Host ""
Write-Host "📊 Access your services:" -ForegroundColor Cyan
Write-Host "   • EventSphere Backend: http://localhost:8000" -ForegroundColor White
Write-Host "   • Prometheus:          http://localhost:9090" -ForegroundColor White
Write-Host "   • Grafana:             http://localhost:3001 (admin/admin123)" -ForegroundColor White
Write-Host "   • AlertManager:        http://localhost:9093" -ForegroundColor White
Write-Host "   • PostgreSQL:          localhost:5432 (postgres/Priapta123)" -ForegroundColor White
Write-Host ""
Write-Host "📈 Sample endpoints to test:" -ForegroundColor Cyan
Write-Host "   • Health check:        curl http://localhost:8000/health" -ForegroundColor White
Write-Host "   • Metrics:             curl http://localhost:8000/metrics" -ForegroundColor White
Write-Host ""
Write-Host "🛑 To stop all services: docker-compose down" -ForegroundColor Red
