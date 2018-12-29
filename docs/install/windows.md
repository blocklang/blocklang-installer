# 在 Windows 上安装 BlockLang Installer

## 安装

1. 在 Windows 上新建一个文件夹，如 `C:\BlockLang-Installer`。
2. 下载 [x86]() 二进制文件并存放在刚才创建的文件夹中，将二进制文件重命名为 `blocklang-installer.exe`。
3. 运行 "cmd" 命令提示符。
4. 在 BlockLang Installer 中注册一个待安装的软件，将 BlockLang Installer 与 https://blocklang.com 上的一个项目绑定在一起。
   1. 在 https://blocklang.com 上，为要绑定的项目生成一个 token。
   2. 运行注册命令：
   
        ```sh
        ./blocklang-installer.exe register
        ```

    3. 输入 BlockLang 网站 URL，直接按回车键，则默认值为 `https://blocklang.com`：

         ```
         请输入 Block Lang 站点 URL（默认值为 https://blocklang.com）
         https://blocklang.com
         ```

    4. 输入待绑定项目的 token：

         ```
         请输入待绑定项目的 token
         xxx
         ```

5. 启动 BlockLang Installer，会启动 BlockLang Installer 中的 HTTP REST 服务，并运行 spring boot jar 项目。
    
     ```sh
     ./blocklang-installer.exe start
     ```
     注意，`stop` 命令会停止 HTTP REST 服务，并停止 spring boot jar。

## 升级

1. 运行“cmd”命令提示符。
2. 停止 BlockLang Installer:

     ```sh
     cd C:\BlockLang-Installer
     ./blocklang-installer.exe stop
     ```
    
3. 下载最新版的 [x86]() 二进制文件并替换掉之前的可执行文件。
4. 启动 BlockLang Installer:

     ```sh
     ./blocklang-installer.exe start
     ```

## 卸载

1. 运行“cmd”命令提示符。
2. 卸载，停止 BlockLang Installer 后，直接删除文件夹：

     ```sh
     cd C:\BlockLang-Installer
     ./blocklang-installer.exe stop
     cd ..
     rmdir /s BlockLang-Installer
     ```
