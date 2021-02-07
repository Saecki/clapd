use std::{fmt, fs::File, io::BufWriter, path::Path, process::exit};
use std::{io::Write, path::PathBuf};

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
    #[clap(short, long)]
    before: Vec<String>,
    #[clap(short, long)]
    after: Vec<String>,
    #[clap(short, long)]
    conflicts: Vec<String>,
    #[clap(short, long)]
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
    #[clap(short, long, default_value = "multi-user.target")]
    wanted_by: String,

    // Timer
    #[clap(short = 'T', long)]
    timer: bool,
    #[clap(short, long)]
    persistent: bool,
    #[clap(long)]
    on_calendar: Option<String>,
    #[clap(long)]
    on_unit_active_sec: Option<String>,
    #[clap(long)]
    on_unit_inactive_sec: Option<String>,
    #[clap(long)]
    accuracy_sec: Option<String>,

    #[clap(short, long, default_value = "/etc/systemd/system/")]
    output: PathBuf,

    #[clap(long)]
    no_check: bool,
    #[clap(short, long)]
    name: String,
}

impl Service {
    fn service(&self) -> String {
        let mut string = String::new();

        string.push_str("[Unit]\n");
        if let Some(d) = &self.description {
            string.push_str(&format!("Description={}\n", d));
        }
        for b in &self.before {
            string.push_str(&format!("Before={}\n", b));
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
        let abs_exec_start = canonicalize(&self.exec_start);
        string.push_str(&format!("ExecStart={}\n", abs_exec_start.display()));
        if let Some(e) = &self.exec_reload {
            string.push_str(&format!("ExecReload={}\n", canonicalize(e).display()));
        }
        if let Some(e) = &self.exec_stop {
            string.push_str(&format!("ExecStop={}\n", canonicalize(e).display()));
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
        string.push_str(&format!("WantedBy={}\n", self.wanted_by));

        string
    }

    fn timer(&self) -> String {
        let mut string = String::new();

        string.push_str("[Unit]\n");

        string.push_str("\n[Timer]\n");
        if let Some(c) = &self.on_calendar {
            string.push_str(&format!("OnCalendar={}\n", c));
        }
        if let Some(c) = &self.on_unit_active_sec {
            string.push_str(&format!("OnUnitActiveSec={}\n", c));
        }
        if let Some(c) = &self.on_unit_inactive_sec {
            string.push_str(&format!("OnUnitInactiveSec={}\n", c));
        }
        string.push_str(&format!("Persistent={}\n", self.persistent));

        string.push_str("\n[Install]\n");
        string.push_str("WantedBy=timers.target\n");

        string
    }
}

fn main() {
    let opt = Service::parse();

    if !opt.exec_start.exists() && !opt.no_check {
        println!("Executable {} does not exist", opt.exec_start.display());
        exit(1);
    }

    if opt.timer {
        if opt.on_calendar.is_none() {
            println!("Timer flag was specified but no OnCalendar");
            exit(1);
        }
    }

    let service_path = opt.output.join(format!("{}.service", opt.name));
    let mut service_writer = match File::create(&service_path) {
        Ok(f) => BufWriter::new(f),
        Err(_) => {
            println!("Error creating sevice file {}", service_path.display());
            exit(1);
        }
    };
    match service_writer.write(opt.service().as_bytes()) {
        Ok(_) => println!("Wrote service file {}", service_path.display()),
        Err(_) => {
            println!("Error writing service file {}", service_path.display());
            exit(1);
        }
    }

    let timer_path = opt.output.join(format!("{}.timer", opt.name));
    let mut timer_writer = match File::create(&timer_path) {
        Ok(f) => BufWriter::new(f),
        Err(_) => {
            println!("Error creating timer file {}", timer_path.display());
            exit(1);
        }
    };
    match timer_writer.write(opt.timer().as_bytes()) {
        Ok(_) => println!("Wrote timer file {}", timer_path.display()),
        Err(_) => {
            println!("Error writing timer file {}", timer_path.display());
            exit(1);
        }
    }
}

fn canonicalize(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or(path.to_owned())
}
