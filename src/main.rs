use std::{fmt, fs::File, process::exit};
use std::{io::BufWriter, io::Write, path::PathBuf};

use clap::Clap;

#[derive(Clap, Debug, PartialEq)]
enum ServiceType {
    Simple,
    Forking,
    Oneshot,
    Dbus,
    Notify,
    Idle,
}

impl fmt::Display for ServiceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Simple => write!(f, "{}", "simple"),
            Self::Forking => write!(f, "{}", "forking"),
            Self::Oneshot => write!(f, "{}", "oneshot"),
            Self::Dbus => write!(f, "{}", "dbus"),
            Self::Notify => write!(f, "{}", "notify"),
            Self::Idle => write!(f, "{}", "idle"),
        }
    }
}

#[derive(Clap, Debug, PartialEq)]
enum RestartType {
    No,
    Always,
    OnSuccess,
    OnFailure,
    OnAbnormal,
    OnAbort,
    OnWatchdog,
}

impl fmt::Display for RestartType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::No => write!(f, "{}", "no"),
            Self::Always => write!(f, "{}", "always"),
            Self::OnSuccess => write!(f, "{}", "on-success"),
            Self::OnFailure => write!(f, "{}", "on-failure"),
            Self::OnAbnormal => write!(f, "{}", "on-abnormal"),
            Self::OnAbort => write!(f, "{}", "on-abort"),
            Self::OnWatchdog => write!(f, "{}", "on-watchdog"),
        }
    }
}

#[derive(Clap, Debug, PartialEq)]
#[clap(author, about, version)]
struct Service {
    // [Unit]
    #[clap(short, long)]
    description: Option<String>,
    #[clap(long)]
    after: Vec<String>,
    #[clap(long)]
    conflicts: Vec<String>,
    #[clap(long)]
    requires: Vec<String>,
    #[clap(long)]
    on_failure: Option<String>,

    // [Service]
    #[clap(short = 't', long = "type", arg_enum, default_value = "simple")]
    service_type: ServiceType,
    #[clap(short, long)]
    exec_start: PathBuf,
    #[clap(long)]
    exec_reload: Option<PathBuf>,
    #[clap(long)]
    exec_stop: Option<PathBuf>,
    #[clap(long, arg_enum)]
    restart: Option<RestartType>,
    #[clap(long)]
    restart_sec: Option<usize>,
    #[clap(short, long)]
    user: Option<String>,
    #[clap(short, long)]
    group: Option<String>,

    // [Install]
    #[clap(short, long)]
    wanted_by: Option<String>,

    #[clap(short, long, default_value = "/etc/systemd/system/")]
    output: PathBuf,
    #[clap(short, long)]
    name: String,
}

impl fmt::Display for Service {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut string = String::new();

        string.push_str("[Unit]\n");
        if let Some(d) = &self.description {
            string.push_str(&format!("Description={}\n", d));
        }
        for a in &self.after {
            string.push_str(&format!("After={}\n", a));
        }
        for c in &self.conflicts {
            string.push_str(&format!("Conflicts={}\n", c));
        }
        for r in &self.requires {
            string.push_str(&format!("Requires={}\n", r));
        }
        if let Some(f) = &self.description {
            string.push_str(&format!("OnFailure={}\n", f));
        }

        string.push_str("\n[Service]\n");
        string.push_str(&format!("Type={}\n", &self.service_type));
        string.push_str(&format!("ExecStart={}\n", &self.exec_start.display()));
        if let Some(e) = &self.exec_reload {
            string.push_str(&format!("ExecReload={}\n", e.display()));
        }
        if let Some(e) = &self.exec_stop {
            string.push_str(&format!("ExecStop={}\n", e.display()));
        }
        if let Some(r) = &self.restart {
            string.push_str(&format!("Restart={}\n", r));
        }
        if let Some(r) = &self.restart_sec {
            string.push_str(&format!("RestartSec={}\n", r));
        }
        if let Some(u) = &self.user {
            string.push_str(&format!("User={}\n", u));
        }
        if let Some(g) = &self.group {
            string.push_str(&format!("Group={}\n", g));
        }

        string.push_str("\n[Install]\n");
        if let Some(w) = &self.wanted_by {
            string.push_str(&format!("WantedBy={}\n", w));
        }

        write!(f, "{}", string)
    }
}

fn main() {
    let opt = Service::parse();

    if !opt.exec_start.exists() {
        println!("executable {} does not exist", opt.exec_start.display());
        exit(1);
    }

    let path = opt.output.join(format!("{}.service", opt.name));
    let mut writer = match File::create(&path) {
        Ok(f) => BufWriter::new(f),
        Err(_) => {
            println!("Error creating file {}", path.display());
            exit(1);
        }
    };

    match writer.write(opt.to_string().as_bytes()) {
        Ok(_) => println!("Wrote service file {}", path.display()),
        Err(_) => {
            println!("Error writing file {}", path.display());
            exit(1);
        }
    }
}
