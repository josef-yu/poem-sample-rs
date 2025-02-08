use std::future::Future;
use std::panic::{self, AssertUnwindSafe};
use std::fs::File;

use futures::FutureExt;


pub static TEST_FILE_NAME: &str = "./test-data.json";

pub fn run_with_file_create_teardown<T>(test: T) -> ()
    where T: FnOnce() -> () + panic::UnwindSafe
{
    let _ = File::create(TEST_FILE_NAME);

    let result = panic::catch_unwind(|| {
        test()
    });

    let _ = std::fs::remove_file(TEST_FILE_NAME);

    assert!(result.is_ok())
}


pub async fn async_run_with_file_create_teardown<T, U>(test: T) -> ()
    where T: FnOnce() -> U + panic::UnwindSafe,
        U: Future<Output = ()>
{
    let _ = File::create(TEST_FILE_NAME);

    let result = AssertUnwindSafe(test())
        .catch_unwind()
        .await;

    let _ = std::fs::remove_file(TEST_FILE_NAME);

    assert!(result.is_ok())
}