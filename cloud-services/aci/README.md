# Azure Container Instances Deployment for Clipper Server

This directory contains ARM templates to deploy `windoze/clipper-server:latest` to Azure Container Instances (ACI).

[![Deploy to Azure](https://aka.ms/deploytoazurebutton)](https://portal.azure.com/#create/Microsoft.Template/uri/https%3A%2F%2Fraw.githubusercontent.com%2Fwindoze%2Fclipper%2Fmain%2Fcloud-services%2Faci%2Fazuredeploy.json)

## Resources Created

- **Storage Account**: Standard LRS storage with File Share enabled
- **File Share**: For persistent data storage (mounted to `/data`)
- **Container Instance**: Running clipper-server with public IP and DNS label

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
    bearerToken=your-secure-token-here \
    acmeEmail=admin@example.com
```

### Using Parameters File

1. Edit `azuredeploy.parameters.json` with your values:
   - `containerGroupName`: Name for the Container Instance
   - `storageAccountName`: Leave empty for auto-generated name, or specify your own
   - `bearerToken`: Your desired authentication token (required)
   - `acmeEmail`: Email for Let's Encrypt certificate notifications (required)
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
4. Fill in the required parameters
5. Click **Review + create**

## Outputs

After deployment, the template outputs:
- `containerFqdn`: The fully qualified domain name (e.g., `clipper-server.eastus.azurecontainer.io`)
- `containerUrl`: The HTTPS URL to access the server (e.g., `https://clipper-server.eastus.azurecontainer.io`)
- `containerIpAddress`: The public IP address of the container
- `storageAccountName`: The name of the created storage account

## Configuration

The container is configured with:

| Environment Variable | Value |
|---------------------|-------|
| PORT | 80 |
| CLIPPER_TLS_PORT | 443 |
| CLIPPER_BEARER_TOKEN | (from parameter, stored as secure value) |
| CLIPPER_ACME_ENABLED | true |
| CLIPPER_ACME_DOMAIN | {dnsNameLabel}.{region}.azurecontainer.io |
| CLIPPER_ACME_EMAIL | (from parameter) |
| CLIPPER_SHORT_URL_BASE | https://{dnsNameLabel}.{region}.azurecontainer.io |
| CLIPPER_DB_PATH | /data/db |
| CLIPPER_STORAGE_PATH | /data/storage |

## Notes

- **No built-in HTTPS**: ACI does not provide automatic TLS. For HTTPS, use a reverse proxy (e.g., Azure Application Gateway, Cloudflare) or consider Azure Container Apps instead
- **Public IP with DNS**: The container gets a public IP with a DNS label (`<name>.<region>.azurecontainer.io`)
- **Data persistence**: Data is persisted to Azure File Share and survives container restarts
- **Storage**: The storage account uses Standard LRS with 5GB quota by default
- **Simple deployment**: Best for development, testing, or simple single-instance deployments
- **Cost model**: Pay per second of running time (billed by CPU and memory)

## HTTPS Options

Since ACI doesn't provide built-in HTTPS, here are some options:

1. **Azure Application Gateway**: Add an Application Gateway in front of the container for TLS termination
2. **Cloudflare Proxy**: Use Cloudflare as a reverse proxy with their free SSL
3. **Azure Front Door**: Use Azure Front Door for global load balancing and HTTPS
4. **Use Container Apps**: For built-in HTTPS, consider [Azure Container Apps](../aca/README.md) instead

## Comparison with Azure Container Apps (ACA)

| Feature | Container Instances (ACI) | Container Apps (ACA) |
|---------|--------------------------|---------------------|
| Built-in HTTPS | No (manual setup) | Yes (automatic) |
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
