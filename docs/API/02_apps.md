# 下载软件

从软件发布中心下载软件。

```text
GET /apps?name={name}&version={version}&os={os}
```

Parameters

| Name | Type | Description |
|------|------|-------------|
| `name` | `string` | **Required**. 软件名称。 |
| `version` | `string` | 完整版本号，如果未设置值，则获取最新版本。 |
| `os` | `string` | **Required**.操作系统名：`linux` 或 `windows`。 |

注意：要根据服务器的操作系统下载对应的软件。

Response

```text
Status: 200 OK
```

## 软件的存放目录结构

### 下载的文件

下载的文件存储在 `apps` 文件夹下。

```text
apps
|---name
    |---version
        |---file
```

注意：

1. `name` 使用小写字母
2. `version` 是完整版本号
3. `file` 的名字必须使用官网提供的文件完整名称

如 JDK 的存放目录结构为

```text
apps
|---jdk
    |---11.0.1
        |---jdk-11.0.1_windows-x64_bin.zip
```

注意：jdk 使用的是压缩版，而不是安装版。

### 要运行的文件

运行的文件和下载的文件分开存储，运行的文件存在 `prod` 文件夹下。

```text
prod
|---name
    |---version
        |---file/folder
```
