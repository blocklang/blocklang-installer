# Changelog

## 2018-12-17

搭建一个 CLI 项目。
在控制台中输入 `installer update` 命令后，在控制台打印出“更新成功”。

```sh
$ installer update
更新成功
```

## 2018-12-19

1. 实现从模拟服务器下载文件功能

## 2018-12-21

1. 解压 zip 文件

## 2018-12-23

1. 在 Windows 环境，用 Rust 启动 Jar 文件
    1. 在后台运行 jar
    2. 保存进程 id
    3. 根据进行 id 关闭运行 jar 的进程

## 2018-12-24

1. 设计交互命令
2. 在 Linux 环境，用 Rust 启动 Jar 文件
    1. 在后台运行 jar
    2. 保存进程 id
    3. 根据进行 id 关闭运行 jar 的进程

## 2018-12-27

1. 开发往 Block Lang 平台注册部署服务器的 REST API
2. 将注册通过的部署信息存储在 `config.toml` 配置文件中

## 2018-12-28

1. 支持在 Windows 上获取服务器 IP 地址、MAC 地址

## 2018-12-29

1. 开发 `register` 命令，详见 [install on windows API](docs/install/windows.md)
2. 开发 `start` 命令
   1. 检查当前版本的 Spring Boot jar 和依赖的 JDK 在 `prod` 文件夹下是否已存在，若未没有则先下载并解压
   2. 启动 Spring Boot jar

## 2018-12-30

1. 实现 windows 版
   1. 根据端口号获取进程 id 的函数；
   2. 根据进程 id kill 进程的函数；
2. 开发 `stop` 命令
   1. 根据占用的端口停止 Spring Boot jar，根据进程 ID 的话，可能会遇到 Installer 被意外关闭的情况

## 2018-12-31

1. 实现 linux 版 `kill_process` 函数
2. 重构代码，将 lib.rs 中的代码拆分到不同的模块中
3. 开发 `update` 命令

## 2019-01-01

1. 支持获取服务器的操作系统名和版本号
2. 添加 CI 工具(初步添加 travis-ci 和 appveyor 配置文件)

## 2019-01-02

1. 实现 linux 版 `util::process::get_id` 函数
2. 完善 `config::save` 函数的测试用例，修复可能两个测试函数同时处理 `config.toml` 引起的断言不确定的问题
3. 支持在 linux 环境下获取服务器的 IP 地址、MAC 地址和 CPU 架构信息
4. 支持在 Windows 上获取服务器的 CPU 架构信息
5. 修复在 linux 下 `download_success` 测试用例未通过的 bug

## 2019-01-06

1. 调整 installers API，明确 registration token 和 installer token，以支持在一台应用服务器上安装多个 installer

## 2019-01-07

1. 调整 `register` 子命令，添加端口号，并支持在一台服务器上注册多个 installer
2. 添加 `list` 子命令，如果没有注册 installer 时，要给出友好的提示信息
3. 将 `start` 子命令调整为 `run --port <port>` 子命令
4. 添加 `run --all` 子命令
5. 添加 `update --all` 和 `update --port <port>` 子命令
6. 添加 `stop --all` 和 `stop --port <port>` 子命令

## 2019-01-08

1. 添加 `unregister` 命令，支持 `port` 和 `all` 选项

## 2019-01-12

1. 在注册信息中添加 `target_os` 属性

## TODO

1. 在 windows 和 linux 下测试完整流程
   1. 下载 JDK
   2. 下载 Spring boot jar
   3. 移动 JDK 和 Spring boot jar 到 prod 文件夹下
   4. 解压 JDK
   5. 将 JDK 设置到环境变量中
   6. 启动 jar
2. 或者将启动的 spring boot jar 的进程 id 存在文件中（依然使用根据端口号获取进程 id）
3. 命令执行失败后，使用的是系统自带的提示信息，需优化这些提示信息，让用户了解出了什么错，并知道正确的应该怎么做
4. 在 app_name 中要包含用户名，这样就能确保名称的唯一，如 `@user_name/project_name`
5. 支持自动将 build 结果发到 github 的 release 中
6. 在用户输入端口后，添加校验逻辑，确认
   1. 端口号是不是有效的数字类型；
   2. 端口号是不是已在 `config.toml` 中配置（配置了，但可能还没有运行）；
   3. 端口号是否已被占用，即当前有程序运行在该端口上
7. 在 `config.toml` 中新增一个字段，标识出 APP 的运行状态 `RUNNING`
8. lock `config.toml` 文件，不允许手工编辑