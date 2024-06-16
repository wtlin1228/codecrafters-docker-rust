use anyhow::Context;
use std::process::Stdio;
use tempfile::tempdir;

// Usage: your_docker.sh run <image> <command> <arg1> <arg2> ...
fn main() -> anyhow::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let command = &args[3];
    let command_args = &args[4..];

    let temp_dir = tempdir().context("create tempdir")?;

    // create /dev/null file
    let path_dev_null = temp_dir.path().join("dev/null");
    std::fs::create_dir_all(path_dev_null.parent().unwrap()).context("create dir /dev")?;
    std::fs::File::create(path_dev_null).context("create file /dev/null")?;

    // copy command
    let path_command = temp_dir
        .path()
        .join(command.strip_prefix("/").unwrap_or(command));
    std::fs::create_dir_all(path_command.parent().unwrap())
        .context(format!("create dir {:?}", path_command.parent().unwrap()))?;
    std::fs::copy(command, path_command).context("copy command")?;

    // chroot jail
    std::os::unix::fs::chroot(temp_dir.path()).context("chroot into temporary directory")?;

    // process isolation
    #[cfg(target_os = "linux")]
    unsafe {
        libc::unshare(libc::CLONE_NEWPID)
    };

    let status = std::process::Command::new(command)
        .args(command_args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .with_context(|| {
            format!(
                "Tried to run '{}' with arguments {:?}",
                command, command_args
            )
        })?;

    if let Some(code) = status.code() {
        std::process::exit(code);
    }

    Ok(())
}
