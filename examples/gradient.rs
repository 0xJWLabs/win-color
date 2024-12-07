use win_color::Color;
use win_color::ColorImpl;
use win_color::GlobalColor;
use win_color::GradientDirection;
use win_color::GradientMapping;

fn main() {
    println!(
        "{:?}",
        Color::try_from_string("gradient(#89b4fa, #cba6f7, to right)", None)
    );

    println!(
        "{:?}",
        Color::try_from_global(
            &GlobalColor::Mapping(GradientMapping {
                colors: vec!["#89b4fa".to_string(), "#cba6f7".to_string()],
                direction: GradientDirection::Direction("to right".to_string())
            }),
            None
        )
    );
}
