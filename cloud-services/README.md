# Azure Deployment for Clipper Server

This directory contains ARM templates to deploy `windoze/clipper-server:latest` to Azure.

## Deployment Options

- **[aci/](aci/)** - Azure Container Instances (ACI): Simple, serverless container deployment
- **[aca/](aca/)** - Azure Container Apps (ACA): Managed container apps with built-in HTTPS and auto-scaling

See the subdirectories for detailed deployment instructions:

- [Azure Container Instances (ACI)](aci/README.md) - Best for simple, cost-effective deployments
- [Azure Container Apps (ACA)](aca/README.md) - Best for production with built-in HTTPS and monitoring
