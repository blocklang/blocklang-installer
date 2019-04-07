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
| `appRunPort` | `int` | Spring Boot jar 在服务器上的运行端口。 |
| `osType` | `string` | 服务器操作系统的具体类型，如 `Windows`、`Ubuntu`。 |
| `osVersion` | `string` | 服务器操作系统版本号。 |
| `targetOs` | `string` | 服务器操作系统类型，如 `Windows`、`Linux`、`Macos`、`Solaris`。 |
| `arch` | `string` | CPU 架构。 |
| `serverToken` | `string` | **Required**. 用于唯一标识服务器，使用服务器的 MAC 地址。 |

说明：有了更具体的 `osType`，为什么还要添加 `target_os`？这样做是为了减少转换，直接根据 `target_os` 获取 APP 发行版文件。

Response

输入参数校验

```text
Status: 422 Unprocessable Entity
```

只有以下校验规则通过后，才能开始注册：

1. 所有输入参数的值不能为空；
2. 根据注册 token 没有获取到 APP 基本信息
3. APP 的发行版基本信息不存在
4. APP 发行版文件信息不存在
5. JDK 发行版信息不存在
6. JDK 的基本信息不存在
7. JDK 的发行版文件信息不存在

注册成功

```text
Status: 201 CREATED
```

| Name | Type | Description |
|------|------|-------------|
| `installerToken` | `string` | Block Lang 平台为每个 installer 生成的 token。 |
| `appName` | `string` | 要部署的 Spring Boot jar 的完整名称，由 Block Lang 平台中的用户名和项目名组成，格式为 `@userName/projectName`。 |
| `appVersion` | `string` | 要部署的 Spring Boot jar 的版本号。 |
| `appFileName` | `string` | 要部署的 Spring Boot jar 的完整文件名，与发布中心的名字保持一致。 |
| `appRunPort` | `int` | Spring Boot jar 在服务器上的运行端口。 |
| `jdkName` | `string` | JDK 在 Block Lang 平台登记的名称。 |
| `jdkVersion` | `string` | JDK 的完整版本号。 |
| `jdkFileName` | `string` | JDK 的完整文件名，在服务器上 JDK 以此命名。 |

## 向 Block Lang 平台获取软件最新信息

更新服务器信息并获取最新版本软件信息。

注意，因为在 `installer_config.toml` 中只存储了 `installer_token`，并没有存储 `registration_token`，所以获取更新信息时使用的是 `installer_token`。

```text
PUT /installers
```

Parameters

| Name | Type | Description |
|------|------|-------------|
| `installerToken` | `string` | **Required**. Block Lang 平台为每个 installer 生成的 token。 |
| `ip` | `string` | 服务器的 IP 地址。 |
| `port` | `int` | 服务器上运行响应 Block Lang 服务的端口。 |
| `osType` | `string` | 服务器操作系统类型，如 `Windows`、`Ubuntu`。 |
| `osVersion` | `string` | 服务器操作系统版本号。 |
| `targetOs` | `string` | 服务器操作系统类型，如 `Windows`、`Linux`、`Macos`、`Solaris`。 |
| `arch` | `string` | CPU 架构。 |
| `serverToken` | `string` | **Required**. 用于唯一标识服务器，使用服务器的 MAC 地址。 |

Response

输入参数校验

```text
Status: 422 Unprocessable Entity
```

只有以下校验规则通过后，才能开始注册：

1. 所有输入参数的值不能为空；
2. 根据安装器 token 没有获取到安装器基本信息
3. 确认 Block Lang 平台的 Server Token 与参数中的 Server Token 一致
4. APP 的发行版基本信息不存在
5. APP 基本信息不存在
6. APP 最新发行版信息不存在
7. APP 最新发行版文件信息不存在
8. JDK 发行版信息不存在
9. JDK 的基本信息不存在
10. JDK 的发行版文件信息不存在

获取升级信息成功

```text
Status: 200 OK
```

| Name | Type | Description |
|------|------|-------------|
| `installerToken` | `string` | Block Lang 平台为每个 installer 生成的 token。 |
| `appName` | `string` | 要部署的 Spring Boot jar 的完整名称，由 Block Lang 平台中的用户名和项目名组成，格式为 `@userName/projectName`。 |
| `appVersion` | `string` | 要部署的 Spring Boot jar 的版本号。 |
| `appFileName` | `string` | 要部署的 Spring Boot jar 的完整文件名，与发布中心的名字保持一致。 |
| `appRunPort` | `int` | Spring Boot jar 在服务器上的运行端口。 |
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

要注销的安装器不存在

```text
Status: 404 NOT FOUND
```

注销成功

```text
Status: 204 NO CONTENT
```

## `installer_config.toml` 结构

| Name | Description |
|------|-------------|
| `url` | Block Lang 软件发布中心的 URL，如 `https://blocklang.com`。 |
| `installer_token` | 为每一个 installer 生成的唯一 token。 |
| `server_token` | 用于唯一标识服务器，使用服务器的 MAC 地址。 |
| `app_name` | 要部署的 Spring Boot jar 的完整名称，由 Block Lang 平台中的用户名和项目名组成，格式为 `@userName/projectName`。 |
| `app_version` | 要部署的 Spring Boot jar 的版本号。 |
| `app_file_name` | 要部署的 Spring Boot jar 的完整文件名，与发布中心的名字保持一致。 |
| `app_run_port` | Spring Boot jar 在服务器上的运行端口。 |
| `jdk_name` | JDK 在 Block Lang 平台登记的名称。 |
| `jdk_version` | JDK 的完整版本号。 |
| `jdk_file_name` | JDK 的完整文件名，在服务器上 JDK 以此命名。 |
