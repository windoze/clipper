# Azure Container Apps Deployment for Clipper Server

This directory contains ARM templates to deploy `windoze/clipper-server:latest` to Azure Container Apps.

[![Deploy to Azure](https://aka.ms/deploytoazurebutton)](https://portal.azure.com/#create/Microsoft.Template/uri/https%3A%2F%2Fraw.githubusercontent.com%2Fwindoze%2Fclipper%2Fmain%2Fcloud-services%2Faca%2Fazuredeploy.json)

## Resources Created

- **Storage Account**: Standard LRS storage with File Share enabled
- **File Share**: For persistent data storage (mounted to `/data`)
- **Log Analytics Workspace**: For container logs and monitoring
- **Container Apps Environment**: Managed environment for the Container App
- **Container App**: Running clipper-server with ACME (Let's Encrypt) enabled

## Prerequisites

- Azure CLI installed and logged in (`az login`)
- An Azure subscription

## Deployment

### Using Azure CLI

1. Create a resource group (if not exists):

```bash
az group create --name clipper-rg --location eastus
```

2. Deploy the template:

```bash
az deployment group create \
  --resource-group clipper-rg \
  --template-file azuredeploy.json \
  --parameters \
    containerAppName=clipper-server \
    bearerToken=your-secure-token-here
```

### Using Parameters File

1. Edit `azuredeploy.parameters.json` with your values:
   - `containerAppName`: Name for the Container App (also used for DNS)
   - `storageAccountName`: Leave empty for auto-generated name, or specify your own
   - `bearerToken`: Your desired authentication token
   - `location`: Azure region (leave empty to use resource group location)
   - `acmeEmail`: Email for Let's Encrypt notifications (optional)

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
4. Fill in the required parameters
5. Click **Review + create**

## Outputs

After deployment, the template outputs:
- `containerAppFqdn`: The fully qualified domain name (e.g., `clipper-server.kindpond-abc123.eastus.azurecontainerapps.io`)
- `containerAppUrl`: The HTTPS URL to access the server
- `shortUrlBase`: The base URL for short links
- `storageAccountName`: The name of the created storage account

## Configuration

The container is configured with:

| Environment Variable | Value |
|---------------------|-------|
| PORT | 80 |
| CLIPPER_TLS_PORT | 443 |
| CLIPPER_ACME_ENABLED | true |
| CLIPPER_BEARER_TOKEN | (from parameter, stored as secret) |
| CLIPPER_ACME_DOMAIN | {name}.{location}.azurecontainerapps.io |
| CLIPPER_SHORT_URL_BASE | https://{domain} |
| CLIPPER_DB_PATH | /data/db |
| CLIPPER_STORAGE_PATH | /data/storage |
| CLIPPER_CERTS_DIR | /data/certs |
| CLIPPER_ACME_EMAIL | (from parameter) |

## Notes

- Container Apps provides built-in HTTPS with automatic TLS termination
- The internal ACME configuration is still useful for the clipper-server's own certificate management
- Data is persisted to Azure File Share and survives container restarts
- The storage account uses Standard LRS with 5GB quota by default
- Container Apps Environment includes Log Analytics for logging and monitoring
- Scale is fixed at 1 replica to ensure data consistency (single-instance SQLite/SurrealDB)

## Differences from Azure Container Instance (ACI)

| Feature | Container Apps (ACA) | Container Instance (ACI) |
|---------|---------------------|--------------------------|
| Built-in HTTPS | Yes (automatic) | No (manual setup) |
| Scaling | Auto-scale supported | No auto-scale |
| Logging | Log Analytics included | Separate setup needed |
| Custom domains | Easy configuration | DNS setup required |
| Cost model | Pay per usage | Pay per second running |
