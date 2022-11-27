use std::fs::File;
use std::io::{self, BufRead, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpStream};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

fn main() {
    let list_open_ips = Arc::new(Mutex::new(Vec::new()));
    let mut qtd_ips_verified: u32 = 0;
    let start_time = std::time::Instant::now();
    let max_threads = 500;

    let Ok(lines) = read_lines("C:\\Users\\Juninho\\Desktop\\rust\\find_minecraft_servers\\br.csv") else { return };
    for line_r in lines {
        if let Ok(line) = line_r {
            let mut chuncks_list: Vec<Vec<String>> = vec![];

            let size = process_line(line, &mut chuncks_list);

            let mut handles = vec![];

            for chunck in chuncks_list {
                let list_open_ips = Arc::clone(&list_open_ips);
                handles.push(thread::spawn(move || {
                    let open_ips = run_job(&chunck);

                    let mut m_aux = list_open_ips.lock().unwrap();
                    m_aux.extend(open_ips);
                }));

                if handles.len() == max_threads {
                    for handle in handles {
                        handle.join().unwrap();
                    }
                    handles = vec![];
                }
            }

            for handle in handles {
                handle.join().unwrap();
            }

            qtd_ips_verified += size;

            //clear the window console
            print!("{}[2J", 27 as char);
            println!("{}[1;1H", 27 as char);

            println!("{} ips verified", qtd_ips_verified);
            println!("{} ips open", list_open_ips.lock().unwrap().len());

            //ips per second rounded
            let elapsed = start_time.elapsed();
            let ips_per_second = qtd_ips_verified as f64 / elapsed.as_secs_f64();
            println!("{} ips per second", ips_per_second.round());
        }
    }

    // write to file
    let mut file =
        File::create("C:\\Users\\Juninho\\Desktop\\rust\\find_minecraft_servers\\open_servers.txt")
            .unwrap();

    let m_aux = list_open_ips.lock().unwrap().clone();

    println!("***********************************************************************");

    writeln!(file, "{}", m_aux.join(";\n")).unwrap();

    println!("***********************************************************************");

    println!("done");
}

fn process_line(line: String, ips_pac: &mut Vec<Vec<String>>) -> u32 {
    //139.82.0.0,139.82.255.255,65536,12/02/90,
    let ips: Vec<&str> = line.split(",").collect();
    let ip_start: Ipv4Addr = ips[0].parse().unwrap();
    let size = ips[2].parse::<u32>().unwrap();

    let mut ip = ip_start.clone();

    let mut qtd_ips_generated = 0;

    while qtd_ips_generated < size {
        let mut chunck = vec![];
        for _i in 0..10 {
            if qtd_ips_generated < size {
                chunck.push(ip.to_string());
                qtd_ips_generated += 1;
                ip = get_next_ip(ip);
            }
        }
        ips_pac.push(chunck);
    }

    size
}

fn run_job(ips: &Vec<String>) -> Vec<String> {
    let mut open_ips: Vec<String> = vec![];
    for ip in ips {
        let ip_parsed: SocketAddr = format!("{}:25565", ip).parse().unwrap();
        let stream = TcpStream::connect_timeout(&ip_parsed, Duration::from_millis(100));

        match stream {
            Ok(_) => {
                println!("{} is open", ip);
                open_ips.push(ip.to_string());
            }
            Err(_) => {}
        }
    }

    open_ips
}

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn get_next_ip(ip: Ipv4Addr) -> Ipv4Addr {
    let mut ip_bytes = ip.octets();
    let mut i = 3;

    loop {
        if ip_bytes[i] < 255 {
            ip_bytes[i] += 1;
            break;
        } else {
            ip_bytes[i] = 0;
            i -= 1;
        }
    }

    Ipv4Addr::from(ip_bytes)
}
