use win_color::Color;
use win_color::ColorImpl;
use win_color::GlobalColor;

fn main() {
    let gc = GlobalColor::String("#89b4fa".to_string());
    let gc_1 = GlobalColor::String("red".to_string());
    println!("{:?}", Color::fetch(&gc, None));
    println!("{:?}", Color::fetch(&gc_1, None));
}
