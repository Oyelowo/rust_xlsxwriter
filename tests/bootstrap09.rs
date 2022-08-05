// Test case that compares a file generated by rust_xlsxwriter with a file
// created by Excel.
//
// SPDX-License-Identifier: MIT OR Apache-2.0 Copyright 2022, John McNamara,
// jmcnamara@cpan.org

use rust_xlsxwriter::{Workbook, XlsxError};

mod common;

// Test case to demonstrate creating a basic file with some string cell data.
fn create_new_xlsx_file(filename: &str) -> Result<(), XlsxError> {
    let mut workbook = Workbook::new(filename);

    let mut format = workbook.add_format();
    format.set_bold().register_with(&mut workbook);

    let worksheet = workbook.add_worksheet();
    worksheet.write_string(0, 0, "Hello", &format)?;

    workbook.close()?;

    Ok(())
}

#[test]
fn bootstrap09_write_a_formatted_string() {
    let testcase = "bootstrap09";

    let (excel_file, xlsxwriter_file) = common::get_xlsx_filenames(testcase);
    _ = create_new_xlsx_file(&xlsxwriter_file);
    common::assert_eq(&excel_file, &xlsxwriter_file);
    common::remove_test_xlsx_file(&xlsxwriter_file);
}
