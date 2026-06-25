pub fn scaffold_observer(name: &str) -> String {
    format!(
        "pub struct {}Observer;\n\nimpl Observer for {}Observer {{ ... }}",
        name, name
    )
}
pub fn scaffold_detector(name: &str) -> String {
    format!(
        "pub struct {}Detector;\n\nimpl Detector for {}Detector {{ ... }}",
        name, name
    )
}
pub fn scaffold_strategy(name: &str) -> String {
    format!(
        "pub struct {}Strategy;\n\nimpl Strategy for {}Strategy {{ ... }}",
        name, name
    )
}
