use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::collections::HashMap;

mod output;
mod stream_read;
mod process;
mod log;
mod signal;

struct Script {
    cmd: String,
    concurrency: usize
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut proc_handle_threads = vec![];
    let mut procs: Vec<Arc<Mutex<process::Process>>> = vec![];
    
    let mut scripts = HashMap::<&str, Script>::new();
    scripts.insert("loop", Script {
        cmd: String::from("./bin/loop.sh"),
        concurrency: 2
    });
    scripts.insert("exit_1", Script {
        cmd: String::from("./bin/exit_1.sh"),
        concurrency: 1
    });
    scripts.insert("exit_0", Script {
        cmd: String::from("./bin/exit_0.sh"),
        concurrency: 1
    });

    for (key, script) in scripts {
        let con = script.concurrency;

        for n in 0..con {
            let tmp_proc = process::Process {
                name: String::from(format!("{}.{}", key, n+1)),
                child: Mutex::new(
                            Command::new(&script.cmd)
                                .stdout(Stdio::piped())
                                .stderr(Stdio::piped())
                                .spawn()?
                            ),
                        
            };

            let proc = Arc::new(Mutex::new(tmp_proc));
            let proc2 = Arc::clone(&proc);

            let handle_output = thread::Builder::new()
                .name(String::from("handling output"))
                .spawn(move || {
                    output::handle_output(&proc)
                })?;
        
            proc_handle_threads.push(handle_output);
            procs.push(proc2);
        }
    }

    let procs2 = procs.clone();
    let procs3 = procs.clone();
    let procs_amut = Arc::new(Mutex::new(procs));

    let proc_check_child_terminated_threads = procs2.into_iter().enumerate().map(|(idx, proc)| {
        let procs_amut = Arc::clone(&procs_amut);
        let proc2 = proc.clone();
        let proc_arc = Arc::clone(&proc2);

        let check_child_terminated = thread::Builder::new()
            .name(String::from(format!("checking child terminated: {}", idx)))
            .spawn(move || {
                let child = &proc_arc.lock().unwrap().child;
                if let Ok(status) = child.lock().unwrap().wait() {
                    log::output(&proc_arc.lock().unwrap().name, &format!("exit {}", status));
                    procs_amut.lock().unwrap().remove(idx);
                }
                return ()
            }).expect("check");
        
        check_child_terminated
    });

    let proc_handle_signal_threads = procs3.clone().into_iter().enumerate().map(|(idx, proc)| {
        let proc2 = proc.clone();
        let proc_arc = Arc::clone(&proc2);

        let handlel_signal = thread::Builder::new()
            .name(String::from(format!("handling signal: {}", idx)))
            .spawn(move || {
                let child = &proc_arc.lock().unwrap().child;
                signal::handle_signal(child).unwrap();
            }).expect("signal");

        handlel_signal
    });

    for check in proc_check_child_terminated_threads.into_iter() {
        check.join().expect("failed join");
    }

    for handle in proc_handle_signal_threads.into_iter() {
        handle.join().expect("failed join");
    }

    for handle in proc_handle_threads {
        handle.join().expect("failed join");
    }

    Ok(())
}
