mod app;
mod args;
mod scenario;
mod smoke_admission;

use args::StorybookArgs;

fn main() -> eframe::Result<()> {
    let args = StorybookArgs::parse(std::env::args().skip(1));
    smoke_admission::validate(args.smoke)?;
    app::run(args)
}
