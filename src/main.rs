use komorebi_client::Notification;
use komorebi_client::NotificationEvent;
use komorebi_client::Rect;
use komorebi_client::SocketMessage;
use komorebi_client::WindowManagerEvent;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::BufRead;
use std::io::BufReader;

#[derive(Serialize, Deserialize, Debug)]
struct Rule {
    count: usize,
    padding: Rect,
}
#[derive(Serialize, Deserialize, Debug)]
struct Workspace {
    idx: usize,
    rules: Vec<Rule>,
    default: Option<Rect>,
}
#[derive(Serialize, Deserialize, Debug)]
struct Monitor {
    workspaces: Vec<Workspace>,
    rules: Vec<Rule>,
    default: Option<Rect>,
}
#[derive(Serialize, Deserialize, Debug)]
struct Config {
    monitors: Vec<Monitor>,
    default: Rect,
}

const NAME: &str = "komofake.sock";

pub fn main() -> anyhow::Result<()> {
    let socket = komorebi_client::subscribe(NAME)?;
    let json_data = fs::read_to_string("./config.json").expect("Failed to read config.json");

    let config: Config = serde_json::from_str(&json_data).expect("Failed to deserialize JSON");
    println!("{:#?}", config);
    for incoming in socket.incoming() {
        let mut padding = None;
        match incoming {
            Ok(data) => {
                let reader = BufReader::new(data.try_clone()?);

                for line in reader.lines().flatten() {
                    let notification: Notification = match serde_json::from_str(&line) {
                        Ok(notification) => notification,
                        Err(error) => {
                            println!("discarding malformed komorebi notification: {error}");
                            continue;
                        }
                    };
                    match notification.event {
                        NotificationEvent::WindowManager(WindowManagerEvent::FocusChange(
                            event,
                            window,
                        )) => {
                            // println!("Focus changed! :)");
                        }
                        _ => {
                            continue;
                        }
                    }
                    let focused_monitor_idx = notification.state.monitors.focused_idx();
                    // println!("{:#?}", notification.event);
                    if let Some(focused_monitor) = notification.state.monitors.focused() {
                        let focused_workspace_idx = focused_monitor.focused_workspace_idx();
                        if let Some(focused_workspace) = focused_monitor.focused_workspace() {
                            let window_count = focused_workspace.containers.elements().len();
                            for workspace in &config.monitors[focused_monitor_idx].workspaces {
                                if workspace.idx == focused_workspace_idx {
                                    for rule in &workspace.rules {
                                        if window_count <= rule.count {
                                            padding = Some(rule.padding);
                                            break;
                                        }
                                    }
                                    if padding == None {
                                        if workspace.default != None {
                                            padding = workspace.default;
                                        }
                                    }
                                }
                            }
                            if padding == None {
                                if config.monitors[focused_monitor_idx].rules.len() > 0 {
                                    for rule in &config.monitors[focused_monitor_idx].rules {
                                        if window_count <= rule.count {
                                            padding = Some(rule.padding);
                                            break;
                                        }
                                    }
                                    if padding == None {
                                        padding = config.monitors[focused_monitor_idx].default;
                                    }
                                } else if config.monitors[focused_monitor_idx].default != None {
                                    padding = config.monitors[focused_monitor_idx].default;
                                } else {
                                    padding = Some(config.default);
                                }
                            }
                            // println!(
                            //     "Active monitor idx: {} \n Active workspace idx: {} \n {:#?}",
                            //     focused_monitor_idx, focused_workspace_idx, padding
                            // );

                            komorebi_client::send_message(&SocketMessage::MonitorWorkAreaOffset(
                                focused_monitor_idx,
                                padding.unwrap(),
                            ))
                            .unwrap();
                        }
                    }
                }
            }
            Err(error) => {
                println!("{error}");
            }
        }
    }
    Ok(())
}
