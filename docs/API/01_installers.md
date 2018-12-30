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
| `ip` | `string` | 服务器的 IP 地址。 |
| `port` | `int` | 服务器上运行响应 Block Lang 服务的端口。 |
| `platform_name` | `string` | 服务器操作系统名称。 |
| `platform_version` | `string` | 服务器操作系统版本号。 |
| `architecture` | `string` | CPU 架构。 |
| `serverToken` | `string` | **Required**. 用于唯一标识服务器，使用服务器的 MAC 地址。 |

Response

```
Status: 201 CREATED
```

| Name | Type | Description |
|------|------|-------------|
| `token` | `string` | Block Lang 平台为每个项目生成的部署专用 token。 |
| `softwareName` | `string` | 要部署的 Spring Boot jar 的完整名称。 |
| `softwareVersion` | `string` | 要部署的 Spring Boot jar 的版本号。 |
| `softwareFileName` | 要部署的 Spring Boot jar 的完整文件名，与发布中心的名字保持一致。 |
| `softwareRunPort` | Spring Boot jar 在服务器上的运行端口。 |
| `jdkName` | `string` | JDK 在 Block Lang 平台登记的名称。 |
| `jdkVersion` | `string` | JDK 的完整版本号。 |
| `jdkFileName` | `string` | JDK 的完整文件名，在服务器上 JDK 以此命名。 |

config.toml 结构

| Name | Description |
|------|-------------|
| `token` | 部署专用 token。 |
| `server_token` | 用于唯一标识服务器，使用服务器的 MAC 地址。 |
| `software_name` | 要部署的 Spring Boot jar 的完整名称。 |
| `software_version` | 要部署的 Spring Boot jar 的版本号。 |
| `software_file_name` | 要部署的 Spring Boot jar 的完整文件名，与发布中心的名字保持一致。 |
| `software_run_port` | Spring Boot jar 在服务器上的运行端口。 |
| `jdk_name` | JDK 在 Block Lang 平台登记的名称。 |
| `jdk_version` | JDK 的完整版本号。 |
| `jdk_file_name` | JDK 的完整文件名，在服务器上 JDK 以此命名。 |
