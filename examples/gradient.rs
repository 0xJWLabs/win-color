use win_color::Color;
use win_color::ColorImpl;
use win_color::ColorMapping;
use win_color::ColorMappingImpl;
use win_color::GlobalColor;
use win_color::GradientDirection;

fn main() {
    let gc = GlobalColor::String("gradient(#89b4fa, #cba6f7, to right)".to_string());
    let gc_mapping = GlobalColor::Mapping(ColorMapping::new(
        &["#89b4fa", "#cba6f7"],
        GradientDirection::from("40grad"),
    ));
    println!("{:?}", Color::fetch(&gc, None));
    println!("{:?}", Color::fetch(&gc_mapping, None));
}
