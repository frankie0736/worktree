use std::collections::{HashMap, HashSet};

use serde::Serialize;

use crate::constants::TASKS_DIR;
use crate::error::Result;
use crate::models::{Task, TaskStatus, TaskStore};

#[derive(Serialize)]
struct TaskJson {
    name: String,
    status: TaskStatus,
    depends: Vec<String>,
}

#[derive(Serialize)]
struct ListOutput {
    tasks: Vec<TaskJson>,
}

pub fn execute(tree: bool, json: bool) -> Result<()> {
    let store = TaskStore::load()?;
    let tasks = store.list();

    if json {
        print_json(&tasks, &store);
    } else if tree {
        print_tree(&tasks, &store);
    } else {
        print_grouped(&tasks, &store);
    }

    Ok(())
}

fn print_json(tasks: &[&Task], store: &TaskStore) {
    let output = ListOutput {
        tasks: tasks
            .iter()
            .map(|t| TaskJson {
                name: t.name().to_string(),
                status: store.get_status(t.name()),
                depends: t.depends().to_vec(),
            })
            .collect(),
    };
    println!("{}", serde_json::to_string(&output).unwrap_or_default());
}

fn print_grouped(tasks: &[&Task], store: &TaskStore) {
    if tasks.is_empty() {
        println!("No tasks found in {}/", TASKS_DIR);
        return;
    }

    // Group tasks by status
    let mut archived: Vec<&Task> = Vec::new();
    let mut merged: Vec<&Task> = Vec::new();
    let mut ready: Vec<&Task> = Vec::new();
    let mut blocked: Vec<(&Task, Vec<&str>)> = Vec::new();
    let mut running: Vec<&Task> = Vec::new();
    let mut done: Vec<&Task> = Vec::new();

    for task in tasks {
        let status = store.get_status(task.name());
        match status {
            TaskStatus::Archived => archived.push(task),
            TaskStatus::Merged => merged.push(task),
            TaskStatus::Running => running.push(task),
            TaskStatus::Done => done.push(task),
            TaskStatus::Pending => {
                // Check if all dependencies are merged or archived
                let unmerged_deps: Vec<&str> = task
                    .depends()
                    .iter()
                    .filter(|dep| {
                        let dep_status = store.get_status(dep);
                        dep_status != TaskStatus::Merged && dep_status != TaskStatus::Archived
                    })
                    .map(|s| s.as_str())
                    .collect();

                if unmerged_deps.is_empty() {
                    ready.push(task);
                } else {
                    blocked.push((task, unmerged_deps));
                }
            }
        }
    }

    // Print Archived
    if !archived.is_empty() {
        println!("Archived ({}):", archived.len());
        for task in &archived {
            println!("  {} {}", TaskStatus::Archived.icon(), task.name());
        }
        println!();
    }

    // Print Merged
    if !merged.is_empty() {
        println!("Merged ({}):", merged.len());
        for task in &merged {
            println!("  {} {}", TaskStatus::Merged.icon(), task.name());
        }
        println!();
    }

    // Print Running
    if !running.is_empty() {
        println!("Running ({}):", running.len());
        for task in &running {
            print_task_with_deps(task, store);
        }
        println!();
    }

    // Print Done
    if !done.is_empty() {
        println!("Done ({}):", done.len());
        for task in &done {
            print_task_with_deps(task, store);
        }
        println!();
    }

    // Print Ready
    if !ready.is_empty() {
        println!("Ready ({}):", ready.len());
        for task in &ready {
            print_task_with_deps(task, store);
        }
        println!();
    }

    // Print Blocked
    if !blocked.is_empty() {
        println!("Blocked ({}):", blocked.len());
        for (task, waiting_for) in &blocked {
            print!("  {} {}", TaskStatus::Pending.icon(), task.name());
            if !task.depends().is_empty() {
                print!(" ←");
                for (i, dep) in task.depends().iter().enumerate() {
                    if i > 0 {
                        print!(",");
                    }
                    let icon = if waiting_for.contains(&dep.as_str()) {
                        TaskStatus::Pending.icon()
                    } else {
                        TaskStatus::Merged.icon()
                    };
                    print!(" {}{}", dep, icon);
                }
            }
            println!();
        }
    }
}

fn print_task_with_deps(task: &Task, store: &TaskStore) {
    let status = store.get_status(task.name());
    print!("  {} {}", status.icon(), task.name());
    if !task.depends().is_empty() {
        print!(" ←");
        for (i, dep) in task.depends().iter().enumerate() {
            if i > 0 {
                print!(",");
            }
            let dep_icon = store.get_status(dep).icon();
            print!(" {}{}", dep, dep_icon);
        }
    }
    println!();
}

fn print_tree(tasks: &[&Task], store: &TaskStore) {
    if tasks.is_empty() {
        println!("No tasks found in {}/", TASKS_DIR);
        return;
    }

    let children: HashMap<&str, Vec<&Task>> = build_children_map(tasks);
    let roots: Vec<&Task> = tasks
        .iter()
        .filter(|t| t.depends().is_empty())
        .copied()
        .collect();

    let mut visited = HashSet::new();
    for root in &roots {
        print_tree_node(root, &children, store, "", true, true, None, &mut visited);
    }

    for task in tasks {
        if !visited.contains(task.name()) {
            print_tree_node(task, &children, store, "", true, true, None, &mut visited);
        }
    }
}

fn build_children_map<'a>(tasks: &'a [&Task]) -> HashMap<&'a str, Vec<&'a Task>> {
    let mut map: HashMap<&str, Vec<&Task>> = HashMap::new();
    for task in tasks {
        for dep in task.depends() {
            map.entry(dep.as_str()).or_default().push(task);
        }
    }
    map
}

fn print_tree_node<'a>(
    task: &'a Task,
    children: &HashMap<&str, Vec<&'a Task>>,
    store: &TaskStore,
    prefix: &str,
    is_last: bool,
    is_root: bool,
    parent_name: Option<&str>,
    visited: &mut HashSet<&'a str>,
) {
    if visited.contains(task.name()) {
        return;
    }
    visited.insert(task.name());

    let connector = if is_root {
        ""
    } else if is_last {
        "└── "
    } else {
        "├── "
    };

    // Calculate other dependencies not shown in current tree path
    let other_deps: Vec<&str> = if let Some(parent) = parent_name {
        task.depends()
            .iter()
            .filter(|dep| dep.as_str() != parent)
            .map(|s| s.as_str())
            .collect()
    } else {
        vec![]
    };

    let other_deps_str = if other_deps.is_empty() {
        String::new()
    } else {
        format!(" (+{})", other_deps.join(", "))
    };

    let status = store.get_status(task.name());
    println!(
        "{}{}{} [{}]{}",
        prefix,
        connector,
        task.name(),
        status.icon(),
        other_deps_str
    );

    let new_prefix = if is_root {
        "".to_string()
    } else if is_last {
        format!("{}    ", prefix)
    } else {
        format!("{}│   ", prefix)
    };

    if let Some(task_children) = children.get(task.name()) {
        let count = task_children.len();
        for (i, child) in task_children.iter().enumerate() {
            let is_last_child = i == count - 1;
            print_tree_node(child, children, store, &new_prefix, is_last_child, false, Some(task.name()), visited);
        }
    }
}
