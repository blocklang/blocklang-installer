# installer

从软件中心下载软件，并在应用服务器上安装软件环境和软件。
本软件部署在应用服务器上。

## 功能

![结构图](images/installer.png)

1. 从软件中心下载JDK和Jar文件；
1. 在应用服务器上安装JDK和Jar文件；
1. 启动Jar文件

### 功能点说明

#### 2018-12-17

搭建一个 CLI 项目。
在控制台中输入 `installer update` 命令后，在控制台打印出“更新成功”。

```
$ installer update
更新成功
```

#### 2018-12-19

1. 实现从模拟服务器下载文件功能

#### 2018-12-21

1. 解压 zip 文件

#### TODO

1. 移动 JDK 和 Spring Boot jar 到 prod 文件夹下
1. 解压 JDK
1. 启动 Spring Boot jar

## RESTful API

Installer 与软件中心交互的 REST API。

需要软件中心提供的有：
1. [下载软件](doc/API/01_softwares.md)

TODO: 提供一个获取安装软件信息的 API，然后根据获取到的信息来指定下载软件的名称和版本

## 社区

每晚7:00-9:00抖音和斗鱼同步直播

1. 斗鱼直播间： https://www.douyu.com/6140385
2. 抖音号：jinzhengwei
3. QQ群：619312757
