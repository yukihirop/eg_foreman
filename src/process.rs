use std::process::{Child};
use std::sync::{Arc, Mutex};

pub struct Process {
  pub name: String,
  pub child: Arc<Mutex<Child>>,
}
