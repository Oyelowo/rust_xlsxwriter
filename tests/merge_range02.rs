// Test case that compares a file generated by rust_xlsxwriter with a file
// created by Excel.
//
// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2022, John McNamara, jmcnamara@cpan.org

use rust_xlsxwriter::{Format, Workbook, XlsxAlign, XlsxError};

mod common;

// Test to demonstrate merged ranges.
fn create_new_xlsx_file(filename: &str) -> Result<(), XlsxError> {
    let mut workbook = Workbook::new();

    let worksheet = workbook.add_worksheet();

    let format = Format::new().set_align(XlsxAlign::Center);

    worksheet.merge_range(1, 1, 5, 3, "Foo", &format)?;

    workbook.save(filename)?;

    Ok(())
}

#[test]
fn test_merge_range02() {
    let test_runner = common::TestRunner::new("merge_range02").initialize();

    let result = create_new_xlsx_file(test_runner.output_file());
    assert_result!(result);

    test_runner.assert_eq();
    test_runner.cleanup();
}
