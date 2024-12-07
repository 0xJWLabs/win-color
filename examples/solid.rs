use win_color::Color;
use win_color::ColorImpl;

fn main() {
    println!("{:?}", Color::try_from_string("#89b4fa", None));
    println!("{:?}", Color::try_from_string("red", None));
}
