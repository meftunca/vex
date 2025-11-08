# Vex Net Linux Test Environment
# Tests epoll and io_uring backends

FROM ubuntu:24.04

# Install build tools and libraries
RUN apt-get update && apt-get install -y \
    build-essential \
    gcc \
    make \
    netcat-openbsd \
    liburing-dev \
    linux-headers-generic \
    && rm -rf /var/lib/apt/lists/*

# Create working directory
WORKDIR /vex_net

# Copy source files
COPY . /vex_net/

# Build
RUN make clean && make

# Default command runs tests
CMD ["./test_linux.sh"]

