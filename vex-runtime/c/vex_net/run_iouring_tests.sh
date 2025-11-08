#!/bin/bash
# Run io_uring tests in Docker

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${YELLOW}=======================================================${NC}"
echo -e "${YELLOW}  Vex Net Docker io_uring Test Runner${NC}"
echo -e "${YELLOW}  Testing Linux io_uring backend${NC}"
echo -e "${YELLOW}=======================================================${NC}"
echo ""

# Build Docker image
echo -e "${BLUE}▶ Building Docker image...${NC}"
docker build -t vex_net_test -f Dockerfile . 2>&1 | grep -E "(#|ERROR|=>)" || echo "Build in progress..."

if [ $? -ne 0 ]; then
    echo -e "${RED}✗ Docker build failed${NC}"
    exit 1
fi

echo -e "${GREEN}✓ Docker image built${NC}"
echo ""

# Run io_uring tests
echo -e "${BLUE}▶ Running io_uring tests in Docker...${NC}"
docker run --rm vex_net_test bash -c "chmod +x test_iouring.sh && ./test_iouring.sh"

EXIT_CODE=$?

echo ""
if [ $EXIT_CODE -eq 0 ]; then
    echo -e "${GREEN}=======================================================${NC}"
    echo -e "${GREEN}  ✅ ALL io_uring TESTS PASSED${NC}"
    echo -e "${GREEN}=======================================================${NC}"
else
    echo -e "${RED}=======================================================${NC}"
    echo -e "${RED}  ✗ io_uring TESTS FAILED (exit code: $EXIT_CODE)${NC}"
    echo -e "${RED}=======================================================${NC}"
fi

exit $EXIT_CODE

