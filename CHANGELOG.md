# Changelog

## 2018-12-17

搭建一个 CLI 项目。
在控制台中输入 `installer update` 命令后，在控制台打印出“更新成功”。

```
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
2. 在 windows 下实现完整流程
   1. 下载 JDK
   2. 下载 Spring boot jar
   3. 移动 JDK 和 Spring boot jar 到 prod 文件夹下
   4. 解压 JDK
   5. 将 JDK 设置到环境变量中
   6. 启动 jar

## TODO

1. 在 Linux 环境，用 Rust 启动 Jar 文件
    1. 在后台运行 jar
    2. 保存进程 id
    3. 根据进行 id 关闭运行 jar 的进程
2. 根据端口号获取进程标识，然后关闭进程
3. 移动 JDK 和 Spring Boot jar 到 prod 文件夹下
4. 解压 JDK