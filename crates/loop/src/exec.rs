//! Running commands: git, cargo, glassmind and agents, optionally inside a
//! bubblewrap sandbox.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// What a sandboxed process may see and write.
#[derive(Debug, Clone, Default)]
pub struct Sandbox {
    /// Directories it may write.
    pub writable: Vec<PathBuf>,
    /// Directories hidden from it (replaced by empty ones).
    pub hidden: Vec<PathBuf>,
    /// Bound over `/tmp`, so nothing is written to RAM-backed `/tmp`.
    pub tmp: Option<PathBuf>,
}

impl Sandbox {
    /// Wraps `program args` in `bwrap`: the whole filesystem read-only, the
    /// hidden directories empty, then the writable ones on top.
    pub fn wrap(&self, program: &str, args: &[String]) -> (String, Vec<String>) {
        let mut a: Vec<String> = vec![
            "--die-with-parent".into(),
            "--ro-bind".into(),
            "/".into(),
            "/".into(),
            "--dev".into(),
            "/dev".into(),
            "--proc".into(),
            "/proc".into(),
        ];
        for h in &self.hidden {
            let path = h.to_string_lossy().into_owned();
            if h.is_dir() {
                a.extend(["--tmpfs".into(), path]);
            } else if h.exists() {
                a.extend(["--ro-bind".into(), "/dev/null".into(), path]);
            }
        }
        if let Some(tmp) = &self.tmp {
            a.extend([
                "--bind".into(),
                tmp.to_string_lossy().into_owned(),
                "/tmp".into(),
            ]);
        }
        for w in &self.writable {
            let w = w.to_string_lossy().into_owned();
            a.extend(["--bind".into(), w.clone(), w]);
        }
        a.push("--".into());
        a.push(program.into());
        a.extend(args.iter().cloned());
        ("bwrap".into(), a)
    }
}

/// The result of a finished command.
#[derive(Debug, Clone)]
pub struct Output {
    pub ok: bool,
    pub stdout: String,
    pub stderr: String,
    pub timed_out: bool,
}

/// A command to run to completion (or until its timeout), capturing output.
pub struct Run<'a> {
    program: String,
    args: Vec<String>,
    cwd: &'a Path,
    env: Vec<(String, String)>,
    remove_env: Vec<String>,
    stdin: Option<String>,
    timeout: Option<Duration>,
    sandbox: Option<&'a Sandbox>,
}

impl<'a> Run<'a> {
    pub fn new(program: &str, args: &[String], cwd: &'a Path) -> Self {
        Run {
            program: program.to_owned(),
            args: args.to_vec(),
            cwd,
            env: Vec::new(),
            remove_env: Vec::new(),
            stdin: None,
            timeout: None,
            sandbox: None,
        }
    }

    pub fn env(mut self, key: &str, value: &str) -> Self {
        self.env.push((key.to_owned(), value.to_owned()));
        self
    }

    /// Removes every inherited variable whose name starts with `prefix`.
    pub fn remove_env_prefix(mut self, prefix: &str) -> Self {
        self.remove_env.push(prefix.to_owned());
        self
    }

    pub fn stdin(mut self, input: &str) -> Self {
        self.stdin = Some(input.to_owned());
        self
    }

    pub fn timeout(mut self, t: Duration) -> Self {
        self.timeout = Some(t);
        self
    }

    pub fn sandbox(mut self, s: Option<&'a Sandbox>) -> Self {
        self.sandbox = s;
        self
    }

    pub fn go(self) -> Result<Output, String> {
        let (program, args) = match self.sandbox {
            Some(s) => s.wrap(&self.program, &self.args),
            None => (self.program.clone(), self.args.clone()),
        };
        let mut cmd = Command::new(&program);
        cmd.args(&args)
            .current_dir(self.cwd)
            .stdin(if self.stdin.is_some() {
                Stdio::piped()
            } else {
                Stdio::null()
            })
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        for (k, _) in std::env::vars() {
            if self.remove_env.iter().any(|p| k.starts_with(p.as_str())) {
                cmd.env_remove(&k);
            }
        }
        for (k, v) in &self.env {
            cmd.env(k, v);
        }
        let mut child = cmd
            .spawn()
            .map_err(|e| format!("cannot start {program}: {e}"))?;
        if let (Some(input), Some(mut pipe)) = (self.stdin, child.stdin.take()) {
            use std::io::Write;
            std::thread::spawn(move || {
                let _ = pipe.write_all(input.as_bytes());
            });
        }
        // Read output on threads so a full pipe never blocks the child.
        let out_pipe = child.stdout.take();
        let err_pipe = child.stderr.take();
        let out_thread = std::thread::spawn(move || read_all(out_pipe));
        let err_thread = std::thread::spawn(move || read_all(err_pipe));
        let start = Instant::now();
        let mut timed_out = false;
        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break Some(status),
                Ok(None) => {
                    if self.timeout.is_some_and(|t| start.elapsed() > t) {
                        let _ = child.kill();
                        timed_out = true;
                        break child.wait().ok();
                    }
                    std::thread::sleep(Duration::from_millis(200));
                }
                Err(e) => return Err(e.to_string()),
            }
        };
        let stdout = out_thread.join().unwrap_or_default();
        let stderr = err_thread.join().unwrap_or_default();
        Ok(Output {
            ok: !timed_out && status.is_some_and(|s| s.success()),
            stdout,
            stderr,
            timed_out,
        })
    }
}

fn read_all<R: std::io::Read>(pipe: Option<R>) -> String {
    let mut s = String::new();
    if let Some(mut p) = pipe {
        let _ = p.read_to_string(&mut s);
    }
    s
}

/// Runs `git` in `repo` and returns its trimmed stdout.
pub fn git(repo: &Path, args: &[&str]) -> Result<String, String> {
    let args: Vec<String> = args.iter().map(|s| (*s).to_owned()).collect();
    let out = Run::new("git", &args, repo).go()?;
    if out.ok {
        Ok(out.stdout.trim().to_owned())
    } else {
        Err(format!("git {}: {}", args.join(" "), out.stderr.trim()))
    }
}

/// Runs cargo with the machine caps: 8 jobs, and 8 test threads.
pub fn cargo(dir: &Path, target: &Path, args: &[&str]) -> Result<Output, String> {
    let mut a: Vec<String> = args.iter().map(|s| (*s).to_owned()).collect();
    let tests = a.first().is_some_and(|c| c == "test");
    a.insert(1, "-j".into());
    a.insert(2, "8".into());
    if tests {
        a.extend(["--".into(), "--test-threads".into(), "8".into()]);
    }
    Run::new("cargo", &a, dir)
        .env("CARGO_TARGET_DIR", &target.to_string_lossy())
        .timeout(Duration::from_secs(1800))
        .go()
}
