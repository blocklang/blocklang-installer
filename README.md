# BlockLang Installer

BlockLang Installer 是一款云部署工具。
BlockLang Installer 工具安装在应用服务器上，
从软件发布中心 https://blocklang.store 下载软件，
并在应用服务器上安装软件环境和软件。

## 功能

![结构图](images/installer.png)

1. 从软件中心下载JDK和Jar文件；
2. 在应用服务器上安装JDK和Jar文件；
3. 启动Jar文件

欲了解功能更新日志，详见 [CHANGELOG.md](CHANGELOG.md)。

## 安装 BlockLang Installer

* [在 Windows 上安装](docs/install/windows.md)

## 升级 Spring Boot Jar

使用 `blocklang-installer update` 命令将 Spring Boot Jar 升级到最新版本。

1. 尝试下载最新版 Spring Boot jar
2. 如果已经是最新版，则在控制台给出提示，并结束命令
3. 如果发现新版则
   1. 下载最新版的 Spring Boot jar
   2. 停止运行的 Spring Boot Jar
   3. 启动新版 Spring Boot jar

注意：此命令只用于升级 Spring Boot Jar，没有升级 BlockLang Installer 软件。

## RESTful API

Installer 与软件中心交互的 REST API。

需要软件中心提供的有：

1. [注册部署信息](docs/API/01_installers.md)
2. [下载软件](docs/API/02_softwares.md)

## 社区

每晚7:00-9:00抖音和斗鱼同步直播

1. 斗鱼直播间： https://www.douyu.com/6140385
2. 抖音号：jinzhengwei
3. QQ群：619312757
