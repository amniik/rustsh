

# rustsh

A Unix-like job control shell written in **Rust**.

`rustsh` is a lightweight command-line shell that supports built-in commands, external program execution, pipelines, I/O redirection, and foreground/background job control.

## Features

- **Built-in commands**
  - `cd` – Change the current working directory.
  - `ls` – List directory contents.
  - `type` – Identify command types.
  - `jobs` – List active jobs.
  - `fg` – Bring a background or stopped job to the foreground.
  - `bg` – Resume a stopped job in the background.
  - `exit` – Exit the shell.

- **External program execution** – Execute external programs using the system's executable lookup and process execution mechanisms.

- **Pipelines** – Connect multiple commands using pipes.

- **I/O redirection**
  - `>` – Redirect standard output.
  - `>>` – Append standard output to a file.
  - `<` – Redirect standard input.
  - `2>` – Redirect standard error.

- **Job control**
  - Run commands in the background using `&`.
  - Manage foreground and background processes.
  - Suspend and resume jobs.
  - Handle terminal signals and process groups.

## Usage

Clone the repository:

```bash
git clone https://github.com/amniik/rustsh.git
cd rustsh
```

Build and run:

```bash
cargo build --release
./target/release/rustsh
```

## Development

Run:

```bash
cargo run
```

Run tests:

```bash
cargo test
```

## Architecture

See the [Architecture Documentation](doc/arch.md) for an overview of the shell's modules and command execution flow.
