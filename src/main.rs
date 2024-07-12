// Date:   Thu Jul 11 22:55:13 2024
// Mail:   lunar_ubuntu@qq.com
// Author: https://github.com/xiaoqixian

use auto_from::auto_throw;

#[auto_throw]
enum Error {
    IOError(std::io::Error),
    FmtError(std::fmt::Error),
    TupleError(i32, String),
    UnitError,
}

fn main() {}
