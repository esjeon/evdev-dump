
use evdev_rs as evdev;
use evdev::enums::{EventType, EventCode, EV_KEY};
use nix::sys::termios::{tcgetattr, tcsetattr, LocalFlags, SetArg};
use std::os::unix::io::AsRawFd;
use std::time::Duration;

#[derive(Default)]
struct Config {
    devpath: String,
}

fn usage() {
    let progname = std::env::current_exe().unwrap();
    let progname = progname
        .file_name().unwrap()
        .to_str().unwrap();
    eprintln!("Usage: {} <event device path>", progname);
}

fn parse_args() -> Option<Config> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        return None;
    }

    let mut config = Config::default();
    config.devpath = args[1].clone();

    Some(config)
}

fn set_terminal_echo(enable: bool) -> Result<(), nix::Error> {
    let stdin = std::io::stdin();

    let mut termios = tcgetattr(stdin.as_raw_fd())?;
    let enabled = termios.local_flags.contains(LocalFlags::ECHO);
    match (enabled, enable) {
        (false, true) => termios.local_flags.insert(LocalFlags::ECHO),
        (true, false) => termios.local_flags.remove(LocalFlags::ECHO),
        _ => return Ok(()),
    }

    tcsetattr(stdin.as_raw_fd(), SetArg::TCSAFLUSH, &termios)?;

    Ok(())
}

fn main() {
    let config = match parse_args() {
        Some(v) => v,
        None => return usage(),
    };

    let file = std::fs::File::open(config.devpath)
        .expect("can't open the device");
    let dev = evdev::Device::new_from_fd(file)
        .expect("can't open the device as an evdev device");

    let _ = set_terminal_echo(false);

    let mut lastq_stamp = Duration::new(0, 0);
    eprintln!("Press Q twice to quit...");
    loop {
        let ev = match dev.next_event(evdev::ReadFlag::NORMAL) {
            Ok((evdev::ReadStatus::Success, ev)) => ev,
            Ok((evdev::ReadStatus::Sync, _)) => continue,
            Err(errno) => {
                if errno as u32 == nix::errno::Errno::EAGAIN as u32 {
                    continue
                }
                eprintln!("an error has occured: {}", errno);
                break
            }
        };

        println!("{:>012}.{:<9}: {:12} {:24} {}",
            ev.time.tv_sec, ev.time.tv_usec,
            ev.event_type.to_string(), ev.event_code.to_string(), ev.value);

        if ev.event_type == EventType::EV_KEY && ev.event_code == EventCode::EV_KEY(EV_KEY::KEY_Q) {
            if ev.value == 1 {
                let stamp = Duration::new(ev.time.tv_sec as u64, ev.time.tv_usec as u32);
                if (stamp - lastq_stamp).as_millis() < 200 {
                    break;
                }
                lastq_stamp = stamp;
            }
        }
    }
    eprintln!("Exiting...");

    let _ = set_terminal_echo(true);
}
