use clap;
use clap::{Parser, Subcommand};
use rs_hp4x::{parse_hp4x, Extable, Obj};
use anyhow::Result;
use std::io::Write;
use rs_hp4x::decompile::Decompiled;

#[derive(Parser)]
#[command(name = "rs-hp4x")]
#[command(about = "A CLI tool to dump extable and arbitrary objects", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// The path to the extable.HP file, to decode the Ext objects
    #[arg(short, long)]
    with_extable: Option<String>,

}

#[derive(Subcommand)]
enum Commands {
    /// Dump the extable into a csv file
    DumpExtable {
        /// The output file path
        #[arg(short, long)]
        output: String,
    },
    /// Dump an arbitrary object into a directory
    DumpObject {
        /// The path to an object to dump
        #[arg(long)]
        object: String,
        /// The output directory path
        #[arg(short, long)]
        output_dir: String,
    },
}
fn get_extable(path: &str) -> Result<Extable> {
    let obj = parse_hp4x(&std::path::Path::new(path))?;
    if let Obj::Library(lib) = obj {
        Ok(Extable::from(lib))
    } else {
        Err(anyhow::anyhow!("Failed to parse extable"))
    }
}

fn main()  -> Result<()> {
    let cli = Cli::parse();
    let extable = if let Some(path) = &cli.with_extable {
        Some(get_extable(path)?)
    } else {
        None
    };
    match &cli.command {
        Commands::DumpExtable { output } => {
            if extable.is_none() {
                eprintln!("No extable provided, exiting");
                std::process::exit(1);
            }
            let extable = extable.unwrap();
            println!("Dumping extable to file: {}", output);

            let mut out = std::fs::File::create(output)?;
            writeln!(out, "Name,Address")?;
            for (name, addr) in extable.name_to_addr.iter() {
                writeln!(out, "{},0x{:x}", name, addr)?;
            }
        }
        Commands::DumpObject { object, output_dir } => {
            println!("Dumping object {} to directory: {}", object, output_dir);
            let extable = extable.unwrap_or_default();
            let in_path = std::path::Path::new(object);
            let obj = parse_hp4x(in_path)?;
            fn dump_object(obj: &Obj, output_name: &str, extable: &Extable) -> Result<()> {
                match obj {
                    Obj::Dir(dir) => {
                        std::fs::create_dir_all(output_name)?;
                        for e in dir.entities.iter() {
                            dump_object(&e.obj, &format!("{}/{}", output_name, e.name), extable)?;
                        }
                    }
                    _ => {
                        let mut out = std::fs::File::create(output_name)?;
                        out.write_all(obj.decompile(extable).as_bytes())?;
                    }
                }
                Ok(())
            }
            dump_object(&obj, output_dir, &extable)?;
        }

    }
    Ok(())
}