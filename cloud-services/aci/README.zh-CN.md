# Azure 容器实例部署 Clipper Server

此目录包含将 Clipper Server 部署到 Azure 容器实例 (ACI) 的 ARM 模板。

[![部署到 Azure](https://aka.ms/deploytoazurebutton)](https://portal.azure.com/#create/Microsoft.Template/uri/https%3A%2F%2Fraw.githubusercontent.com%2Fwindoze%2Fclipper%2Fmain%2Fcloud-services%2Faci%2Fazuredeploy.json)

## 为什么需要构建自己的镜像？

Azure 容器实例使用基于 SMB 协议的 Azure 文件共享作为持久存储。然而，RocksDB（SurrealDB 使用）需要 SMB 不支持的硬链接。这意味着：

- **数据库无法直接存储在 Azure 文件共享上**
- `backup` 镜像变体包含一个包装脚本，用于：
  - 容器停止时将数据库备份到 Azure 文件共享上的 tar.gz 文件
  - 容器启动时从备份恢复数据库

由于官方 `windoze/clipper-server:backup` 镜像可能不可用，您需要构建并发布自己的镜像。

## 构建和发布 Docker 镜像

### 前提条件

- 本地已安装 Docker
- 容器注册表（Docker Hub、Azure 容器注册表、GitHub 容器注册表等）

### 构建备份镜像

```bash
# 克隆仓库
git clone https://github.com/windoze/clipper.git
cd clipper

# 构建支持备份的镜像
docker build -f Dockerfile.backup -t your-registry/clipper-server:backup .

# 多平台构建（推荐用于 ACI）
docker buildx build -f Dockerfile.backup \
  --platform linux/amd64,linux/arm64 \
  -t your-registry/clipper-server:backup \
  --push .
```

### 推送到 Docker Hub

```bash
docker login
docker push your-dockerhub-username/clipper-server:backup
```

### 推送到 Azure 容器注册表

```bash
# 创建 ACR（如不存在）
az acr create --resource-group clipper-rg --name yourregistry --sku Basic

# 登录 ACR
az acr login --name yourregistry

# 标记并推送
docker tag your-registry/clipper-server:backup yourregistry.azurecr.io/clipper-server:backup
docker push yourregistry.azurecr.io/clipper-server:backup
```

### 推送到 GitHub 容器注册表

```bash
# 登录 GHCR
echo $GITHUB_TOKEN | docker login ghcr.io -u USERNAME --password-stdin

# 标记并推送
docker tag your-registry/clipper-server:backup ghcr.io/your-username/clipper-server:backup
docker push ghcr.io/your-username/clipper-server:backup
```

## 创建的资源

- **存储账户**：启用文件共享的标准 LRS 存储
- **文件共享**：用于持久化备份和附件存储（挂载到 `/data`）
- **容器实例**：运行带公网 IP 和 DNS 标签的 clipper-server

## 前提条件

- 已安装并登录 Azure CLI（`az login`）
- Azure 订阅
- 已推送到注册表的 clipper-server:backup 镜像

## 部署

### 使用 Azure CLI

1. 创建资源组（如不存在）：

```bash
az group create --name clipper-rg --location eastus
```

2. 使用您的镜像部署模板：

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

### 使用参数文件

1. 编辑 `azuredeploy.parameters.json` 填写您的值：
   - `containerGroupName`：容器实例名称
   - `imageName`：您的 Docker 镜像名称（例如 `your-dockerhub-username/clipper-server`）
   - `imageTag`：镜像标签（使用 `backup` 以支持数据库持久化）
   - `storageAccountName`：留空自动生成，或指定您自己的名称
   - `bearerToken`：您希望使用的身份验证令牌（必填）
   - `acmeEmail`：Let's Encrypt 证书通知邮箱（必填）
   - `enableBackup`：启用备份/恢复（默认：true）
   - `includeFilesInBackup`：在备份中包含文件附件（默认：false）
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
4. 填写必需的参数（特别是 `imageName` 填写您的注册表）
5. 点击 **查看 + 创建**

### 使用私有注册表 (ACR)

如果使用 Azure 容器注册表，需要启用管理员访问或使用托管标识：

```bash
# 在 ACR 上启用管理员访问
az acr update -n yourregistry --admin-enabled true

# 获取凭据
az acr credential show -n yourregistry
```

然后将注册表凭据添加到部署中（修改 ARM 模板或使用带托管标识的 ACI）。

## 输出

部署后，模板输出：
- `containerFqdn`：完全限定域名（例如 `clipper-server.eastus.azurecontainer.io`）
- `containerUrl`：访问服务器的 HTTPS URL（例如 `https://clipper-server.eastus.azurecontainer.io`）
- `containerIpAddress`：容器的公网 IP 地址
- `storageAccountName`：创建的存储账户名称

## 配置

容器配置如下：

| 环境变量 | 值 | 说明 |
|---------|-----|------|
| PORT | 80 | HTTP 端口 |
| CLIPPER_TLS_PORT | 443 | HTTPS 端口 |
| CLIPPER_BEARER_TOKEN | （来自参数） | 身份验证令牌 |
| CLIPPER_ACME_ENABLED | true | 启用 Let's Encrypt |
| CLIPPER_ACME_DOMAIN | {dnsNameLabel}.{region}.azurecontainer.io | 证书域名 |
| CLIPPER_ACME_EMAIL | （来自参数） | Let's Encrypt 联系邮箱 |
| CLIPPER_SHORT_URL_BASE | https://{domain} | 短链接基础 URL |
| CLIPPER_DB_PATH | /tmp/db | 数据库路径（临时，会备份） |
| CLIPPER_STORAGE_PATH | /data/storage | 文件存储路径（持久化） |
| CLIPPER_BACKUP_ON_EXIT | true/false | 关闭时创建备份 |
| CLIPPER_RESTORE_ON_START | true/false | 启动时从备份恢复 |
| CLIPPER_BACKUP_PATH | /data/backup.tar.gz | 备份文件位置 |
| CLIPPER_INCLUDE_FILES | true/false | 在备份中包含文件 |

## 备份工作原理

`backup` 镜像变体包含一个入口脚本，用于：

1. **启动时**：如果 `CLIPPER_RESTORE_ON_START=true` 且数据库目录为空，则解压 `/data/backup.tar.gz` 恢复数据库
2. **关闭时**：如果 `CLIPPER_BACKUP_ON_EXIT=true`，则创建包含数据库（以及可选的文件附件）的 `/data/backup.tar.gz`

备份文件存储在 Azure 文件共享上，可在容器重启后持久保存。

## 注意事项

- **数据库持久化**：数据库存储在 `/tmp/db`（容器本地），但会自动备份到 Azure 文件共享。数据可在容器重启后保留，但在意外崩溃时可能有少量数据丢失的窗口期。
- **文件附件**：直接存储在 Azure 文件共享（`/data/storage`）上，无需备份即可持久化
- **存储**：存储账户默认使用标准 LRS，配额 5GB
- **计费模式**：按运行秒数付费（按 CPU 和内存计费）

## 与 Azure 容器应用 (ACA) 的比较

| 功能 | 容器实例 (ACI) | 容器应用 (ACA) |
|------|----------------|----------------|
| 内置 HTTPS | 是（使用 ACME） | 是（自动） |
| 数据库持久化 | 通过备份/恢复 | 通过备份/恢复 |
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

## 故障排除

### 容器启动失败

检查容器日志：
```bash
az container logs --resource-group clipper-rg --name clipper-server
```

### 数据库未持久化

1. 确保 `enableBackup` 设置为 `true`
2. 检查是否使用了 `backup` 镜像标签
3. 验证容器是否正常停止（而非被强制终止）

### 证书问题

1. 确保端口 80 可访问（ACME HTTP-01 验证需要）
2. 检查 `acmeEmail` 是否为有效邮箱地址
3. 首次启动时等待几分钟以完成证书配置
