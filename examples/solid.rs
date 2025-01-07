use win_color::GlobalColor;
use win_color::GlobalColorImpl;

fn main() {
    let gc = GlobalColor::String("#89b4fa".to_string());
    let gc_1 = GlobalColor::String("red".to_string());
    println!("{:?}", &gc.to_color());
    println!("{:?}", gc_1.to_color());
}
