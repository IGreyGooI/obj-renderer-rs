// The MIT License

// Copyright 2019 Kikokushi <s468zhan@edu.uwaterloo.ca>
// Copyright 2016-2018 The Amethyst Project Developers

// Permission is hereby granted, free of charge, to any person obtaining a copy of this software
// and associated documentation files (the "Software"), to deal in the Software without
// restriction, including without limitation the rights to use, copy, modify, merge, publish,
// distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the
// Software is furnished to do so, subject to the following conditions:

// The above copyright notice and this permission notice shall be included in all copies or
// substantial portions of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING
// BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
// NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
// DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.


use std::{env, io, path};

/// Returns the cargo manifest directory when running the executable with cargo or the directory in
/// which the executable resides otherwise, traversing symlinks if necessary.
///
/// The algorithm used is:
///
/// * If the `CARGO_MANIFEST_DIR` environment variable is defined it is used as application root.
///   This simplifies running development projects through `cargo run`.
///   See the [cargo reference documentation][cargo-ref] for more details.
/// * If the executable name can be found using [`std::env::current_exe`], resolve all symlinks and
///   use the directory it resides in as application root.
///
/// If none of the above works, an error is returned.
pub fn application_root_dir() -> Result<path::PathBuf, io::Error> {
    if let Some(manifest_dir) = env::var_os("CARGO_MANIFEST_DIR") {
        return Ok(path::PathBuf::from(manifest_dir));
    }
    
    let mut exe = env::current_exe()?;
    
    // Modify in-place to avoid an extra copy.
    if exe.pop() {
        return Ok(exe);
    }
    
    Err(io::Error::new(
        io::ErrorKind::Other,
        "Failed to find an application root",
    ))
}

