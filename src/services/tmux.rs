use crate::error::Result;
use crate::services::command::CommandRunner;

pub fn session_exists(session: &str) -> bool {
    CommandRunner::tmux().success(&["has-session", "-t", session])
}

pub fn create_session(session: &str) -> Result<()> {
    CommandRunner::tmux().run(&["new-session", "-d", "-s", session])
}

pub fn create_window(session: &str, window: &str, cwd: &str, command: &str) -> Result<()> {
    let target = format!("{}:", session);

    // 先创建窗口（不带命令），这样会启动交互式 shell
    CommandRunner::tmux().run(&[
        "new-window",
        "-t",
        &target,
        "-n",
        window,
        "-c",
        cwd,
    ])?;

    // 然后用 send-keys 发送命令，这样 shell 别名也能生效
    // 使用 -l (literal) 选项确保命令中的空格和特殊字符被正确发送
    let window_target = format!("{}:{}", session, window);
    CommandRunner::tmux().run(&[
        "send-keys",
        "-t",
        &window_target,
        "-l",
        command,
    ])?;
    // 单独发送 Enter 键
    CommandRunner::tmux().run(&["send-keys", "-t", &window_target, "Enter"])
}

pub fn kill_window(session: &str, window: &str) -> Result<()> {
    let target = format!("{}:{}", session, window);
    CommandRunner::tmux().run(&["kill-window", "-t", &target])
}

pub fn window_exists(session: &str, window: &str) -> bool {
    let target = format!("{}:{}", session, window);
    CommandRunner::tmux().success(&["select-window", "-t", &target])
}

/// 如果窗口存在则关闭，返回是否执行了关闭操作
pub fn kill_window_if_exists(session: &str, window: &str) -> Result<bool> {
    if window_exists(session, window) {
        kill_window(session, window)?;
        Ok(true)
    } else {
        Ok(false)
    }
}
