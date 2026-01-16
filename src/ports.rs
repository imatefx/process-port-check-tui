use netstat2::{get_sockets_info, AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo, TcpState};
use sysinfo::{Pid, ProcessRefreshKind, System};
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct PortInfo {
    pub port: u16,
    pub pid: u32,
    pub process_name: String,
    pub exe_path: Option<PathBuf>,
    pub cwd: Option<PathBuf>,
    pub cmd_args: Vec<String>,
}

pub fn get_listening_ports() -> Result<Vec<PortInfo>, Box<dyn std::error::Error>> {
    let af_flags = AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6;
    let proto_flags = ProtocolFlags::TCP;
    let sockets = get_sockets_info(af_flags, proto_flags)?;

    let mut sys = System::new();
    sys.refresh_processes_specifics(
        sysinfo::ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::everything(),
    );

    let mut ports = Vec::new();

    for si in sockets {
        if let ProtocolSocketInfo::Tcp(tcp) = si.protocol_socket_info {
            if tcp.state == TcpState::Listen {
                // Take only the first associated PID for each socket
                if let Some(pid) = si.associated_pids.first() {
                    let pid_usize = *pid as usize;
                    let (name, exe, cwd, cmd) = if let Some(proc) = sys.process(Pid::from(pid_usize)) {
                        (
                            proc.name().to_string_lossy().to_string(),
                            proc.exe().map(|p| p.to_path_buf()),
                            proc.cwd().map(|p| p.to_path_buf()),
                            proc.cmd().iter().map(|s| s.to_string_lossy().to_string()).collect(),
                        )
                    } else {
                        (String::from("unknown"), None, None, vec![])
                    };

                    ports.push(PortInfo {
                        port: tcp.local_port,
                        pid: *pid,
                        process_name: name,
                        exe_path: exe,
                        cwd,
                        cmd_args: cmd,
                    });
                }
            }
        }
    }

    ports.sort_by_key(|p| p.port);
    ports.dedup_by_key(|p| (p.port, p.pid));
    Ok(ports)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_listening_ports() {
        let ports = get_listening_ports().expect("Should get ports");
        println!("Found {} listening ports:", ports.len());
        for p in &ports {
            println!(
                "  Port {} - PID {} - {} - cwd: {:?}",
                p.port, p.pid, p.process_name, p.cwd
            );
        }
        // Should at least run without error
    }
}
