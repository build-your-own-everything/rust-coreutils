fn main() {
    if let Err(e) = findr::get_args().and_then(findr::run) {
        eprint!("{e}");
        std::process::exit(1);
    }
}
