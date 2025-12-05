# Clipper Server 云部署

此目录包含将 `windoze/clipper-server:latest` 部署到不同云服务商的方法。

## 部署选项

- **[aca/](aca/)** - Azure 容器应用 (ACA)：托管容器应用，内置 HTTPS 和自动缩放
- **[aci/](aci/)** - Azure 容器实例 (ACI)：简单的无服务器容器部署

- TODO

详细部署说明请参阅子目录：

- [Azure 容器应用 (ACA)](aca/README.zh-CN.md) - 推荐用于生产环境，内置 HTTPS 和监控
- [Azure 容器实例 (ACI)](aci/README.zh-CN.md) - 适合简单、经济的部署
