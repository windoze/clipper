# Azure Deployment for Clipper Server

This directory contains ARM templates to deploy `windoze/clipper-server:latest` to Azure Container Instances.

## Resources Created

- **Storage Account**: Standard LRS storage with File Share enabled
- **File Share**: For persistent data storage (mounted to `/data`)
- **Container Instance**: Running clipper-server with ACME (Let's Encrypt) enabled

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
    containerGroupName=clipper-server \
    bearerToken=your-secure-token-here
```

### Using Parameters File

1. Edit `azuredeploy.parameters.json` with your values:
   - `containerGroupName`: Name for the container (also used for DNS)
   - `storageAccountName`: Leave empty for auto-generated name, or specify your own
   - `bearerToken`: Your desired authentication token
   - `location`: Azure region (leave empty to use resource group location)

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
- `containerFqdn`: The fully qualified domain name (e.g., `clipper-server.eastus.azurecontainer.io`)
- `containerUrl`: The HTTPS URL to access the server
- `shortUrlBase`: The base URL for short links
- `storageAccountName`: The name of the created storage account

## Configuration

The container is configured with:

| Environment Variable | Value |
|---------------------|-------|
| CLIPPER_ACME_ENABLED | true |
| CLIPPER_BEARER_TOKEN | (from parameter) |
| CLIPPER_ACME_DOMAIN | {name}.{location}.azurecontainer.io |
| CLIPPER_SHORT_URL_BASE | https://{domain}/s/ |
| CLIPPER_DB_PATH | /data/db |
| CLIPPER_STORAGE_PATH | /data/storage |
| CLIPPER_CERTS_DIR | /data/certs |

## Notes

- The container exposes ports 80 and 443 (HTTP and HTTPS)
- ACME will automatically obtain and renew Let's Encrypt certificates
- Data is persisted to Azure File Share and survives container restarts
- The storage account uses Standard LRS with 5GB quota by default
- Initial certificate provisioning may take a few minutes after deployment
