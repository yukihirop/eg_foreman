use std::process::{Child};
use std::sync::{Mutex};

pub struct Process {
  pub name: String,
  pub child: Mutex<Child>,
}
