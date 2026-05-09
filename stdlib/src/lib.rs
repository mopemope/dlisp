pub fn get_module(name: &str) -> Option<&'static str> {
    match name {
        "core" => Some(include_str!("core.lisp")),
        _ => None,
    }
}

pub fn module_names() -> &'static [&'static str] {
    &["core"]
}
