use std::process::{Command, Child};
use std::path::Path;
use std::fs;

/// 在后台运行 Spring boot jar 文件，并返回进程
/// 
/// 默认将 JDK 与要运行的 spring boot jar 放在同一个文件夹中,
/// 其中 `jar_file_path` 指 spring boot jar 的存放路径，
/// `jdk_path` 指 jdk 的存放路径。
/// 
/// Examples
/// 
/// ```no_run
/// use installer::jar::run_spring_boot;
/// 
/// fn main() {
///     run_spring_boot("prod/app1/demo-0.0.1-SNAPSHOT.jar", "prod/app1/temp/jdk-11.0.1", 80);
/// }
/// ```
pub fn run_spring_boot(
    jar_file_path: &str, 
    jdk_path: &str,
    port: u32) -> Child {

    if cfg!(target_os = "windows") {
        // 注意，在 windows 操作系统中，使用 `javaw`，不使用 `java`
        // 因为 `java` 会在命令行中启动新的命令行，因此无法直接跟踪到
        // 使用 `java` 命令运行的 jar。而 `javaw` 正是为解决此类问题诞生的。
        // 而在 linux 环境下， `java` 命令不会启动一个新的命令行，
        // 所以在 linux 环境下，并不存在 `javaw` 命令。
        // 详见 https://stackoverflow.com/questions/14331406/why-javaw-is-not-found-on-my-java-installation-on-ubuntu
        Command::new("javaw")
            .env("PATH", Path::new(jdk_path).join("bin"))
            .arg("-jar")
            .arg(jar_file_path)
            .arg("--server.port")
            .arg(port.to_string())
            .spawn()
            .expect("failed to run javaw -jar")
    } else {
        // 设置权限，初次运行时会提示没有权限
        let java_executable_path = Path::new(jdk_path).join("bin").join("java");
        let java_executable_path = java_executable_path
            .to_str()
            .expect("parse jdk/bin/java path error");

	    // 为 jdk/bin/java 设置可执行权限
        set_executable_permission(java_executable_path)
            .expect("failed to set jdk/bin/java permission");
        // 为 spring boot jar 设置可执行权限
        set_executable_permission(jar_file_path)
            .expect("failed to set sping boot jar permission");
	
        // linux 下直接运行 java -jar 就是生成一个新的后台进程
        // 当关闭 installer 进程后，运行 java -jar 的进程依然存在
        Command::new("java")
            .env("PATH", Path::new(jdk_path).join("bin"))
            .arg("-jar")
            .arg(jar_file_path)
            .arg("--server.port")
            .arg(port.to_string())
            .spawn()
            .expect("failed to run java -jar")
    }
}

// 在 linux 等操作系统下，让文件具有可执行权限。
fn set_executable_permission(path: &str) -> std::io::Result<()> {
    let mut perms = fs::metadata(path)?.permissions();
    perms.set_readonly(false);
    fs::set_permissions(path, perms)?;

    Ok(())
}

/// 停止运行 spring boot jar
/// 
/// 这里是通过直接杀死进程来停止 spring boot 项目的。
/// 
/// Examples
/// 
/// ```no_run
/// use installer::jar::run_spring_boot;
/// use installer::jar::stop_spring_boot;
/// 
/// fn main() {
///     let mut process = run_spring_boot("prod/app1/demo-0.0.1-SNAPSHOT.jar", "prod/app1/temp/jdk-11.0.1", 80);
///     stop_spring_boot(&mut process);
/// }
/// ```
pub fn stop_spring_boot(process: &mut Child) -> std::io::Result<()> {
    process.kill()
}
