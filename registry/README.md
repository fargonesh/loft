# loft Package Registry

A simple, public package registry server for the loft programming language.

## Features

- **Public Access**: No authentication required
- **Package Publishing**: Upload packages to the registry
- **Package Discovery**: List and search for packages
- **Package Download**: Download packages by name and version
- **Persistent Storage**: Packages are stored on disk
- **Production Ready**: Configurable binding address and storage location

## Running the Server

### Development (Local)
```bash
cd registry
cargo run
```

The server will start on `http://0.0.0.0:3030` by default (accessible from any network interface).

### Production Deployment

Configure the server using environment variables:

```bash
# Set custom bind address (default: 0.0.0.0:3030)
export BIND_ADDR="0.0.0.0:8080"

# Set custom storage directory (default: ./registry-storage)
export STORAGE_DIR="/var/lib/loft-registry"

# Run the server
cargo run --release
```

For production, consider:
- Running behind a reverse proxy (nginx, caddy) for HTTPS
- Setting up proper logging and monitoring
- Configuring firewall rules
- Using a process manager (systemd, docker, etc.)

## API Endpoints

### GET /
Get registry information

**Response:**
```json
{
  "name": "loft Package Registry",
  "version": "0.1.0",
  "packages_count": 5
}
```

### GET /packages
List all available packages

**Response:**
```json
[
  {
    "name": "http-client",
    "version": "1.0.0",
    "description": "HTTP client library"
  }
]
```

### GET /packages/:name
Get all versions of a specific package

**Response:**
```json
[
  {
    "name": "http-client",
    "version": "1.0.0",
    "description": "HTTP client library"
  },
  {
    "name": "http-client",
    "version": "1.1.0",
    "description": "HTTP client library"
  }
]
```

### GET /packages/:name/:version/download
Download a specific package version as a tarball

**Response:** Binary tarball data

### POST /packages/publish
Publish a new package version

**Request Body:**
```json
{
  "name": "my-package",
  "version": "1.0.0",
  "description": "My awesome package",
  "manifest": {
    "name": "my-package",
    "version": "1.0.0",
    "entrypoint": "src/main.lf",
    "dependencies": {}
  },
  "tarball": "base64-encoded-tarball-data"
}
```

**Response:**
```json
{
  "name": "my-package",
  "version": "1.0.0",
  "description": "My awesome package"
}
```

## Storage

Packages are stored in the `registry-storage/` directory (configurable via `STORAGE_DIR` environment variable).

Directory structure:
```
registry-storage/
├── package-name/
│   ├── 1.0.0.tar.gz
│   ├── 1.0.0.json
│   ├── 1.1.0.tar.gz
│   └── 1.1.0.json
```

## Integration with loft CLI

The loft CLI (`loft` command) is configured to communicate with this registry server for package installation and management.

Use `loft add <package-name>` to install packages from the registry.
