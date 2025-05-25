#!/bin/bash

echo "🚀 Starting EventSphere Development Environment..."

if [ ! -f .env ]; then
    echo "📄 Creating .env file from .env.example..."
    cp .env.example .env
    echo "✅ Please review and update .env file with your configurations"
fi

echo "🐳 Starting Docker containers..."
docker-compose up -d

echo "⏳ Waiting for services to start..."
sleep 10

echo "🔍 Checking service status..."
docker-compose ps

echo ""
echo "🎉 EventSphere Development Environment is ready!"
echo ""
echo "📊 Access your services:"
echo "   • EventSphere Backend: http://localhost:8000"
echo "   • Prometheus:          http://localhost:9090"
echo "   • Grafana:             http://localhost:3001 (admin/admin123)"
echo "   • PostgreSQL:          localhost:5432 (postgres/Priapta123)"
echo ""
echo "📈 Sample endpoints to test:"
echo "   • Health check:        curl http://localhost:8000/health"
echo "   • Metrics:             curl http://localhost:8000/metrics"
echo ""
echo "🛑 To stop all services: docker-compose down"
