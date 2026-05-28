use sysinfo::{Pid, ProcessesToUpdate, System};

pub fn process_alive(sys: &mut System, pid: u32) -> bool {
    sys.refresh_processes(ProcessesToUpdate::Some(&[Pid::from_u32(pid)]), false);
    sys.process(Pid::from_u32(pid)).is_some()
}
