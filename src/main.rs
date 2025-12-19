use std::env;
use std::net::{IpAddr, SocketAddr, TcpStream, ToSocketAddrs};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn try_pair(ip: &str, port: u16, pairing_code: &str) -> bool {
    let result = Command::new("adb")
        .arg("pair")
        .arg(format!("{}:{}", ip, port))
        .arg(pairing_code)
        .output();

    match result {
        Ok(output) if output.status.success() => {
            println!("[OK] Paired successfully on port {}", port);
            true
        }
        Ok(_) => false,
        Err(_) => false,
    }
}

fn try_connect(ip: &str, port: u16) -> bool {
    let result = Command::new("adb")
        .arg("connect")
        .arg(format!("{}:{}", ip, port))
        .output();

    match result {
        Ok(output) if output.status.success() => {
            println!("[OK] Connected on port {}", port);
            true
        }
        Ok(_) => false,
        Err(_) => false,
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: zadb <ip|hostname> <pairing-code>");
        std::process::exit(1);
    }

    let ip = &args[1];
    let pairing_code = &args[2];

    println!("Scanning ports on {}...", ip);

    let open_ports = scan_ports_fast(ip);

    if open_ports.is_empty() {
        eprintln!("[FAIL] No open ports found in range 30000-49999");
        std::process::exit(1);
    }

    if open_ports.len() == 1 {
        eprintln!("[FAIL] Only found one port, need both pairing and service ports, rerun this");
        std::process::exit(1);
    }

    println!("[OK] Found open ports: {:?}", open_ports);

    // try pairing on all OPENED ports in parallel
    println!("Trying to pair on all opened ports...");
    let pairing_port = Arc::new(Mutex::new(None));
    let mut handles = vec![];

    for &port in &open_ports {
        let ip = ip.to_string();
        let pairing_code = pairing_code.to_string();
        let pairing_port_clone = Arc::clone(&pairing_port);

        let handle = thread::spawn(move || {
            if try_pair(&ip, port, &pairing_code) {
                let mut p = pairing_port_clone.lock().unwrap();
                if p.is_none() {
                    *p = Some(port);
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let pairing_port = match *pairing_port.lock().unwrap() {
        Some(p) => p,
        None => {
            eprintln!("[FAIL] None of the ports accepted pairing");
            std::process::exit(1);
        }
    };

    // wait for pairing to settle
    thread::sleep(Duration::from_millis(500));

    // try connecting to all non-pairing opened ports in parallel
    println!("Trying to connect on remaining ports...");
    let service_port = Arc::new(Mutex::new(None));
    let mut handles = vec![];

    for &port in &open_ports {
        if port == pairing_port {
            continue;
        }

        let ip = ip.to_string();
        let service_port_clone = Arc::clone(&service_port);

        let handle = thread::spawn(move || {
            if try_connect(&ip, port) {
                let mut p = service_port_clone.lock().unwrap();
                if p.is_none() {
                    *p = Some(port);
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let service_port = match *service_port.lock().unwrap() {
        Some(p) => p,
        None => {
            eprintln!("[FAIL] None of the ports accepted connection");
            std::process::exit(1);
        }
    };

    println!("[OK] Done! Device connected on {}:{}", ip, service_port);
}

// nmap turned out to be too slow for this task
fn scan_ports_fast(ip: &str) -> Vec<u16> {
    let ip_addr: IpAddr = match ip.parse() {
        Ok(addr) => addr,
        Err(_) => match (ip, 0).to_socket_addrs() {
            Ok(mut addrs) => {
                if let Some(socket_addr) = addrs.next() {
                    socket_addr.ip()
                } else {
                    eprintln!("[FAIL] Could not resolve hostname: {}", ip);
                    std::process::exit(1);
                }
            }
            Err(e) => {
                eprintln!("[FAIL] Invalid IP/hostname: {}", e);
                std::process::exit(1);
            }
        },
    };

    let open_ports = Arc::new(Mutex::new(Vec::new()));
    let mut handles = vec![];

    // 200 threads, 100 ports each
    let chunk_size = 100;
    let num_threads = 200;

    for i in 0..num_threads {
        let chunk_start = 30000 + (i * chunk_size);
        let chunk_end = std::cmp::min(chunk_start + chunk_size, 50000);

        let ports_clone = Arc::clone(&open_ports);

        let handle = thread::spawn(move || {
            for port in chunk_start..chunk_end {
                let addr = SocketAddr::new(ip_addr, port);

                if TcpStream::connect_timeout(&addr, Duration::from_millis(100)).is_ok() {
                    ports_clone.lock().unwrap().push(port);
                }
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let mut ports = open_ports.lock().unwrap().clone();
    ports.sort();
    ports
}
