use std::panic;
use std::fs::File;


static FILE_NAME: &str = "./data.json";

pub fn run_with_file_create_teardown<T>(test: T) -> ()
    where T: FnOnce() -> () + panic::UnwindSafe
{
    let _ = File::create(FILE_NAME);

    let result = panic::catch_unwind(|| {
        test()
    });

    let _ = std::fs::remove_file(FILE_NAME);

    assert!(result.is_ok())
}