use crate::log;
use std::sync::{Mutex};
use signal_hook::{iterator::Signals, SIGINT, SIGALRM, SIGHUP, SIGTERM};
use nix::sys::signal::{self, Signal};
use nix::unistd::Pid;
use std::process::{exit, Child};

pub fn handle_signal(child: &Mutex<Child>) -> Result<(), Box<dyn std::error::Error>> {
  let signals = Signals::new(&[SIGALRM, SIGHUP, SIGINT, SIGTERM])?;

  for sig in signals.forever() {
    match sig {
      SIGINT => {
        log::output("system", "ctrl-c detected");
        log::output("system", "sending SIGTERM for children");
        log::output("system", &format!("child pid: {}", child.lock().unwrap().id()));

        if let Err(e) = signal::kill(Pid::from_raw(child.lock().unwrap().id() as i32), Signal::SIGTERM) {
          log::error("system", &e);
          log::output("system", "exit 1");
          exit(1);
        }
      },
      _ => ()
    }
  }

  Ok(())
}
