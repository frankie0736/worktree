use wt::models::{Task, TaskFrontmatter, TaskStore};

fn make_task(name: &str, depends: Vec<&str>) -> Task {
    Task {
        frontmatter: TaskFrontmatter {
            name: name.to_string(),
            depends: depends.into_iter().map(String::from).collect(),
        },
        content: String::new(),
        file_path: format!(".wt/tasks/{}.md", name),
    }
}

#[test]
fn test_complex_graph_no_cycle() {
    //     a
    //    /|\
    //   b c d
    //    \|/
    //     e
    let mut store = TaskStore::default();
    store.tasks.insert("e".to_string(), make_task("e", vec![]));
    store
        .tasks
        .insert("b".to_string(), make_task("b", vec!["e"]));
    store
        .tasks
        .insert("c".to_string(), make_task("c", vec!["e"]));
    store
        .tasks
        .insert("d".to_string(), make_task("d", vec!["e"]));
    store
        .tasks
        .insert("a".to_string(), make_task("a", vec!["b", "c", "d"]));

    let errors = store.validate();
    let cycle_errors: Vec<_> = errors
        .iter()
        .filter(|(_, e)| e.contains("circular"))
        .collect();
    assert!(cycle_errors.is_empty());
}

#[test]
fn test_complex_graph_with_cycle() {
    //     a -> b -> c
    //          ^    |
    //          |    v
    //          e <- d
    let mut store = TaskStore::default();
    store
        .tasks
        .insert("a".to_string(), make_task("a", vec!["b"]));
    store
        .tasks
        .insert("b".to_string(), make_task("b", vec!["c"]));
    store
        .tasks
        .insert("c".to_string(), make_task("c", vec!["d"]));
    store
        .tasks
        .insert("d".to_string(), make_task("d", vec!["e"]));
    store
        .tasks
        .insert("e".to_string(), make_task("e", vec!["b"]));

    let errors = store.validate();
    let cycle_errors: Vec<_> = errors
        .iter()
        .filter(|(_, e)| e.contains("circular"))
        .collect();
    assert!(!cycle_errors.is_empty());
}

#[test]
fn test_multiple_independent_cycles() {
    // Cycle 1: a -> b -> a
    // Cycle 2: x -> y -> x
    let mut store = TaskStore::default();
    store
        .tasks
        .insert("a".to_string(), make_task("a", vec!["b"]));
    store
        .tasks
        .insert("b".to_string(), make_task("b", vec!["a"]));
    store
        .tasks
        .insert("x".to_string(), make_task("x", vec!["y"]));
    store
        .tasks
        .insert("y".to_string(), make_task("y", vec!["x"]));

    let errors = store.validate();
    let cycle_errors: Vec<_> = errors
        .iter()
        .filter(|(_, e)| e.contains("circular"))
        .collect();
    assert!(cycle_errors.len() >= 2);
}

#[test]
fn test_partial_cycle_detection() {
    // a -> b -> c (no cycle, but b -> c -> b creates cycle)
    let mut store = TaskStore::default();
    store
        .tasks
        .insert("a".to_string(), make_task("a", vec!["b"]));
    store
        .tasks
        .insert("b".to_string(), make_task("b", vec!["c"]));
    store
        .tasks
        .insert("c".to_string(), make_task("c", vec!["b"]));

    let errors = store.validate();
    let cycle_errors: Vec<_> = errors
        .iter()
        .filter(|(_, e)| e.contains("circular"))
        .collect();
    assert!(!cycle_errors.is_empty());
}
