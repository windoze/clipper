# Azure 容器应用部署 Clipper Server

此目录包含将 `windoze/clipper-server:latest` 部署到 Azure 容器应用的 ARM 模板。

[![部署到 Azure](https://aka.ms/deploytoazurebutton)](https://portal.azure.com/#create/Microsoft.Template/uri/https%3A%2F%2Fraw.githubusercontent.com%2Fwindoze%2Fclipper%2Fmain%2Fcloud-services%2Faca%2Fazuredeploy.json)

## 创建的资源

- **存储账户**：启用文件共享的标准 LRS 存储
- **文件共享**：用于持久化数据存储（挂载到 `/data`）
- **Log Analytics 工作区**：用于容器日志和监控
- **容器应用环境**：容器应用的托管环境
- **容器应用**：运行启用 ACME (Let's Encrypt) 的 clipper-server

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
    containerAppName=clipper-server \
    bearerToken=your-secure-token-here
```

### 使用参数文件

1. 编辑 `azuredeploy.parameters.json` 填写您的值：
   - `containerAppName`：容器应用名称（也用于 DNS）
   - `storageAccountName`：留空自动生成，或指定您自己的名称
   - `bearerToken`：您希望使用的身份验证令牌
   - `location`：Azure 区域（留空使用资源组位置）
   - `acmeEmail`：Let's Encrypt 通知邮箱（可选）

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
- `containerAppFqdn`：完全限定域名（例如 `clipper-server.kindpond-abc123.eastus.azurecontainerapps.io`）
- `containerAppUrl`：访问服务器的 HTTPS URL
- `shortUrlBase`：短链接的基础 URL
- `storageAccountName`：创建的存储账户名称

## 配置

容器配置如下：

| 环境变量 | 值 |
|---------|-----|
| PORT | 80 |
| CLIPPER_TLS_PORT | 443 |
| CLIPPER_ACME_ENABLED | true |
| CLIPPER_BEARER_TOKEN | （来自参数，作为密钥存储） |
| CLIPPER_ACME_DOMAIN | {name}.{location}.azurecontainerapps.io |
| CLIPPER_SHORT_URL_BASE | https://{domain} |
| CLIPPER_DB_PATH | /data/db |
| CLIPPER_STORAGE_PATH | /data/storage |
| CLIPPER_CERTS_DIR | /data/certs |
| CLIPPER_ACME_EMAIL | （来自参数） |

## 注意事项

- 容器应用提供内置 HTTPS 和自动 TLS 终止
- 内部 ACME 配置对于 clipper-server 自身的证书管理仍然有用
- 数据持久化到 Azure 文件共享，容器重启后仍然保留
- 存储账户默认使用标准 LRS，配额 5GB
- 容器应用环境包含 Log Analytics 用于日志和监控
- 缩放固定为 1 个副本以确保数据一致性（单实例 SQLite/SurrealDB）

## 与 Azure 容器实例 (ACI) 的区别

| 功能 | 容器应用 (ACA) | 容器实例 (ACI) |
|------|----------------|----------------|
| 内置 HTTPS | 是（自动） | 否（需手动设置） |
| 缩放 | 支持自动缩放 | 不支持自动缩放 |
| 日志 | 包含 Log Analytics | 需单独设置 |
| 自定义域名 | 易于配置 | 需要 DNS 设置 |
| 计费模式 | 按使用量付费 | 按运行秒数付费 |
