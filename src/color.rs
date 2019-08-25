use crossterm::Color;

pub fn clint_type_to_color(ty: &str) -> Color {
    match ty {
        "build" => Color::White,
        "ci" => Color::Blue,
        "chore" => Color::Yellow,
        "docs" => Color::DarkBlue,
        "feat" => Color::Blue,
        "fix" => Color::Red,
        "perf" => Color::Green,
        "refactor" => Color::DarkCyan,
        "revert" => Color::DarkRed,
        "style" => Color::Cyan,
        "test" => Color::Magenta,
        _ => Color::White,
    }
}
