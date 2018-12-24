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

### 在 Windows 上安装 BlockLang Installer

#### 安装

1. 在 Windows 上新建一个文件夹，如 `C:\BlockLang-Installer`。
1. 下载 [x86]() 二进制文件并存放在刚才创建的文件夹中，将二进制文件重命名为 `blocklang-installer.exe`。
1. 运行 "cmd" 命令提示符。
1. 在 BlockLang Installer 中注册一个待安装的软件，将 BlockLang Installer 与 https://blocklang.com 上的一个项目绑定在一起。
   1. 在 https://blocklang.com 上，为要绑定的项目生成一个 token。
   1. 运行注册命令：
   
        ```sh
        ./blocklang-installer.exe register
        ```

    1. 输入 BlockLang 网站 URL，直接按回车键，则默认值为 `https://blocklang.com`：

         ```
         请输入 Block Lang 站点 URL（默认值为 https://blocklang.com）
         https://blocklang.com
         ```

    1. 输入待绑定项目的 token：

         ```
         请输入待绑定项目的 token
         xxx
         ```

1. 启动 BlockLang Installer。
    
     ```sh
     ./blocklang-installer.exe start
     ```

#### 升级

1. 运行“cmd”命令提示符。
1. 停止 BlockLang Installer:

     ```sh
     cd C:\BlockLang-Installer
     ./blocklang-installer.exe stop
     ```
    
1. 下载最新版的 [x86]() 二进制文件并替换掉之前的可执行文件。
1. 启动 BlockLang Installer:

     ```sh
     ./blocklang-installer.exe start
     ```

#### 卸载

1. 运行“cmd”命令提示符。
2. 卸载，停止 BlockLang Installer 后，直接删除文件夹：

     ```sh
     cd C:\BlockLang-Installer
     ./blocklang-installer.exe stop
     cd ..
     rmdir /s BlockLang-Installer
     ```

## RESTful API

Installer 与软件中心交互的 REST API。

需要软件中心提供的有：
1. [下载软件](docs/API/01_softwares.md)

TODO: 提供一个获取安装软件信息的 API，然后根据获取到的信息来指定下载软件的名称和版本

## 社区

每晚7:00-9:00抖音和斗鱼同步直播

1. 斗鱼直播间： https://www.douyu.com/6140385
2. 抖音号：jinzhengwei
3. QQ群：619312757
