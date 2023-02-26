// Test case that compares a file generated by rust_xlsxwriter with a file
// created by Excel.
//
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright 2022-2023, John McNamara, jmcnamara@cpan.org

use rust_xlsxwriter::{Chart, ChartSeries, ChartType, Format, Workbook, XlsxError};

mod common;

// Test to demonstrate charts.
fn create_new_xlsx_file(filename: &str) -> Result<(), XlsxError> {
    let mut workbook = Workbook::new();

    let worksheet = workbook.add_worksheet();

    // Add some test data for the chart(s).
    let data = [[1, 2, 3], [2, 4, 6], [3, 6, 9], [4, 8, 12], [5, 10, 15]];
    for (row_num, row_data) in data.iter().enumerate() {
        for (col_num, col_data) in row_data.iter().enumerate() {
            worksheet.write_number(row_num as u32, col_num as u16, *col_data)?;
        }
    }

    let mut chart = Chart::new(ChartType::Bar);
    chart.set_axis_ids(40274944, 40294272);
    chart
        .push_series(&ChartSeries::new().set_values(("Sheet1", 0, 0, 4, 0)))
        .push_series(&ChartSeries::new().set_values(("Sheet1", 0, 1, 4, 1)))
        .push_series(&ChartSeries::new().set_values(("Sheet1", 0, 2, 4, 2)));

    worksheet.insert_chart(8, 4, &chart)?;

    let mut chart = Chart::new(ChartType::Bar);
    chart.set_axis_ids(62355328, 62356864);
    chart
        .push_series(&ChartSeries::new().set_values(("Sheet1", 0, 0, 4, 0)))
        .push_series(&ChartSeries::new().set_values(("Sheet1", 0, 1, 4, 1)));

    worksheet.insert_chart(24, 3, &chart)?;

    let mut chart = Chart::new(ChartType::Bar);
    chart.set_axis_ids(79538816, 65422464);
    chart.push_series(&ChartSeries::new().set_values(("Sheet1", 0, 0, 4, 0)));

    worksheet.insert_chart(31, 11, &chart)?;

    let format = Format::default();

    worksheet.write_url_with_format(6, 0, "http://www.perl.com/", &format)?;
    worksheet.write_url_with_format(7, 0, "http://www.perl.org/", &format)?;
    worksheet.write_url_with_format(8, 0, "http://www.perl.net/", &format)?;

    workbook.save(filename)?;

    Ok(())
}

#[test]
fn test_chart_bar11() {
    let test_runner = common::TestRunner::new()
        .set_name("chart_bar11")
        .set_function(create_new_xlsx_file)
        .initialize();

    test_runner.assert_eq();
    test_runner.cleanup();
}
