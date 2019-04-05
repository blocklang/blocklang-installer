# 下载软件

从软件发布中心下载软件，支持断点续传。

```text
GET /apps?appName={appName}&version={version}&targetOs={targetOs}&arch={arch}
```

Parameters

| Name       | Type     | Description                                      |
| ---------- | -------- | ------------------------------------------------ |
| `appName`  | `string` | **Required**. 软件名称。                         |
| `version`  | `string` | **Required**. 完整版本号。                       |
| `targetOs` | `string` | **Required**. 操作系统名：`linux` 或 `windows`。 |
| `arch`     | `string` | **Required**. CPU 架构。                         |

注意：要根据服务器的操作系统下载对应的软件。

Response

下载整个文件时，返回

```text
Status: 200 OK
```

断点续传时，返回

```text
Status: 206 Partial Content
```

未找到下载文件时，返回

```text
Status: 404 Not Found
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

### 断点续传

在 `download.toml` 文件中记录下载文件的 MD5 值，用于断点续传时，文件是否被修改。记录格式为：

```text
[[files]]
name = "jdk"
version = "11.0.2"
md5 = "xxxx"
```

1. 在开始下载时，记录文件的 md5 信息
2. 断点续传时，不修改文件的 md5 信息
3. 下载完成后，删除文件的 md5 信息
