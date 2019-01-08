# 软件部署服务

## 向 Block Lang 平台注册 Installer 信息

将 BlockLang Installer 与 <https://blocklang.com> 中的一个项目绑定，
即交由 BlockLang Installer 来自动管理项目的部署。

```text
POST /installers
```

在注册时，需要登记服务器信息。

Parameters

| Name | Type | Description |
|------|------|-------------|
| `registrationToken` | `string` | **Required**. Block Lang 平台为每个项目生成唯一的注册 token。 |
| `ip` | `string` | 服务器的 IP 地址。 |
| `port` | `int` | 服务器上运行响应 Block Lang 服务的端口（**未实现**）。 |
| `softwareRunPort` | `int` | Spring Boot jar 在服务器上的运行端口。 |
| `osType` | `string` | 服务器操作系统类型，如 `Windows`、`Ubuntu`。 |
| `osVersion` | `string` | 服务器操作系统版本号。 |
| `arch` | `string` | CPU 架构。 |
| `serverToken` | `string` | **Required**. 用于唯一标识服务器，使用服务器的 MAC 地址。 |

Response

```text
Status: 201 CREATED
```

| Name | Type | Description |
|------|------|-------------|
| `installerToken` | `string` | Block Lang 平台为每个 installer 生成的 token。 |
| `softwareName` | `string` | 要部署的 Spring Boot jar 的完整名称，由 Block Lang 平台中的用户名和项目名组成，格式为 `@userName/projectName`。 |
| `softwareVersion` | `string` | 要部署的 Spring Boot jar 的版本号。 |
| `softwareFileName` | `string` | 要部署的 Spring Boot jar 的完整文件名，与发布中心的名字保持一致。 |
| `softwareRunPort` | `int` | Spring Boot jar 在服务器上的运行端口。 |
| `jdkName` | `string` | JDK 在 Block Lang 平台登记的名称。 |
| `jdkVersion` | `string` | JDK 的完整版本号。 |
| `jdkFileName` | `string` | JDK 的完整文件名，在服务器上 JDK 以此命名。 |

## 向 Block Lang 平台获取软件最新信息

更新服务器信息并获取最新版本软件信息。

```text
PUT /installers
```

Parameters

| Name | Type | Description |
|------|------|-------------|
| `registrationToken` | `string` | **Required**. Block Lang 平台为每个项目生成唯一的注册 token。 |
| `ip` | `string` | 服务器的 IP 地址。 |
| `port` | `int` | 服务器上运行响应 Block Lang 服务的端口。 |
| `osType` | `string` | 服务器操作系统类型，如 `Windows`、`Ubuntu`。 |
| `osVersion` | `string` | 服务器操作系统版本号。 |
| `arch` | `string` | CPU 架构。 |
| `serverToken` | `string` | **Required**. 用于唯一标识服务器，使用服务器的 MAC 地址。 |

Response

```text
Status: 200 OK
```

| Name | Type | Description |
|------|------|-------------|
| `installerToken` | `string` | Block Lang 平台为每个 installer 生成的 token。 |
| `softwareName` | `string` | 要部署的 Spring Boot jar 的完整名称，由 Block Lang 平台中的用户名和项目名组成，格式为 `@userName/projectName`。 |
| `softwareVersion` | `string` | 要部署的 Spring Boot jar 的版本号。 |
| `softwareFileName` | `string` | 要部署的 Spring Boot jar 的完整文件名，与发布中心的名字保持一致。 |
| `softwareRunPort` | `int` | Spring Boot jar 在服务器上的运行端口。 |
| `jdkName` | `string` | JDK 在 Block Lang 平台登记的名称。 |
| `jdkVersion` | `string` | JDK 的完整版本号。 |
| `jdkFileName` | `string` | JDK 的完整文件名，在服务器上 JDK 以此命名。 |

## 向 Block Lang 平台注销 Installer 信息

```text
DELETE /installers/{installerToken}
```

Parameters

|       Name       |   Type   |                         Description                        |
|------------------|----------|------------------------------------------------------------|
| `installerToken` | `string` | **Required**. Block Lang 平台为每个 installer 生成的 token。 |

Response

```text
Status: 204 NO CONTENT
```

## `config.toml` 结构

| Name | Description |
|------|-------------|
| `url` | Block Lang 软件发布中心的 URL，如 `https://blocklang.store`。 |
| `installer_token` | 为每一个 installer 生成的唯一 token。 |
| `server_token` | 用于唯一标识服务器，使用服务器的 MAC 地址。 |
| `software_name` | 要部署的 Spring Boot jar 的完整名称，由 Block Lang 平台中的用户名和项目名组成，格式为 `@userName/projectName`。 |
| `software_version` | 要部署的 Spring Boot jar 的版本号。 |
| `software_file_name` | 要部署的 Spring Boot jar 的完整文件名，与发布中心的名字保持一致。 |
| `software_run_port` | Spring Boot jar 在服务器上的运行端口。 |
| `jdk_name` | JDK 在 Block Lang 平台登记的名称。 |
| `jdk_version` | JDK 的完整版本号。 |
| `jdk_file_name` | JDK 的完整文件名，在服务器上 JDK 以此命名。 |
