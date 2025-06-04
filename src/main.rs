use komorebi_client::Notification;
use komorebi_client::Rect;
use komorebi_client::SocketMessage;
use komorebi_client::State;
use komorebi_client::SubscribeOptions;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::BufRead;
use std::io::BufReader;

#[derive(Clone, Serialize, Deserialize, Debug)]
struct Rule {
    count: usize,
    padding: Rect,
}
#[derive(Clone, Serialize, Deserialize, Debug)]
struct Workspace {
    rules: Option<Vec<Rule>>,
    default: Option<Rect>,
    monocle: Option<Rect>,
}
#[derive(Clone, Serialize, Deserialize, Debug)]
struct Monitor {
    workspaces: Option<Vec<Workspace>>,
    rules: Option<Vec<Rule>>,
    default: Option<Rect>,
    monocle: Option<Rect>,
}
#[derive(Clone, Serialize, Deserialize, Debug)]
struct Config {
    monitors: Vec<Monitor>,
    default: Rect,
    monocle: Option<Rect>,
}
#[derive(Clone, Debug)]
struct AppState {
    active_workspace: Vec<usize>,
    monitors: Vec<MonitorState>,
}
impl AppState {
    fn new() -> Self {
        Self {
            active_workspace: Vec::new(),
            monitors: Vec::new(),
        }
    }
}
#[derive(Clone, Debug)]
struct MonitorState {
    workspaces: Vec<WorkspaceState>,
}
impl MonitorState {
    fn new() -> Self {
        Self {
            workspaces: Vec::new(),
        }
    }
}
#[derive(Clone, Debug)]
struct WorkspaceState {
    window_count: usize,
    rules: Option<Vec<Rule>>,
    default: Rect,
    monocle: Option<Rect>,
}

const NAME: &str = "komofake.sock";

pub fn main() -> anyhow::Result<()> {
    // let socket = komorebi_client::subscribe(NAME)?;
    let socket = komorebi_client::subscribe_with_options(
        NAME,
        SubscribeOptions {
            filter_state_changes: true,
        },
    )?;
    let json_data = fs::read_to_string("./config.json").expect("Failed to read config.json");

    let state_data = komorebi_client::send_query(&SocketMessage::State).unwrap();
    let state: State = serde_json::from_str(&state_data).expect("Failed to get state");

    let config: Config = serde_json::from_str(&json_data).expect("Failed to deserialize JSON");
    let mut app_state = initialize_app_state(&config, &state);

    handle_state(state, &mut app_state);

    for incoming in socket.incoming() {
        match incoming {
            Ok(data) => {
                let reader = BufReader::new(data.try_clone()?);
                let mut state: Option<State> = None;
                for line in reader.lines().flatten() {
                    let notification: Notification = match serde_json::from_str(&line) {
                        Ok(notification) => notification,
                        Err(error) => {
                            println!("discarding malformed komorebi notification: {error}");
                            continue;
                        }
                    };
                    state = Some(notification.state);
                }
                handle_state(state.unwrap(), &mut app_state);
            }
            Err(error) => {
                println!("{error}");
            }
        }
    }
    Ok(())
}

fn handle_state(state: State, app_state: &mut AppState) {
    for (monitor_index, monitor) in state.monitors.elements().iter().enumerate() {
        for (workspace_index, workspace) in monitor.workspaces.elements().iter().enumerate() {
            let workspace_state =
                &mut app_state.monitors[monitor_index].workspaces[workspace_index];
            let window_count = workspace.containers.elements().len();
            if window_count != workspace_state.window_count {
                workspace_state.window_count = window_count;
            }
        }
        let focused_workspace_index = monitor.workspaces.focused_idx();
        let mut offset = None;
        if focused_workspace_index != app_state.active_workspace[monitor_index] {
            app_state.active_workspace[monitor_index] = focused_workspace_index;
        }
        let workspace_state =
            &app_state.monitors[monitor_index].workspaces[focused_workspace_index];
        if let Some(rules) = &workspace_state.rules {
            for rule in rules {
                if workspace_state.window_count <= rule.count {
                    offset = Some(rule.padding);
                    break;
                }
            }
        }
        if workspace_state.monocle != None
            && *monitor.focused_workspace().unwrap().monocle_container() != None
        {
            offset = workspace_state.monocle;
        }

        if offset == None {
            offset = Some(workspace_state.default);
        }
        match monitor.work_area_offset {
            Some(work_area_offset) => {
                if work_area_offset != offset.unwrap() {
                    update_offset(monitor_index, offset.expect("Invalid offset"));
                }
            }
            None => update_offset(monitor_index, offset.expect("Invalid offset")),
        }
    }
}

fn update_offset(monitor_index: usize, offset: Rect) {
    komorebi_client::send_message(&SocketMessage::MonitorWorkAreaOffset(monitor_index, offset))
        .unwrap();
}

fn initialize_app_state(config: &Config, state: &State) -> AppState {
    let mut app_state = AppState::new();
    let global_default = config.default;
    let global_monocle = config.monocle;
    for (monitor_index, monitor) in state.monitors.elements().iter().enumerate() {
        let mut monitor_state = MonitorState::new();
        let focused_workspace_index = monitor.workspaces.focused_idx();
        app_state.active_workspace.push(focused_workspace_index);
        let monitor_default = config.monitors[monitor_index].default;
        let monitor_rules = &config.monitors[monitor_index].rules;
        let monitor_monocle = config.monitors[monitor_index].monocle;
        for (workspace_index, workspace) in monitor.workspaces.elements().iter().enumerate() {
            let window_count = workspace.containers().len();
            let mut workspace_rules = None;
            let mut workspace_default: Option<Rect> = None;
            let mut workspace_monocle = None;

            let monitor = &config.monitors[monitor_index];
            if let Some(workspaces) = &monitor.workspaces {
                if let Some(workspace) = workspaces.get(workspace_index) {
                    workspace_default = workspace.default;
                    workspace_rules = workspace.rules.clone();
                    workspace_monocle = workspace.monocle;
                }
            }
            let default = workspace_default
                .or(monitor_default)
                .unwrap_or(global_default);
            let rules = workspace_rules.or(monitor_rules.clone());
            let monocle = workspace_monocle.or(monitor_monocle).or(global_monocle);
            let workspace_state = WorkspaceState {
                window_count,
                rules,
                default,
                monocle,
            };
            monitor_state.workspaces.push(workspace_state);
        }
        app_state.monitors.push(monitor_state);
    }
    println!("{:#?}", app_state);
    return app_state;
}
