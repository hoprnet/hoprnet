extern crate clicolors_control;

pub fn main() {
    if clicolors_control::colors_enabled() {
        println!("Colors are on!");
        println!("\x1b[36mThis is colored text.\x1b[0m");
    } else {
        println!("Someone turned off the colors :()")
    }
}
