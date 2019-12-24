
# BlockLang Installer

|              **Linux**              |            **Windows**             |                      **Github Action**                      |
| :---------------------------------: | :--------------------------------: | :---------------------------------------------------------: |
| [![Trivas-ci][tci badge]][tci link] | [![Appveyor][avy badge]][avy link] | [![Github Action][github action badge]][github action link] |

BlockLang Installer 是一款自动化部署工具，专用于部署 Spring boot 项目。

## 功能

核心功能：

1. 从 [Block Lang 平台](https://blocklang.com) 下载 JDK 和 spring boot jar 文件；
2. 安装 JDK；
3. 启动 spring boot jar。

功能示意图：

![结构图](images/installer.png)

注意：可参考下述的 REST API 文档搭建自己的软件中心。

## BlockLang Installer Commands

BlockLang Installer 是一个 CLI 程序，有 6 个命令：

* 注册相关命令
  1. `blocklang-installer register`
  2. `blocklang-installer list`
  3. `blocklang-installer unregister`
* 运行 APP 相关命令
  1. `blocklang-installer run`
  1. `blocklang-installer stop`
  1. `blocklang-installer update`

详见 [CLI Commands](docs/commands.md)。

## 安装

BlockLang Installer 安装在部署 Spring boot jar 的应用服务器上。支持 Windows 和 Linux。

* [在 Windows 上安装](docs/install/windows.md)
* 在 Linux 上安装 (TBD)

## REST API

BlockLang Installer 需要与 [Block Lang 平台](https://blocklang.com) 交互，使用 REST API 向 [Block Lang](https://blocklang.com) 请求数据：

1. [注册和更新项目信息](docs/API/01_installers.md)
2. [下载软件](docs/API/02_apps.md)

欲了解更多功能，详见 [CHANGELOG.md](CHANGELOG.md)。

<!-- prettier-ignore -->
[tci badge]: https://travis-ci.org/blocklang/blocklang-installer.svg?branch=master
[tci link]: https://travis-ci.org/blocklang/blocklang-installer
[avy badge]: https://ci.appveyor.com/api/projects/status/bm3mrtr4p0vu8kx8?svg=true
[avy link]: https://ci.appveyor.com/project/xiaohulu/blocklang-installer
[github action badge]: https://github.com/blocklang/blocklang-installer/workflows/Rust/badge.svg
[github action link]: https://github.com/blocklang/blocklang-installer/actions
