# 软件部署服务

将 BlockLang Installer 与 https://blocklang.com 中的一个项目绑定，
即交由 BlockLang Installer 来自动管理项目的部署。

```
POST /installers
```

在注册时，需要登记服务器信息。

Parameters

| Name | Type | Description |
|------|------|-------------|
| `token` | `string` | **Required**. Block Lang 平台为每个项目生成的部署专用 token。 |
| `hwaddr` | `string` | **Required**. 服务器的 MAC 地址。 |
| `ip` | `string` | 服务器的 IP 地址。 |
| `port` | `int` | 服务器上运行响应 Block Lang 服务的端口。 |
| `platform_name` | `string` | 服务器操作系统名称。 |
| `platform_version` | `string` | 服务器操作系统版本号。 |
| `architecture` | `string` | CPU 架构。 |
| `serverToken` | `string` | **Required**. 在服务器上生成的 uuid，用于唯一标识服务器。 |

Response

```
Status: 200 OK
```

| Name | Type | Description |
|------|------|-------------|
| `token` | `string` | Block Lang 平台为每个项目生成的部署专用 token。 |
| `softwareName` | `string` | 要部署的 Spring Boot jar 的完整名称。 |
| `jdkName` | `string` | JDK 在 Block Lang 平台登记的名称。 |
| `jdkVersion` | `string` | JDK 的完整版本号。 |
| `jdkFileName` | `string` | JDK 的完整文件名，在服务器上 JDK 以此命名。 |

config.toml 结构

| Name | Description |
|------|-------------|
| `token` | 部署专用 token |
| `serverToken` | 在服务器上生成的 uuid，用于唯一标识服务器。 |
| `hwaddr` | 服务器的 MAC 地址。 |
| `softwareName` | 要部署的 Spring Boot jar 的完整名称。 |
| `jdkName` | JDK 在 Block Lang 平台登记的名称。 |
| `jdkVersion` | JDK 的完整版本号。 |
| `jdkFileName` | JDK 的完整文件名，在服务器上 JDK 以此命名。 |
