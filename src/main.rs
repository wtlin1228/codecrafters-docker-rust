use anyhow::Context;
use std::io::copy;

// Usage: your_docker.sh run <image> <command> <arg1> <arg2> ...
fn main() -> anyhow::Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let command = &args[3];
    let command_args = &args[4..];
    let output = std::process::Command::new(command)
        .args(command_args)
        .output()
        .with_context(|| {
            format!(
                "Tried to run '{}' with arguments {:?}",
                command, command_args
            )
        })?;

    if output.status.success() {
        let mut process_stdout = &output.stdout[..];
        let mut stdout = std::io::stdout().lock();
        copy(&mut process_stdout, &mut stdout)
            .context("pipe process's stdout into parent's stdout")?;

        let mut process_stderr = &output.stderr[..];
        let mut stderr = std::io::stderr().lock();
        copy(&mut process_stderr, &mut stderr)
            .context("pipe process's stderr into parent's stderr")?;
    } else {
        std::process::exit(output.status.code().unwrap_or(1));
    }

    Ok(())
}
