# BlockLang Installer Commands

## 注册相关命令

* `blocklang-installer register`
* `blocklang-installer list`
* `blocklang-installer unregister`

### blocklang-installer register

使用注册 token，向 Block Lang 平台注册一个 installer，并返回一个 installer token，来唯一标识 installer。

### blocklang-installer list

列出所有存储在 `config.toml` 中的 installer 信息，包括 Block Lang 平台的 URL、installer token 和运行 APP 实例的端口号。

执行 `blocklang-installer list` 后显示以下信息：

```sh
Port=80    Token=t0k3n    URL=https://blocklang.com
```

### blocklang-installer unregister

#### 根据 port 来注销一个 installer：

```sh
blocklang-installer unregister --port 8080
```

如果 installer 已启动一个 App 实例，则先停止该 APP 实例，然后再从 `config.toml` 文件中删除该 installer 的配置信息。

#### 注销所有 installer

```sh
blocklang-installer unregister --all
```

停止在 `config.toml` 中配置的所有 installer 启动的 APP 实例，并从 `config.toml` 中删除所有配置信息。

注意：注销成功后，并不会删除已下载的 JDK 和 Spring boot jar 等文件。

## 运行 APP 相关命令

因为一个端口上只能运行一个 APP，所以在应用服务器上，端口号也可以作为 installer 的唯一标识。本系列命令使用端口号来唯一定位应用服务器上的 installer。

* `blocklang-installer run`
* `blocklang-installer stop`
* `blocklang-installer update`

### blocklang-installer run

#### 通过指定端口号，运行单个 APP

```sh
# 运行与80端口关联的 APP，并运行在80端口上。
blocklang-installer run --port 80
```

#### 运行 `config.toml` 配置文件中的所有 APP

```sh
blocklang-installer run --all
```

注意，如果某 APP 正处于运行状态，则跳过，而不会重启。

### blocklang-installer stop

#### 通过指定端口号，停止单个 APP

```sh
# 停止运行在 80 端口上的 APP
blocklang-installer stop --port 80
```

#### 停止 `config.toml` 配置文件中的所有 APP

```sh
blocklang-installer stop --all
```

### blocklang-installer update

#### 通过指定端口号，升级单个 APP

```sh
# 升级运行在 80 端口上的 APP
blocklang-installer update --port 80
```

注意，不论 APP 是否运行，都会下载并安装最新 APP。如果之前未运行，则升级完后并不会运行该程序，而是打印 APP 的运行状态，来提醒用户。

#### 升级 `config.toml` 配置文件中的所有 APP

```sh
blocklang-installer update --all
```

注意，升级 APP，并不会改变程序的运行状态，升级完每个 APP 后都会显示 APP 的运行状态。
