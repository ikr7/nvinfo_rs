use std::process::exit;
use std::process::Command;

struct GPU {
    index: u8,
    gpu_uuid: String,
    name: String,
    memory_used: u32,
    memory_total: u32,
    utilization_gpu: u32,
    persistence_mode: bool,
}

impl GPU {
    fn from_line(line: &str) -> GPU {
        let elems: Vec<&str> = line.split(',').collect();
        let index: u8 = elems[0].trim().parse().unwrap();
        let gpu_uuid = String::from(elems[1].trim());
        let name = String::from(elems[2].trim());
        let memory_used: u32 = elems[3].trim().parse().unwrap();
        let memory_total: u32 = elems[4].trim().parse().unwrap();
        let utilization_gpu: u32 = elems[5].trim().parse().unwrap();
        let persistence_mode = elems[6].trim() == "Enabled";
        return GPU {
            index,
            gpu_uuid,
            name,
            memory_used,
            memory_total,
            utilization_gpu,
            persistence_mode,
        };
    }
}

struct Process {
    gpu_uuid: String,
    pid: u32,
    used_gpu_memory: u32,
    user: String,
    command: String,
}

impl Process {
    fn from_line(line: &str) -> Process {
        let elems: Vec<&str> = line.split(',').collect();
        let gpu_uuid = String::from(elems[0].trim());
        let pid: u32 = elems[1].trim().parse().unwrap();
        let used_gpu_memory: u32 = elems[2].trim().parse().unwrap();

        return Process {
            gpu_uuid,
            pid,
            used_gpu_memory,
            user: Process::get_user(pid).trim().to_string(),
            command: Process::get_command(pid).trim().to_string(),
        };
    }
    fn get_user(pid: u32) -> String {
        let output = Command::new("ps")
            .arg("ho")
            .arg("user")
            .arg(format!("{}", pid))
            .output()
            .expect("Failed to find process");
        let stdout = String::from_utf8(output.stdout).unwrap();
        return stdout;
    }
    fn get_command(pid: u32) -> String {
        let output = Command::new("ps")
            .arg("ho")
            .arg("command")
            .arg(format!("{}", pid))
            .output()
            .expect("Failed to find process");
        let stdout = String::from_utf8(output.stdout).unwrap();
        return stdout;
    }
}

fn retrieve_gpus() -> Vec<GPU> {
    let output = Command::new("/usr/bin/env")
        .arg("nvidia-smi")
        .arg("--format=csv,noheader,nounits")
        .arg("--query-gpu=index,gpu_uuid,name,memory.used,memory.total,utilization.gpu,persistence_mode")
        .output()
        .expect("Failed to call nvidia-smi command");
    let stdout = String::from_utf8(output.stdout).expect("Faield to encode command output");
    return stdout.lines().map(GPU::from_line).collect();
}

fn retrieve_processes() -> Vec<Process> {
    let output = Command::new("/usr/bin/env")
        .arg("nvidia-smi")
        .arg("--format=csv,noheader,nounits")
        .arg("--query-compute-apps=gpu_uuid,pid,used_memory")
        .output()
        .expect("Failed to call nvidia-smi command");
    let stdout = String::from_utf8(output.stdout).expect("Faield to encode command output");
    return stdout.lines().map(Process::from_line).collect();
}

fn main() {
    let gpus = retrieve_gpus();
    let processes = retrieve_processes();

    if gpus.iter().any(|gpu| !gpu.persistence_mode) {
        println!("Consider enabling persistence mode on your GPU(s) for faster response.");
        println!("For more information: https://docs.nvidia.com/deploy/driver-persistence/");
    }

    println!("+----------------------------+------+-------------------+---------+");
    println!("| GPU                        | %GPU | VRAM              | PROCESS |");
    println!("|----------------------------+------+-------------------+---------|");

    for gpu in &gpus {
        let used = processes
            .iter()
            .any(|process| process.gpu_uuid == gpu.gpu_uuid);

        println!(
            "| {:3} {:22} | {:3}% | {:5} / {:5} MiB | {} |",
            gpu.index,
            format!("({})", gpu.name),
            gpu.utilization_gpu,
            gpu.memory_used,
            gpu.memory_total,
            if used { "RUNNING" } else { "-------" }
        );
    }

    println!("|=================================================================|");

    if processes.len() == 0 {
        println!("| No running processes found                                      |");
        println!("+-----------------------------------------------------------------+");
        exit(0);
    }

    println!("| GPU | USER       | PID     | VRAM      | COMMAND                |");
    println!("|-----+------------+---------+-----------+------------------------|");

    for process in processes {
        let gpu_uuid = &gpus
            .iter()
            .find(|&gpu| gpu.gpu_uuid == process.gpu_uuid)
            .unwrap()
            .index;
        println!(
            "| {:3} | {:^10} | {:7} | {:5} MiB | {:>22} |",
            gpu_uuid,
            process.user,
            process.pid,
            process.used_gpu_memory,
            &process.command[..22]
        );
    }

    println!("+-----+------------+---------+-----------+------------------------+");
}
