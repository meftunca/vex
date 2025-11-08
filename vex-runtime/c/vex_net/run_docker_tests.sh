#!/bin/bash

# Vex Net Docker Test Runner

set -e

echo "═══════════════════════════════════════════════════════"
echo "  Vex Net Docker Test Runner"
echo "  Testing Linux (epoll/io_uring) backends"
echo "═══════════════════════════════════════════════════════"

# Check if Docker is running
if ! docker info > /dev/null 2>&1; then
    echo "❌ Docker is not running. Please start Docker and try again."
    exit 1
fi

echo ""
echo "▶ Building Docker image..."
docker build -t vex_net_test .

echo ""
echo "▶ Running Linux tests in Docker..."
docker run --rm vex_net_test

echo ""
echo "═══════════════════════════════════════════════════════"
echo "  Docker tests complete!"
echo "═══════════════════════════════════════════════════════"

