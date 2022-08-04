#[cfg(feature = "ui")]
mod app;
mod asymptote;
mod asymptote_list;
mod cli;
#[cfg(all(feature = "pbf", not(target_arch = "wasm32")))]
mod node_db;
#[cfg(all(feature = "pbf", not(target_arch = "wasm32")))]
mod pbf;
mod status;

use anyhow::Result;
use clap::Parser;
use cli::*;
use std::time::Duration;

fn main() -> Result<()> {
    env_logger::init();

    let cli = Cli::parse();

    // You can check for the existence of subcommands, and if found use their
    // matches just as you would the top level cmd
    match &cli.command {
        Commands::Asymptote { filters } => {
            for algo in asymptote_list::ALGORITHMS {
                println!("Algorithm: {}", algo.name);
                dbg!((algo.run)(Default::default()));
            }
            // dbg!(asymptote::asymptote(
            //     Default::default(),
            //     |step| {
            //         (
            //             Duration::from_secs_f64(step as f64 * (step as f64).log2() * 0.1),
            //             step as asymptote::Factor * (step as f64).log2(),
            //         )
            //     },
            //     |i| std::thread::sleep(i),
            // ));
            // println!("'myapp add' was used, name is: {:?}", filters)
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
