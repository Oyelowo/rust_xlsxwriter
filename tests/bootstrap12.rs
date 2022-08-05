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

    let mut format1 = workbook.add_format();
    format1.set_bold().register_with(&mut workbook);

    let mut format2 = workbook.add_format();
    format2.set_italic().register_with(&mut workbook);

    let mut format3 = workbook.add_format();
    format3.set_bold().set_italic().register_with(&mut workbook);

    let worksheet = workbook.add_worksheet();
    worksheet.write_string(0, 0, "Hello", &format1)?;
    worksheet.write_string(1, 0, "Hello", &format2)?;
    worksheet.write_string(2, 0, "Hello", &format3)?;

    workbook.close()?;

    Ok(())
}

#[test]
fn bootstrap12_bold_and_italic_text_mixed() {
    let testcase = "bootstrap12";

    let (excel_file, xlsxwriter_file) = common::get_xlsx_filenames(testcase);
    _ = create_new_xlsx_file(&xlsxwriter_file);
    common::assert_eq(&excel_file, &xlsxwriter_file);
    common::remove_test_xlsx_file(&xlsxwriter_file);
}
