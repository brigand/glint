use crossterm::style::{style, Color, Print, StyledContent};

pub fn reset() -> Color {
    Color::Reset
}

pub fn reset_display() -> StyledContent<&'static str> {
    style("").with(Color::Reset)
}

pub fn reset_item() -> Print<String> {
    Print(style("").with(reset()).on(reset()).to_string())
}

pub fn theme_user_input() -> Color {
    Color::Blue
}

// pub fn glint_type_to_color(ty: &str) -> Color {
//     match ty {
//         "build" => Color::White,
//         "ci" => Color::Blue,
//         "chore" => Color::Yellow,
//         "docs" => Color::DarkBlue,
//         "feat" => Color::Blue,
//         "fix" => Color::Red,
//         "perf" => Color::Green,
//         "refactor" => Color::DarkCyan,
//         "revert" => Color::DarkRed,
//         "style" => Color::Cyan,
//         "test" => Color::Magenta,
//         _ => Color::White,
//     }
// }
