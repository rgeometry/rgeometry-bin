#[cfg(feature = "ui")]
mod app;
mod cli;
#[cfg(all(feature = "pbf", not(target_arch = "wasm32")))]
mod node_db;
#[cfg(all(feature = "pbf", not(target_arch = "wasm32")))]
mod pbf;
mod status;

use anyhow::Result;
use clap::Parser;
use cli::*;

fn main() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Commands::Asymptote { filters } => {
            println!("'myapp add' was used, name is: {:?}", filters)
        }
        Commands::Pbf { path } => {
            #[cfg(all(feature = "pbf", not(target_arch = "wasm32")))]
            {
                pbf::create_db(path)?;
                pbf::test_db()?;
            }
            #[cfg(not(all(feature = "pbf", not(target_arch = "wasm32"))))]
            {
                println!("PBF support not available with wasm");
            }
        }
    }
    Ok(())

    // create_db()
    // test_db()
    // create_polygons()
    // gui::main();
    // let app = app::TemplateApp::default();
    // let native_options = eframe::NativeOptions::default();
    // eframe::run_native(Box::new(app), native_options);
}
