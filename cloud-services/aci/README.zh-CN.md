# Azure 容器实例部署 Clipper Server

此目录包含将 `windoze/clipper-server:latest` 部署到 Azure 容器实例 (ACI) 的 ARM 模板。

[![部署到 Azure](https://aka.ms/deploytoazurebutton)](https://portal.azure.com/#create/Microsoft.Template/uri/https%3A%2F%2Fraw.githubusercontent.com%2Fwindoze%2Fclipper%2Fmain%2Fcloud-services%2Faci%2Fazuredeploy.json)

## 创建的资源

- **存储账户**：启用文件共享的标准 LRS 存储
- **文件共享**：用于持久化附件存储（挂载到 `/data`）
- **容器实例**：运行带公网 IP 和 DNS 标签的 clipper-server

## 前提条件

- 已安装并登录 Azure CLI（`az login`）
- Azure 订阅

## 部署

### 使用 Azure CLI

1. 创建资源组（如不存在）：

```bash
az group create --name clipper-rg --location eastus
```

2. 部署模板：

```bash
az deployment group create \
  --resource-group clipper-rg \
  --template-file azuredeploy.json \
  --parameters \
    containerGroupName=clipper-server \
    bearerToken=your-secure-token-here \
    acmeEmail=admin@example.com
```

### 使用参数文件

1. 编辑 `azuredeploy.parameters.json` 填写您的值：
   - `containerGroupName`：容器实例名称
   - `storageAccountName`：留空自动生成，或指定您自己的名称
   - `bearerToken`：您希望使用的身份验证令牌（必填）
   - `acmeEmail`：Let's Encrypt 证书通知邮箱（必填）
   - `location`：Azure 区域（留空使用资源组位置）
   - `dnsNameLabel`：公网 URL 的 DNS 标签（留空则使用容器名称）

2. 部署：

```bash
az deployment group create \
  --resource-group clipper-rg \
  --template-file azuredeploy.json \
  --parameters @azuredeploy.parameters.json
```

### 使用 Azure 门户

1. 在 Azure 门户中进入 **部署自定义模板**
2. 点击 **在编辑器中构建自己的模板**
3. 粘贴 `azuredeploy.json` 的内容
4. 填写必需的参数
5. 点击 **查看 + 创建**

## 输出

部署后，模板输出：
- `containerFqdn`：完全限定域名（例如 `clipper-server.eastus.azurecontainer.io`）
- `containerUrl`：访问服务器的 HTTPS URL（例如 `https://clipper-server.eastus.azurecontainer.io`）
- `containerIpAddress`：容器的公网 IP 地址
- `storageAccountName`：创建的存储账户名称

## 配置

容器配置如下：

| 环境变量 | 值 |
|---------|-----|
| PORT | 80 |
| CLIPPER_TLS_PORT | 443 |
| CLIPPER_BEARER_TOKEN | （来自参数，作为安全值存储） |
| CLIPPER_ACME_ENABLED | true |
| CLIPPER_ACME_DOMAIN | {dnsNameLabel}.{region}.azurecontainer.io |
| CLIPPER_ACME_EMAIL | （来自参数） |
| CLIPPER_SHORT_URL_BASE | https://{dnsNameLabel}.{region}.azurecontainer.io |
| CLIPPER_DB_PATH | /tmp/db（临时存储） |
| CLIPPER_STORAGE_PATH | /data/storage（持久化） |

## 注意事项

- **无内置 HTTPS**：ACI 不提供自动 TLS。如需 HTTPS，请使用反向代理（如 Azure Application Gateway、Cloudflare）或考虑使用 Azure 容器应用
- **带 DNS 的公网 IP**：容器获得带 DNS 标签的公网 IP（`<name>.<region>.azurecontainer.io`）
- **数据库为临时存储**：数据库存储在容器本地存储（`/tmp/db`），容器重启后会丢失。这是因为 RocksDB（SurrealDB 使用）与 Azure 文件共享存在兼容性问题（SMB 不支持 RocksDB 所需的硬链接）。只有文件附件会持久化到 Azure 文件共享。
- **存储**：存储账户默认使用标准 LRS，配额 5GB
- **简单部署**：最适合开发、测试或简单的单实例部署
- **计费模式**：按运行秒数付费（按 CPU 和内存计费）

## HTTPS 选项

由于 ACI 不提供内置 HTTPS，以下是一些选项：

1. **Azure Application Gateway**：在容器前添加 Application Gateway 进行 TLS 终止
2. **Cloudflare 代理**：使用 Cloudflare 作为反向代理，享受免费 SSL
3. **Azure Front Door**：使用 Azure Front Door 进行全球负载均衡和 HTTPS
4. **使用容器应用**：如需内置 HTTPS，请考虑使用 [Azure 容器应用](../aca/README.zh-CN.md)

## 与 Azure 容器应用 (ACA) 的比较

| 功能 | 容器实例 (ACI) | 容器应用 (ACA) |
|------|----------------|----------------|
| 内置 HTTPS | 否（需手动设置） | 是（自动） |
| 缩放 | 不支持自动缩放 | 支持自动缩放 |
| 自定义域名 | 需要 DNS 设置 | 易于配置 |
| 计费模式 | 按运行秒数付费 | 按使用量付费 |
| 复杂度 | 简单 | 功能更多 |
| 适用场景 | 开发/测试、简单部署 | 生产工作负载 |

## 访问服务器

部署后，通过以下地址访问服务器：

```
https://<dns-label>.<region>.azurecontainer.io
```

例如：
```
https://clipper-server.eastus.azurecontainer.io
```

带身份验证：
```bash
curl -H "Authorization: Bearer your-secure-token" \
  https://clipper-server.eastus.azurecontainer.io/clips
```
