    
use std::process;

use netlink_sys::{protocols::NETLINK_KOBJECT_UEVENT, Socket, SocketAddr};

use kobject_uevent::UEvent;
use kobject_uevent::ActionType;

use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn is_nvidia_gpu(u: UEvent) -> bool {

    if u.env.contains_key("PCI_ID") == false {
        return false;
    }
    assert!(u.env.contains_key("PCI_CLASS"));

    let parts: Vec<&str> = u.env["PCI_ID"].split(':').collect();

    if parts[0] != "10DE" {
        return false;
    }

    if u.env["PCI_CLASS"] != "30200" && u.env["PCI_CLASS"] != "30000" {
        return false
    }

    return true
}

// Function to get the current time in seconds since the UNIX epoch
fn get_current_time() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}

fn hotplug_device(timeout: u64) -> bool {
      // Initialize the timestamp of the last GPU plug event
      let mut last_gpu_plug_timestamp = get_current_time();

      // Wait time in seconds before considering the hot-plugging process done
      let wait_time = timeout;
  
      // Schedule the execution of the check function after wait_time seconds
      loop {
          thread::sleep(Duration::from_secs(wait_time));
          
          if check_hotplug_activity(&mut last_gpu_plug_timestamp, wait_time) {
              return true
          }
      }
}

fn check_hotplug_activity(last_timestamp: &mut u64, wait_time: u64) -> bool {
    let current_time = get_current_time();
    let time_diff = current_time - *last_timestamp;

    // Update the last timestamp to current time for the next check
    *last_timestamp = current_time;

    // If the difference is greater than or equal to wait_time, execute the target commands
    if time_diff >= wait_time {
        return true;
    }

    false
}
fn main() {
    let mut socket = Socket::new(NETLINK_KOBJECT_UEVENT).unwrap();
    let sa = SocketAddr::new(process::id(), 1);

    socket.bind(&sa).unwrap();

    loop {
        let n = socket.recv_from_full().unwrap();
        let s = std::str::from_utf8(&n.0);
        let u = UEvent::from_netlink_packet(&n.0).unwrap();
        println!(">> {}", s.unwrap());
        println!("{:#?}", u);

        // hot-plug event 
        if u.action == ActionType::Add {
            if is_nvidia_gpu(u) {
                if hotplug_device(5) {
                    println!("hotplug activity finished, proceeding.");
                    return
                }
            }
        }
        

    }
}

