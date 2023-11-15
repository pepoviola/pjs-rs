use std::env;
use pjs_rs::{run_file, ReturnValue};
fn main() {
    let args = &env::args().collect::<Vec<String>>()[1..];

    if args.is_empty() {
        eprintln!("Usage: pjs <file>");
        std::process::exit(1);
    }
    let file_path = &args[0];

    let tk_runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    tk_runtime.block_on(async {
        let result = run_file(file_path, None).await;
        match result {
            Err(error) => {
                eprintln!("error: {error}");
            },
            Ok(v) => {
                match v {
                    ReturnValue::Deserialized(value) => {
                        println!("{value}");
                    }
                    ReturnValue::CantDeserialize(de_error) => {
                        println!("script success but we can't deserialize returned value, err: {de_error}");
                    }
                }
            }
        }
    });
}
