# Shortener

An extremely simple url shortener backend made for fun.

### API Endpoints

#### 1. Create a Shortened URL

Insert a new URL mapping with a custom slug.

```
POST /i/{slug}/{location}
```

**Example:**
```bash
curl -X POST http://localhost/i/mylink/https://example.com/very/long/url
```

**Response:**
- Status: `200 OK` on success
- Status: `400 Bad Request` if slug or location is missing

#### 2. Redirect to Original URL

Access a shortened URL to be redirected to the original location. This also increments the visitor count.

```
GET /{slug}
```

**Example:**
```bash
curl -L http://localhost/mylink
```

**Response:**
- Status: `302 Found` with `Location` header pointing to the original URL
- Status: `404 Not Found` if the slug doesn't exist

#### 3. Get Visitor Count

Retrieve the number of visits for a shortened URL.

```
GET /v/{slug}
```

**Example:**
```bash
curl http://localhost/v/mylink
```

**Response:**
- Status: `200 OK` with visitor count in the body
- Status: `404 Not Found` if the slug doesn't exist

### Usage Examples

```bash
# Create a shortened URL
curl -X POST http://localhost/i/github/https://github.com

# Redirect to the URL (increments visitor count)
curl -L http://localhost/github

# Check visitor count
curl http://localhost/v/github
```

## Benchmarking

The project includes a benchmark script using `oha`, a HTTP load generator.

### Install oha

```bash
# On macOS
brew install oha

# On Linux
cargo install oha

# Or use the pre-built binary from GitHub releases
```

### Run Benchmarks

```bash
# Make the script executable (if needed)
chmod +x benchmark.sh

# Run the benchmark
./benchmark.sh
```

The benchmark script tests the server with:
- **1024 concurrent connections**
- **1,048,576 total requests**
- **Random URL patterns** for both insertion and retrieval operations

This stress tests the server's ability to handle high concurrency and mixed workloads.