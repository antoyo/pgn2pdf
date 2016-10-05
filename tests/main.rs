/*
 * Copyright (c) 2016 Boucher, Antoni <bouanto@zoho.com>
 *
 * Permission is hereby granted, free of charge, to any person obtaining a copy of
 * this software and associated documentation files (the "Software"), to deal in
 * the Software without restriction, including without limitation the rights to
 * use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
 * the Software, and to permit persons to whom the Software is furnished to do so,
 * subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be included in all
 * copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
 * IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
 * FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
 * COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
 * IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
 * CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
 */

extern crate tempdir;

use std::fs::{File, create_dir_all, remove_dir_all};
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Command, Stdio};

fn assert_files<P: AsRef<Path>, Q: AsRef<Path>>(expected: P, actual: Q) {
    let expected = format!("{}/{}", std::env::current_dir().unwrap().to_str().unwrap(), expected.as_ref().to_str().unwrap());
    let mut expected_file = BufReader::new(File::open(expected).unwrap());
    let mut expected_output = vec![];
    let mut actual_file = BufReader::new(File::open(actual).unwrap());
    let mut actual_output = vec![];
    loop {
        let size1 = expected_file.read_until(b'\n', &mut expected_output).unwrap();
        let size2 = actual_file.read_until(b'\n', &mut actual_output).unwrap();
        assert_eq!(size1, size2);
        if size1 == 0 || size2 == 0 {
            break;
        }
        let tag1 = b"/CreationDate";
        let tag2 = b"/ModDate";
        if (expected_output.starts_with(tag1) && actual_output.starts_with(tag1)) || (expected_output.starts_with(tag2) && actual_output.starts_with(tag2))  {
            expected_output.clear();
            actual_output.clear();
            continue
        }
        assert_eq!(expected_output, actual_output);
        expected_output.clear();
        actual_output.clear();
    }
}

fn compare(input: &str) {
    let tempdir = format!("/tmp/pgn2pdf-{}", input);
    create_dir_all(&tempdir).unwrap();
    let input_filename = format!("{}.pgn", input);
    let current_dir = std::env::current_dir().unwrap();
    let current_dir = current_dir.to_str().unwrap();
    let input_path = format!("{}/tests/{}", current_dir, input_filename);
    let output_filename = format!("{}.pdf", input);
    let expected_output = format!("tests/{}", output_filename);
    let output_path = format!("{}/{}", tempdir, output_filename);
    let exe = format!("{}/target/debug/pgn2pdf", current_dir);
    Command::new(exe)
        .arg(input_path)
        .arg("-o")
        .arg(output_path)
        .stdout(Stdio::null())
        .status()
        .unwrap();
    let actual_output = format!("{}/{}.pdf", tempdir, input);
    assert_files(expected_output, actual_output);
    remove_dir_all(tempdir).unwrap();
}

macro_rules! compare {
    ($ident:ident) => {
        #[test]
        fn $ident() {
            compare(stringify!($ident));
        }
    };
}

compare!(test1);
compare!(test2);
compare!(test3);
compare!(test4);
compare!(test5);
compare!(test6);
compare!(test7);
compare!(test8);
