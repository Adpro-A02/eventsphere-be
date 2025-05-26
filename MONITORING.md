# EventSphere Monitoring

A minimal guide to using the monitoring setup for EventSphere backend.

## Quick Start

1. Start everything:
   ```
   # Windows
   .\start-dev.ps1
   
   # Linux/Mac
   ./start-dev.sh
   ```

2. Access your services:
   - Backend: http://localhost:8000
   - Prometheus: http://localhost:9090
   - Grafana: http://localhost:3001 (login: admin/admin123)
   - AlertManager: http://localhost:9093

## What Are The Scripts For?

- **start-dev.ps1/sh**: Starts all Docker containers and sets up environment
- **test-monitoring.ps1**: Checks if monitoring endpoints are working properly

## Key Components

### Prometheus (Port 9090)
Collects and stores metrics from the backend.

### Grafana (Port 3001)
Shows pretty dashboards of your metrics data.

### AlertManager (Port 9093)
Sends notifications when things go wrong.

### PostgreSQL (Port 5432)
Database for EventSphere (credentials: postgres/Priapta123)

## Pre-configured Dashboards

- **EventSphere Overview**: All important metrics in one place
- **Backend Metrics**: HTTP requests, response times
- **Database Metrics**: Query performance, errors
- **Transactions**: Business metrics 
- **Alerts**: Overview of active and historical alerts
- **System Overview**: Server health metrics

## Available Metrics

Our backend exposes these at `/metrics`:
- `http_requests_total`: Count of HTTP requests
- `http_request_duration`: How long requests take
- Standard Prometheus metrics

## Useful Queries

### Request Rate
```promql
rate(http_requests_total[5m])
```

### Error Rate
```promql
rate(http_requests_total{code=~"5.."}[5m]) / rate(http_requests_total[5m])
```

### 95th Percentile Response Time
```promql
histogram_quantile(0.95, rate(http_request_duration_bucket[5m]))
```

### Service Uptime
```promql
up{job="eventsphere-be"}
```

## Alerts

We have alerts set up for common issues:

### Critical Alerts
- **HighErrorRate**: Error rate > 5% for 5min
- **ServiceDown**: Backend unavailable for 1min

### Warning Alerts
- **SlowResponseTime**: Response time > 1sec (95th percentile) for 5min

## Common Tasks

### Add Custom Dashboard
1. Go to Grafana → Dashboards → New → Import
2. Use dashboard ID from Grafana.com or upload JSON
3. Select Prometheus as the data source

### Check Alerts
1. View current alerts in Grafana or AlertManager
2. Configure notifications in `alerting/alertmanager.yml`

### Troubleshooting
- Services not starting? Check Docker logs: `docker-compose logs [service-name]`
- No metrics in Grafana? Check Prometheus targets at http://localhost:9090/targets
- Dashboard not loading? Check Grafana logs for errors

### Development Commands
```
# Stop everything
docker-compose down

# Rebuild and restart just the backend
docker-compose build eventsphere-be
docker-compose up -d eventsphere-be

# Check logs
docker-compose logs -f eventsphere-be
```
