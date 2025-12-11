# Azure Container Instances Deployment for Clipper Server

This directory contains ARM templates to deploy Clipper Server to Azure Container Instances (ACI).

[![Deploy to Azure](https://aka.ms/deploytoazurebutton)](https://portal.azure.com/#create/Microsoft.Template/uri/https%3A%2F%2Fraw.githubusercontent.com%2Fwindoze%2Fclipper%2Fmain%2Fcloud-services%2Faci%2Fazuredeploy.json)

## Why Build Your Own Image?

Azure Container Instances uses Azure File Share for persistent storage, which is based on SMB protocol. However, RocksDB (used by SurrealDB) requires hard links that SMB doesn't support. This means:

- **Database cannot be stored on Azure File Share directly**
- The `backup` image variant includes a wrapper script that:
  - Backs up the database to a tar.gz file on Azure File Share when the container stops
  - Restores the database from the backup when the container starts

Since the official `windoze/clipper-server:backup` image may not be available, you need to build and publish your own image.

## Building and Publishing the Docker Image

### Prerequisites

- Docker installed locally
- A container registry (Docker Hub, Azure Container Registry, GitHub Container Registry, etc.)

### Build the Backup Image

```bash
# Clone the repository
git clone https://github.com/windoze/clipper.git
cd clipper

# Build the backup-enabled image
docker build -f Dockerfile.backup -t your-registry/clipper-server:backup .

# For multi-platform build (recommended for ACI)
docker buildx build -f Dockerfile.backup \
  --platform linux/amd64,linux/arm64 \
  -t your-registry/clipper-server:backup \
  --push .
```

### Push to Docker Hub

```bash
docker login
docker push your-dockerhub-username/clipper-server:backup
```

### Push to Azure Container Registry

```bash
# Create ACR (if not exists)
az acr create --resource-group clipper-rg --name yourregistry --sku Basic

# Login to ACR
az acr login --name yourregistry

# Tag and push
docker tag your-registry/clipper-server:backup yourregistry.azurecr.io/clipper-server:backup
docker push yourregistry.azurecr.io/clipper-server:backup
```

### Push to GitHub Container Registry

```bash
# Login to GHCR
echo $GITHUB_TOKEN | docker login ghcr.io -u USERNAME --password-stdin

# Tag and push
docker tag your-registry/clipper-server:backup ghcr.io/your-username/clipper-server:backup
docker push ghcr.io/your-username/clipper-server:backup
```

## Resources Created

- **Storage Account**: Standard LRS storage with File Share enabled
- **File Share**: For persistent backup and attachment storage (mounted to `/data`)
- **Container Instance**: Running clipper-server with public IP and DNS label

## Prerequisites

- Azure CLI installed and logged in (`az login`)
- An Azure subscription
- Your own clipper-server:backup image pushed to a registry

## Deployment

### Using Azure CLI

1. Create a resource group (if not exists):

```bash
az group create --name clipper-rg --location eastus
```

2. Deploy the template with your image:

```bash
az deployment group create \
  --resource-group clipper-rg \
  --template-file azuredeploy.json \
  --parameters \
    containerGroupName=clipper-server \
    imageName=your-registry/clipper-server \
    imageTag=backup \
    bearerToken=your-secure-token-here \
    acmeEmail=admin@example.com
```

### Using Parameters File

1. Edit `azuredeploy.parameters.json` with your values:
   - `containerGroupName`: Name for the Container Instance
   - `imageName`: Your Docker image name (e.g., `your-dockerhub-username/clipper-server`)
   - `imageTag`: Image tag (use `backup` for database persistence)
   - `storageAccountName`: Leave empty for auto-generated name, or specify your own
   - `bearerToken`: Your desired authentication token (required)
   - `acmeEmail`: Email for Let's Encrypt certificate notifications (required)
   - `enableBackup`: Enable backup/restore (default: true)
   - `includeFilesInBackup`: Include file attachments in backup (default: false)
   - `location`: Azure region (leave empty to use resource group location)
   - `dnsNameLabel`: DNS label for the public URL (leave empty to use container name)

2. Deploy:

```bash
az deployment group create \
  --resource-group clipper-rg \
  --template-file azuredeploy.json \
  --parameters @azuredeploy.parameters.json
```

### Using Azure Portal

1. Go to **Deploy a custom template** in Azure Portal
2. Click **Build your own template in the editor**
3. Paste the contents of `azuredeploy.json`
4. Fill in the required parameters (especially `imageName` with your registry)
5. Click **Review + create**

### Using Private Registry (ACR)

If using Azure Container Registry, you need to enable admin access or use managed identity:

```bash
# Enable admin access on ACR
az acr update -n yourregistry --admin-enabled true

# Get credentials
az acr credential show -n yourregistry
```

Then add registry credentials to the deployment (modify the ARM template or use ACI with managed identity).

## Outputs

After deployment, the template outputs:
- `containerFqdn`: The fully qualified domain name (e.g., `clipper-server.eastus.azurecontainer.io`)
- `containerUrl`: The HTTPS URL to access the server (e.g., `https://clipper-server.eastus.azurecontainer.io`)
- `containerIpAddress`: The public IP address of the container
- `storageAccountName`: The name of the created storage account

## Configuration

The container is configured with:

| Environment Variable | Value | Description |
|---------------------|-------|-------------|
| PORT | 80 | HTTP port |
| CLIPPER_TLS_PORT | 443 | HTTPS port |
| CLIPPER_BEARER_TOKEN | (from parameter) | Authentication token |
| CLIPPER_ACME_ENABLED | true | Enable Let's Encrypt |
| CLIPPER_ACME_DOMAIN | {dnsNameLabel}.{region}.azurecontainer.io | Domain for certificate |
| CLIPPER_ACME_EMAIL | (from parameter) | Let's Encrypt contact email |
| CLIPPER_SHORT_URL_BASE | https://{domain} | Base URL for short links |
| CLIPPER_DB_PATH | /tmp/db | Database path (ephemeral, backed up) |
| CLIPPER_STORAGE_PATH | /data/storage | File storage path (persistent) |
| CLIPPER_BACKUP_ON_EXIT | true/false | Create backup on shutdown |
| CLIPPER_RESTORE_ON_START | true/false | Restore from backup on startup |
| CLIPPER_BACKUP_PATH | /data/backup.tar.gz | Backup file location |
| CLIPPER_INCLUDE_FILES | true/false | Include files in backup |

## How Backup Works

The `backup` image variant includes an entrypoint script that:

1. **On startup**: If `CLIPPER_RESTORE_ON_START=true` and the database directory is empty, extracts `/data/backup.tar.gz` to restore the database
2. **On shutdown**: If `CLIPPER_BACKUP_ON_EXIT=true`, creates `/data/backup.tar.gz` containing the database (and optionally file attachments)

The backup file is stored on Azure File Share, which persists across container restarts.

## Notes

- **Database persistence**: The database is stored in `/tmp/db` (container-local) but automatically backed up to Azure File Share. Data survives container restarts but may have a small window of potential data loss during unexpected crashes.
- **File attachments**: Stored directly on Azure File Share (`/data/storage`) and persist without backup
- **Storage**: The storage account uses Standard LRS with 5GB quota by default
- **Cost model**: Pay per second of running time (billed by CPU and memory)

## Comparison with Azure Container Apps (ACA)

| Feature | Container Instances (ACI) | Container Apps (ACA) |
|---------|--------------------------|---------------------|
| Built-in HTTPS | Yes (with ACME) | Yes (automatic) |
| Database persistence | Via backup/restore | Via backup/restore |
| Scaling | No auto-scale | Auto-scale supported |
| Custom domains | DNS setup required | Easy configuration |
| Cost model | Pay per second running | Pay per usage |
| Complexity | Simple | More features |
| Best for | Dev/test, simple deployments | Production workloads |

## Accessing the Server

After deployment, access the server at:

```
https://<dns-label>.<region>.azurecontainer.io
```

For example:
```
https://clipper-server.eastus.azurecontainer.io
```

With authentication:
```bash
curl -H "Authorization: Bearer your-secure-token" \
  https://clipper-server.eastus.azurecontainer.io/clips
```

## Troubleshooting

### Container fails to start

Check container logs:
```bash
az container logs --resource-group clipper-rg --name clipper-server
```

### Database not persisting

1. Ensure `enableBackup` is set to `true`
2. Check that you're using the `backup` image tag
3. Verify the container is stopping gracefully (not being killed)

### Certificate issues

1. Ensure port 80 is accessible (needed for ACME HTTP-01 challenge)
2. Check that `acmeEmail` is a valid email address
3. Wait a few minutes for certificate provisioning on first start
