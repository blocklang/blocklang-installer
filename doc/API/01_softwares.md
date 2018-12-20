# 下载软件

从软件发布中心下载软件。

```
GET /softwares?name={name}&version={version}
```

Parameters

| Name | Type | Description |
|------|------|-------------|
| `name` | `string` | **Required**. 软件名称。 |
| `version` | `string` | 完整版本号，如果未设置值，则获取最新版本。 |

Response

```
Status: 200 OK
```

软件的存放目录结构为

```
softwares
|---name
    |---version
        |---file
```

注意：
1. `name` 使用小写字母
2. `version` 是完整版本号
3. `file` 的名字必须使用官网提供的文件完整名称

如 JDK 的存放目录结构为

```
softwares
|---jdk
    |---11.0.1
        |---jdk-11.0.1_windows-x64_bin.zip
```

注意：jdk 使用的是压缩版，而不是安装版。

