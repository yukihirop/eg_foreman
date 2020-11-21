use crate::log;
use crate::process::Process;
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use signal_hook::{iterator::Signals, SIGALRM, SIGHUP, SIGINT, SIGTERM};
use std::process::exit;
use std::sync::{Arc, Mutex};

pub fn handle_signal(
    procs: Arc<Mutex<Vec<Arc<Mutex<Process>>>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let signals = Signals::new(&[SIGALRM, SIGHUP, SIGINT, SIGTERM])?;

    for sig in signals.forever() {
        match sig {
            SIGINT => {
                log::output("system", "ctrl-c detected");
                log::output("system", "sending SIGTERM for children");
                for proc in procs.lock().unwrap().iter() {
                    let proc = proc.lock().unwrap();
                    let child = &proc.child;

                    log::output(
                        "system",
                        &format!("sending SIGTERM for {} at pid {}", &proc.name, &child.id()),
                    );

                    if let Err(e) = signal::kill(Pid::from_raw(child.id() as i32), Signal::SIGTERM)
                    {
                        log::error("system", &e);
                        log::output("system", "exit 1");
                        exit(1);
                    }
                }
                log::output("system", "exit 0");
                exit(0)
            }
            _ => (),
        }
    }

    Ok(())
}
