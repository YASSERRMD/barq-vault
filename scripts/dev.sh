#!/bin/bash
set -e

echo "Barq-Vault Dev Starter"
echo "----------------------"

# Build and start the containerized version of the application
docker-compose --file docker-compose.yml up --build -d

echo ""
echo "Server is starting up!"
echo "gRPC API: localhost:50051"
echo "REST API: localhost:8080"
echo "Data volume: barq-data"
echo ""
echo "To view logs, run: docker-compose logs -f"
echo "To stop, run: docker-compose down"
