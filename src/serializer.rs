// serializer - A serde serializer for use with `rust_xlsxwriter`.
//
// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Copyright 2022-2023, John McNamara, jmcnamara@cpan.org

//! # Working with Serde
//!
//! Serialization is the process of converting data from one format to another
//! using rules shared between the data and the serializer. The
//! [Serde](https://serde.rs) crate allow you to attach these type of rules to
//! data and it also provides traits that can be implemented by serializers to
//! convert that data into different formats. The `rust_xlsxwriter` crate
//! implements the Serde [`serde::ser::Serializer`] trait for [`Worksheet`]
//! structs which allows you to serialize data directly to a worksheet.
//!
//! The following sections explains how to serialize Serde enabled data to an
//! Excel worksheet using `rust_xlsxwriter`.
//!
//!
//! Contents:
//!
//! - [How serialization works in
//!   `rust_xlsxwriter`](#how-serialization-works-in-rust_xlsxwriter)
//! - [Setting serialization headers](#setting-serialization-headers)
//! - [Renaming fields when serializing](#renaming-fields-when-serializing)
//! - [Skipping fields when serializing](#skipping-fields-when-serializing)
//! - [Setting serialization formatting](#setting-serialization-formatting)
//! - [Limitations of serializing to
//!   Excel](#limitations-of-serializing-to-excel)
//!
//! **Note**: This functionality requires the use of the `serde` feature flag
//! with `rust_xlsxwriter`:
//!
//! ```bash
//! cargo add rust_xlsxwriter -F serde
//! ```
//!
//!
//!
//!
//! ## How serialization works in `rust_xlsxwriter`
//!
//! Serialization with `rust_xlsxwriter` needs to take into consideration that
//! that the target output is a 2D grid of cells into which the data can be
//! serialized. As such the focus is on serializing data types that map to this
//! 2D grid such as structs or compound collections of structs such as vectors
//! or tuples and it (currently) ignores compound types like maps.
//!
//! The image below shows the basic scheme for mapping a struct to a worksheet:
//! fields are mapped to a header and values are mapped to sequential cells
//! below the header.
//!
//! <img src="https://rustxlsxwriter.github.io/images/serialize_intro1.png">
//!
//! This scheme needs an initial (row, col) location from which to start
//! serializing to allow the data to be positioned anywhere on the worksheet.
//! Subsequent serializations will be in the same columns (for the target struct
//! type) but will be one row lower in the worksheet.
//!
//! The type name and fields of the struct being serialized is also required
//! information. We will look at that in more detail in the next section.
//!
//! Here is an example program that demonstrates the basic steps for serializing
//! data to an Excel worksheet:
//!
//!
//! ```
//! # // This code is available in examples/doc_worksheet_serialize_intro.rs
//! #
//! use rust_xlsxwriter::{Format, FormatBorder, Workbook, XlsxError};
//! use serde::Serialize;
//!
//! fn main() -> Result<(), XlsxError> {
//!     let mut workbook = Workbook::new();
//!
//!     // Add a worksheet to the workbook.
//!     let worksheet = workbook.add_worksheet();
//!
//!     // Add some formats to use with the serialization data.
//!     let header_format = Format::new()
//!         .set_bold()
//!         .set_border(FormatBorder::Thin)
//!         .set_background_color("C6E0B4");
//!
//!     // Create a serializable test struct and add serialization information.
//!     #[derive(Serialize)]
//!     #[serde(rename_all = "PascalCase")]
//!     struct Student<'a> {
//!         name: &'a str,
//!         age: u8,
//!         id: u32,
//!     }
//!
//!     // Create an array of data to serialize.
//!     let students = [
//!         Student {
//!             name: "Aoife",
//!             age: 25,
//!             id: 564351,
//!         },
//!         Student {
//!             name: "Caoimhe",
//!             age: 21,
//!             id: 443287,
//!         },
//!     ];
//!
//!     // Set up the start location and headers of the data to be serialized.
//!     worksheet.serialize_headers_with_format(1, 3, &students.get(0).unwrap(), &header_format)?;
//!
//!     // Serialize the data.
//!     worksheet.serialize(&students)?;
//!
//!     // Save the file.
//!     workbook.save("serialize.xlsx")?;
//!
//!     Ok(())
//! }
//! ```
//!
//! Output file:
//!
//! <img src="https://rustxlsxwriter.github.io/images/serialize_intro2.png">
//!
//!
//!
//!
//! ## Setting serialization headers
//!
//! In addition to the cell location shown in the previous section, the
//! type/name of the struct being serialized and the fields it contains are
//! required information so that multiple types of structs can be serialized to
//! the same worksheet. This information also ensures that fields with identical
//! names are mapped to the correct parent struct location.
//!
//! There are three worksheet methods to support this functionality.
//!
//! - [`Worksheet::serialize_headers()`]: The simplest most direct method. It
//!   requires a argument of the type of struct that you wish to serialize. The
//!   library uses this to infer the struct name and fields (via serialization).
//! - [`Worksheet::serialize_headers_with_format()`]: This is similar to the
//!   previous method but it allows you to add a cell format for the headers.
//! - [`Worksheet::serialize_headers_with_options()`]: This requires the struct
//!   type to be passed as a string and an array of [`CustomSerializeHeader`]
//!   objects. However it gives the highest degree of control over the output.
//!   This method also allows you to turn off the headers above the serialized
//!   data.
//!
//! The example below shows the usage of each method.
//!
//! ```
//! # // This code is available in examples/doc_worksheet_serialize_headers3.rs
//! #
//! use rust_xlsxwriter::{CustomSerializeHeader, Format, FormatBorder, Workbook, XlsxError};
//! use serde::Serialize;
//!
//! fn main() -> Result<(), XlsxError> {
//!     let mut workbook = Workbook::new();
//!
//!     // Add a worksheet to the workbook.
//!     let worksheet = workbook.add_worksheet();
//!
//!     // Set some column widths for clarity.
//!     worksheet.set_column_width(2, 4)?;
//!     worksheet.set_column_width(5, 4)?;
//!     worksheet.set_column_width(8, 4)?;
//!
//!     // Add some formats to use with the serialization data.
//!     let header_format = Format::new()
//!         .set_bold()
//!         .set_border(FormatBorder::Thin)
//!         .set_background_color("C6EFCE");
//!
//!     let currency_format = Format::new().set_num_format("$0.00");
//!
//!     // Create a serializable test struct.
//!     #[derive(Serialize)]
//!     struct Produce {
//!         fruit: &'static str,
//!         cost: f64,
//!     }
//!
//!     // Create some data instances.
//!     let item1 = Produce {
//!         fruit: "Peach",
//!         cost: 1.05,
//!     };
//!
//!     let item2 = Produce {
//!         fruit: "Plum",
//!         cost: 0.15,
//!     };
//!
//!     let item3 = Produce {
//!         fruit: "Pear",
//!         cost: 0.75,
//!     };
//!
//!     // 1. Set the serialization location and headers with `serialize_headers()`
//!     //    and serialize some data.
//!     worksheet.serialize_headers(0, 0, &item1)?;
//!     worksheet.serialize(&item1)?;
//!     worksheet.serialize(&item2)?;
//!     worksheet.serialize(&item3)?;
//!
//!     // 2. Set the serialization location and formatted headers with
//!     //    `serialize_headers_with_format()` and serialize some data.
//!     worksheet.serialize_headers_with_format(0, 3, &item1, &header_format)?;
//!     worksheet.serialize(&item1)?;
//!     worksheet.serialize(&item2)?;
//!     worksheet.serialize(&item3)?;
//!
//!     // 3. Set the serialization location and headers with custom headers. We use
//!     //    the customization to set the header format and also the cell format
//!     //    for the number values.
//!     let custom_headers = [
//!         CustomSerializeHeader::new("fruit")
//!             .rename("Item")
//!             .set_header_format(&header_format),
//!         CustomSerializeHeader::new("cost")
//!             .rename("Price")
//!             .set_header_format(&header_format)
//!             .set_cell_format(&currency_format),
//!     ];
//!
//!     worksheet.serialize_headers_with_options(0, 6, "Produce", &custom_headers)?;
//!     worksheet.serialize(&item1)?;
//!     worksheet.serialize(&item2)?;
//!     worksheet.serialize(&item3)?;
//!
//!     // 4. Set the serialization location and headers with custom headers. We use
//!     //    the customization to turn off the headers.
//!     let custom_headers = [
//!         CustomSerializeHeader::new("fruit").hide_headers(true),
//!         CustomSerializeHeader::new("cost"),
//!     ];
//!
//!     worksheet.serialize_headers_with_options(0, 9, "Produce", &custom_headers)?;
//!     worksheet.serialize(&item1)?;
//!     worksheet.serialize(&item2)?;
//!     worksheet.serialize(&item3)?;
//!
//!     // Save the file.
//!     workbook.save("serialize.xlsx")?;
//!
//!     Ok(())
//! }
//! ```
//!
//! Output file:
//!
//! <img
//! src="https://rustxlsxwriter.github.io/images/worksheet_serialize_headers3.png">
//!
//! One final note, you can also overwrite the headers after serialization using
//! standard [`Worksheet`] `write()` methods. This allows additional levels of
//! control over the header formatting.
//!
//!
//!
//! ## Renaming fields when serializing
//!
//! As explained above serialization converts the field names of structs to
//! column headers at the top of serialized data. The default field names are
//! generally lowercase and snake case and may not be the way you want the header
//! names displayed in Excel. In which case you can use one of the two main
//! methods to rename the fields/headers:
//!
//! 1. Rename the field during serialization using the Serde [field attribute]
//!    `#[serde(rename = "name")` or the [container attribute]
//!    `#[serde(rename_all = "...")]`.
//! 2. Rename the header (not field) when setting up custom serialization
//!    headers via [`Worksheet::serialize_headers_with_options()`] and
//!    [`CustomSerializeHeader::rename()`].
//!
//! [field attribute]: https://serde.rs/field-attrs.html
//! [container attribute]: https://serde.rs/container-attrs.html
//!
//! Examples of these methods are shown below.
//!
//! ### Examples of field renaming
//!
//! The following example demonstrates renaming fields during serialization by
//! using Serde field attributes.
//!
//! ```
//! # // This code is available in examples/doc_worksheet_serialize_headers_rename1.rs
//! #
//! # use rust_xlsxwriter::{Workbook, XlsxError};
//! # use serde::Serialize;
//! #
//! # fn main() -> Result<(), XlsxError> {
//! #     let mut workbook = Workbook::new();
//! #
//! #     // Add a worksheet to the workbook.
//! #     let worksheet = workbook.add_worksheet();
//! #
//!     // Create a serializable test struct. Note the serde attributes.
//!     #[derive(Serialize)]
//!     struct Produce {
//!         #[serde(rename = "Item")]
//!         fruit: &'static str,
//!
//!         #[serde(rename = "Price")]
//!         cost: f64,
//!     }
//!
//!     // Create some data instances.
//!     let item1 = Produce {
//!         fruit: "Peach",
//!         cost: 1.05,
//!     };
//!
//!     let item2 = Produce {
//!         fruit: "Plum",
//!         cost: 0.15,
//!     };
//!
//!     let item3 = Produce {
//!         fruit: "Pear",
//!         cost: 0.75,
//!     };
//!
//!     // Set the serialization location and headers.
//!     worksheet.serialize_headers(0, 0, &item1)?;
//!
//!     // Serialize the data.
//!     worksheet.serialize(&item1)?;
//!     worksheet.serialize(&item2)?;
//!     worksheet.serialize(&item3)?;
//! #
//! #     // Save the file.
//! #     workbook.save("serialize.xlsx")?;
//! #
//! #     Ok(())
//! # }
//! ```
//!
//! Output file:
//!
//! <img
//! src="https://rustxlsxwriter.github.io/images/worksheet_serialize_headers_rename1.png">
//!
//!
//! The following example demonstrates renaming fields during serialization by
//! specifying custom headers and renaming them there. The output is the same as
//! the image above.
//!
//!
//! ```
//! # // This code is available in examples/doc_worksheet_serialize_headers_rename2.rs
//! #
//! # use rust_xlsxwriter::{CustomSerializeHeader, Workbook, XlsxError};
//! # use serde::Serialize;
//! #
//! # fn main() -> Result<(), XlsxError> {
//! #     let mut workbook = Workbook::new();
//! #
//! #     // Add a worksheet to the workbook.
//! #     let worksheet = workbook.add_worksheet();
//! #
//!     // Create a serializable test struct.
//!     #[derive(Serialize)]
//!     struct Produce {
//!         fruit: &'static str,
//!         cost: f64,
//!     }
//!
//!     // Create some data instances.
//!     let item1 = Produce {
//!         fruit: "Peach",
//!         cost: 1.05,
//!     };
//!
//!     let item2 = Produce {
//!         fruit: "Plum",
//!         cost: 0.15,
//!     };
//!
//!     let item3 = Produce {
//!         fruit: "Pear",
//!         cost: 0.75,
//!     };
//!
//!     // Set up the custom headers.
//!     let custom_headers = [
//!         CustomSerializeHeader::new("fruit").rename("Item"),
//!         CustomSerializeHeader::new("cost").rename("Price"),
//!     ];
//!
//!     // Set the serialization location and custom headers.
//!     worksheet.serialize_headers_with_options(0, 0, "Produce", &custom_headers)?;
//!
//!     // Serialize the data.
//!     worksheet.serialize(&item1)?;
//!     worksheet.serialize(&item2)?;
//!     worksheet.serialize(&item3)?;
//! #
//! #     // Save the file.
//! #     workbook.save("serialize.xlsx")?;
//! #
//! #     Ok(())
//! # }
//! ```
//!
//! Note: There is actually a third method which is to overwrite the cells of
//! the header with [`Worksheet::write()`] or
//! [`Worksheet::write_with_format()`]. This is a little bit more manual but has
//! the same effect as the two methods above.
//!
//!
//!
//!
//! ## Skipping fields when serializing
//!
//! When serializing a struct you may not want all of the fields to be
//! serialized. For example the struct may contain internal fields that aren't
//! of interest to the end user. There are several ways to skip fields:
//!
//! 1. Using the Serde [field attributes] `#[serde(skip)]` or
//!    `#[serde(skip_serializing)]`.
//! 2. Explicitly omitted the field when setting up custom serialization headers
//!    via [`Worksheet::serialize_headers_with_options()`]. This method is
//!    probably the most flexible since it doesn't require any additional
//!    attributes on the struct.
//! 3. Marking the field as skippable via custom headers and the
//!    [`CustomSerializeHeader::skip()`] method. This is only required in a few
//!    edge cases where you want to reserialize a struct to different parts of
//!    the worksheet with different combinations of fields displayed.
//!
//! [field attributes]: https://serde.rs/field-attrs.html
//!
//! Examples of all three methods are shown below.
//!
//!
//! ### Examples of field skipping
//!
//! The following example demonstrates skipping fields during serialization by
//! using Serde field attributes. Since the field is no longer used we also need
//! to tell `rustc` not to emit a `dead_code` warning.
//!
//! ```
//! # // This code is available in examples/doc_worksheet_serialize_headers_skip1.rs
//! #
//! use rust_xlsxwriter::{Workbook, XlsxError};
//! use serde::Serialize;
//!
//! fn main() -> Result<(), XlsxError> {
//!     let mut workbook = Workbook::new();
//!
//!     // Add a worksheet to the workbook.
//!     let worksheet = workbook.add_worksheet();
//!
//!     // Create a serializable test struct. Note the serde attribute.
//!     #[derive(Serialize)]
//!     struct Produce {
//!         fruit: &'static str,
//!         cost: f64,
//!
//!         #[serde(skip_serializing)]
//!         #[allow(dead_code)]
//!         in_stock: bool,
//!     }
//!
//!     // Create some data instances.
//!     let item1 = Produce {
//!         fruit: "Peach",
//!         cost: 1.05,
//!         in_stock: true,
//!     };
//!
//!     let item2 = Produce {
//!         fruit: "Plum",
//!         cost: 0.15,
//!         in_stock: true,
//!     };
//!
//!     let item3 = Produce {
//!         fruit: "Pear",
//!         cost: 0.75,
//!         in_stock: false,
//!     };
//!
//!     // Set the serialization location and headers.
//!     worksheet.serialize_headers(0, 0, &item1)?;
//!
//!     // Serialize the data.
//!     worksheet.serialize(&item1)?;
//!     worksheet.serialize(&item2)?;
//!     worksheet.serialize(&item3)?;
//!
//!     // Save the file.
//!     workbook.save("serialize.xlsx")?;
//!
//!     Ok(())
//! }
//! ```
//!
//! Output file:
//!
//! <img
//! src="https://rustxlsxwriter.github.io/images/worksheet_serialize_headers_skip1.png">
//!
//!
//!
//! The following example demonstrates skipping fields during serialization by
//! omitting them from the serialization headers. To do this we need to specify
//! custom headers. The output is the same as the image above.
//!
//! ```
//! # // This code is available in examples/doc_worksheet_serialize_headers_skip2.rs
//! #
//! # use rust_xlsxwriter::{CustomSerializeHeader, Workbook, XlsxError};
//! # use serde::Serialize;
//! #
//! # fn main() -> Result<(), XlsxError> {
//! #     let mut workbook = Workbook::new();
//! #
//! #     // Add a worksheet to the workbook.
//! #     let worksheet = workbook.add_worksheet();
//! #
//!     // Create a serializable test struct.
//!     #[derive(Serialize)]
//!     struct Produce {
//!         fruit: &'static str,
//!         cost: f64,
//!         in_stock: bool,
//!     }
//!
//!     // Create some data instances.
//!     let item1 = Produce {
//!         fruit: "Peach",
//!         cost: 1.05,
//!         in_stock: true,
//!     };
//!
//!     let item2 = Produce {
//!         fruit: "Plum",
//!         cost: 0.15,
//!         in_stock: true,
//!     };
//!
//!     let item3 = Produce {
//!         fruit: "Pear",
//!         cost: 0.75,
//!         in_stock: false,
//!     };
//!
//!     // Set up the custom headers.
//!     let custom_headers = [
//!         CustomSerializeHeader::new("fruit"),
//!         CustomSerializeHeader::new("cost"),
//!     ];
//!
//!     // Set the serialization location and custom headers.
//!     worksheet.serialize_headers_with_options(0, 0, "Produce", &custom_headers)?;
//!
//!     // Serialize the data.
//!     worksheet.serialize(&item1)?;
//!     worksheet.serialize(&item2)?;
//!     worksheet.serialize(&item3)?;
//! #
//! #     // Save the file.
//! #     workbook.save("serialize.xlsx")?;
//! #
//! #     Ok(())
//! # }
//! ```
//!
//! The following example is similar in setup to the previous example but
//! demonstrates skipping fields by explicitly skipping them in the custom
//! headers. This method should only be required in a few edge cases. The output
//! is the same as the image above.
//!
//! ```
//! # // This code is available in examples/doc_worksheet_serialize_headers_skip3.rs
//! #
//! # use rust_xlsxwriter::{CustomSerializeHeader, Workbook, XlsxError};
//! # use serde::Serialize;
//! #
//! # fn main() -> Result<(), XlsxError> {
//! #     let mut workbook = Workbook::new();
//! #
//! #     // Add a worksheet to the workbook.
//! #     let worksheet = workbook.add_worksheet();
//! #
//! #     // Create a serializable test struct. Note the serde attribute.
//! #     #[derive(Serialize)]
//! #     struct Produce {
//! #         fruit: &'static str,
//! #         cost: f64,
//! #         in_stock: bool,
//! #     }
//! #
//! #     // Create some data instances.
//! #     let item1 = Produce {
//! #         fruit: "Peach",
//! #         cost: 1.05,
//! #         in_stock: true,
//! #     };
//! #
//! #     let item2 = Produce {
//! #         fruit: "Plum",
//! #         cost: 0.15,
//! #         in_stock: true,
//! #     };
//! #
//! #     let item3 = Produce {
//! #         fruit: "Pear",
//! #         cost: 0.75,
//! #         in_stock: false,
//! #     };
//! #
//!     // Set up the custom headers.
//!     let custom_headers = [
//!         CustomSerializeHeader::new("fruit"),
//!         CustomSerializeHeader::new("cost"),
//!         CustomSerializeHeader::new("in_stock").skip(true),
//!     ];
//!
//!     // Set the serialization location and custom headers.
//!     worksheet.serialize_headers_with_options(0, 0, "Produce", &custom_headers)?;
//! #
//! #    // Serialize the data.
//! #    worksheet.serialize(&item1)?;
//! #    worksheet.serialize(&item2)?;
//! #    worksheet.serialize(&item3)?;
//! #
//! #     // Save the file.
//! #     workbook.save("serialize.xlsx")?;
//! #
//! #     Ok(())
//! # }
//! ```
//!
//!
//!
//! ## Setting serialization formatting
//!
//! Serialization will transfer your data to a worksheet but it won't format it
//! without a few additional steps.
//!
//! The most common requirement is to format the header/fields at the top of the
//! serialized data. The simplest way to do this is to use the
//! [`Worksheet::serialize_headers_with_format()`] method as shown in the
//! [Setting serialization headers](#setting-serialization-headers) section
//! above.
//!
//! The next most common requirement is to format values that are serialized
//! below the headers such as numeric data where you need to control the number
//! of decimal places or make it appear as a currency.
//!
//! The `serialize_headers_with_format()` method doesn't provide value
//! formatting functionality but as a workaround it is possible to set the
//! column format via [`Worksheet::set_column_format()`] as shown in the first
//! example below. This method has the drawback that it will add formatting to
//! any new values written to the column after serialization. In most cases this
//! may not be noticeable to the end user and may even be desirable.
//!
//! For finer control you can use
//! [`Worksheet::serialize_headers_with_options()`],
//! [`CustomSerializeHeader::set_header_format()`] and
//! [`CustomSerializeHeader::set_cell_format()`]as shown in the second example.
//!
//! ### Examples of formatting
//!
//! The following example demonstrates serializing instances of a Serde derived
//! data structure to a worksheet with header and cell formatting.
//!
//! ```
//! # // This code is available in examples/doc_worksheet_serialize_headers_format4.rs
//! #
//! # use rust_xlsxwriter::{Format, FormatBorder, Workbook, XlsxError};
//! # use serde::Serialize;
//! #
//! # fn main() -> Result<(), XlsxError> {
//! #     let mut workbook = Workbook::new();
//! #
//! #     // Add a worksheet to the workbook.
//! #     let worksheet = workbook.add_worksheet();
//! #
//!     // Add some formats to use with the serialization data.
//!     let header_format = Format::new()
//!         .set_bold()
//!         .set_border(FormatBorder::Thin)
//!         .set_background_color("C6EFCE");
//!
//!     let currency_format = Format::new().set_num_format("$0.00");
//!
//!     // Create a serializable test struct.
//!     #[derive(Serialize)]
//!     struct Produce {
//!         #[serde(rename = "Item")]
//!         fruit: &'static str,
//!
//!         #[serde(rename = "Price")]
//!         cost: f64,
//!     }
//!
//!     // Create some data instances.
//!     let item1 = Produce {
//!         fruit: "Peach",
//!         cost: 1.05,
//!     };
//!
//!     let item2 = Produce {
//!         fruit: "Plum",
//!         cost: 0.15,
//!     };
//!
//!     let item3 = Produce {
//!         fruit: "Pear",
//!         cost: 0.75,
//!     };
//!
//!     // Set a column format for the column. This is added to cell data that
//!     // doesn't have any other format so it doesn't affect the headers.
//!     worksheet.set_column_format(2, &currency_format)?;
//!
//!     // Set the serialization location and headers.
//!     worksheet.serialize_headers_with_format(1, 1, &item1, &header_format)?;
//!
//!     // Serialize the data.
//!     worksheet.serialize(&item1)?;
//!     worksheet.serialize(&item2)?;
//!     worksheet.serialize(&item3)?;
//! #
//! #     // Save the file.
//! #     workbook.save("serialize.xlsx")?;
//! #
//! #     Ok(())
//! # }
//! ```
//!
//! Output file:
//!
//! <img
//! src="https://rustxlsxwriter.github.io/images/worksheet_serialize_headers_custom.png">
//!
//!
//! The following example demonstrates header and cell formatting via custom
//! headers. This allows a little bit more precision on cell formatting.
//!
//! ```
//! # // This code is available in examples/doc_worksheet_serialize_headers_custom.rs
//! #
//! # use rust_xlsxwriter::{CustomSerializeHeader, Format, FormatBorder, Workbook, XlsxError};
//! # use serde::Serialize;
//! #
//! # fn main() -> Result<(), XlsxError> {
//! #     let mut workbook = Workbook::new();
//! #
//! #     // Add a worksheet to the workbook.
//! #     let worksheet = workbook.add_worksheet();
//! #
//! #     // Add some formats to use with the serialization data.
//! #     let header_format = Format::new()
//! #         .set_bold()
//! #         .set_border(FormatBorder::Thin)
//! #         .set_background_color("C6EFCE");
//! #
//! #     let currency_format = Format::new().set_num_format("$0.00");
//! #
//! #     // Create a serializable test struct.
//! #     #[derive(Serialize)]
//! #     struct Produce {
//! #         fruit: &'static str,
//! #         cost: f64,
//! #     }
//! #
//! #     // Create some data instances.
//! #     let item1 = Produce {
//! #         fruit: "Peach",
//! #         cost: 1.05,
//! #     };
//! #
//! #     let item2 = Produce {
//! #         fruit: "Plum",
//! #         cost: 0.15,
//! #     };
//! #
//! #     let item3 = Produce {
//! #         fruit: "Pear",
//! #         cost: 0.75,
//! #     };
//! #
//!     // Set up the start location and headers of the data to be serialized.
//!     let custom_headers = [
//!         CustomSerializeHeader::new("fruit")
//!             .rename("Item")
//!             .set_header_format(&header_format),
//!         CustomSerializeHeader::new("cost")
//!             .rename("Price")
//!             .set_header_format(&header_format)
//!             .set_cell_format(&currency_format),
//!     ];
//!
//!     worksheet.serialize_headers_with_options(1, 1, "Produce", &custom_headers)?;
//! #
//! #     // Serialize the data.
//! #     worksheet.serialize(&item1)?;
//! #     worksheet.serialize(&item2)?;
//! #     worksheet.serialize(&item3)?;
//! #
//! #     // Save the file.
//! #     workbook.save("serialize.xlsx")?;
//! #
//! #     Ok(())
//! # }
//! ```
//!
//!
//!
//!
//! ## Limitations of serializing to Excel
//!
//! The cell/grid format of Excel sets a physical limitation on what can be
//! serialized to a worksheet. Unlike other formats such as JSON or XML you
//! cannot serialize arbitrary nested data to Excel without making some some
//! concessions to either the format or the contents of the data. When
//! serializing data to Excel via `rust_xlsxwriter` it is best to consider what
//! that data will look like while designing your serialization.
//!
//! Another limitation is that currently you can only serialize structs or
//! struct values in compound containers such as vectors. Not all of the
//! supported types in the [Serde data model] make sense in the context of
//! Excel. In upcoming releases I will try to add support for additional types
//! where it makes sense. If you have a valid use case please open a GitHub
//! issue to discuss it with an example data structure.
//!
//! [Serde data model]: https://serde.rs/data-model.html
//!
//! Currently [`ExcelDateTime`](crate::ExcelDateTime) and Chrono date/times
//! aren't supported but they will be in the next release(s).
//!
//! Finally if you hit some serialization limitation using `rust_xlsxwriter`
//! remember that there are other non-serialization options available to use in
//! the standard [`Worksheet`] API to write scalar, vector and matrix data
//! types:
//!
//! - [`Worksheet::write()`]
//! - [`Worksheet::write_row()`]
//! - [`Worksheet::write_column()`]
//! - [`Worksheet::write_row_matrix()`]
//! - [`Worksheet::write_column_matrix()`]
//!
//! Magic is great but the direct approach can also work. Think of Terry
//! Pratchett's witches.
//!
#![warn(missing_docs)]

use std::collections::HashMap;
use std::fmt::Display;

use crate::{ColNum, Format, IntoExcelData, RowNum, Worksheet, XlsxError};
use serde::{ser, Serialize};

/// Implementation of the `serde::ser::Error` Trait to allow the use of a single
/// error type for serialization and `rust_xlsxwriter` errors.
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
impl ser::Error for XlsxError {
    fn custom<T: Display>(msg: T) -> Self {
        XlsxError::SerdeError(msg.to_string())
    }
}

// -----------------------------------------------------------------------
// Worksheet extensions to handle serialization.
// -----------------------------------------------------------------------

// The serialization Worksheet methods are added in this module to make it
// easier to isolate the feature specific code.
impl Worksheet {
    /// Write a Serde serializable struct to a worksheet.
    ///
    /// This method can be used to serialize [Serde](https://serde.rs) enabled
    /// data structure into cells in a worksheet.
    ///
    /// See [Working with Serde](crate::serializer#working-with-serde) for
    /// background details on how serialization works with `rust_xlsxwriter`.
    ///
    /// In order to serialize an instance of a data structure you must first
    /// define the fields/headers and worksheet location that the serialization
    /// will refer to. You can do this using one of the following methods:
    ///
    /// - [`Worksheet::serialize_headers()`]: The simplest most direct method. It
    ///   requires a argument of the type of struct that you wish to serialize. The
    ///   library uses this to infer the struct name and fields (via serialization).
    /// - [`Worksheet::serialize_headers_with_format()`]: This is similar to the
    ///   previous method but it allows you to add a cell format for the headers.
    /// - [`Worksheet::serialize_headers_with_options()`]: This requires the struct
    ///   type to be passed as a string and an array of [`CustomSerializeHeader`]
    ///   objects. However it gives the highest degree of control over the output.
    ///   This method also allows you to turn off the headers above the serialized
    ///   data.
    ///
    /// # Parameters
    ///
    /// * `data_structure` - A reference to a struct that implements the
    ///   [`serde::Serializer`] trait.
    ///
    /// # Errors
    ///
    /// * [`XlsxError::RowColumnLimitError`] - Row or column exceeds Excel's
    ///   worksheet limits.
    /// * [`XlsxError::MaxStringLengthExceeded`] - String exceeds Excel's limit
    ///   of 32,767 characters.
    /// * [`XlsxError::SerdeError`] - Errors encountered during the Serde
    ///   serialization.
    ///
    /// # Examples
    ///
    /// The following example demonstrates serializing instances of a Serde
    /// derived data structure to a worksheet.
    ///
    /// ```
    /// # // This code is available in examples/doc_worksheet_serialize.rs
    /// #
    /// use rust_xlsxwriter::{Workbook, XlsxError, Format};
    /// use serde::Serialize;
    ///
    /// fn main() -> Result<(), XlsxError> {
    ///     let mut workbook = Workbook::new();
    ///
    ///     // Add a worksheet to the workbook.
    ///     let worksheet = workbook.add_worksheet();
    ///
    ///     // Add a simple format for the headers.
    ///     let format = Format::new().set_bold();
    ///
    ///     // Create a serializable test struct.
    ///     #[derive(Serialize)]
    ///     #[serde(rename_all = "PascalCase")]
    ///     struct Produce {
    ///         fruit: &'static str,
    ///         cost: f64,
    ///     }
    ///
    ///     // Create some data instances.
    ///     let item1 = Produce {
    ///         fruit: "Peach",
    ///         cost: 1.05,
    ///     };
    ///     let item2 = Produce {
    ///         fruit: "Plum",
    ///         cost: 0.15,
    ///     };
    ///     let item3 = Produce {
    ///         fruit: "Pear",
    ///         cost: 0.75,
    ///     };
    ///
    ///     // Set up the start location and headers of the data to be serialized using
    ///     // any temporary or valid instance.
    ///     worksheet.serialize_headers_with_format(0, 0, &item1, &format)?;
    ///
    ///     // Serialize the data.
    ///     worksheet.serialize(&item1)?;
    ///     worksheet.serialize(&item2)?;
    ///     worksheet.serialize(&item3)?;
    ///
    ///     // Save the file.
    ///     workbook.save("serialize.xlsx")?;
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    /// Output file:
    ///
    /// <img
    /// src="https://rustxlsxwriter.github.io/images/worksheet_serialize.png">
    ///
    #[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
    pub fn serialize<T>(&mut self, data_structure: &T) -> Result<&mut Worksheet, XlsxError>
    where
        T: Serialize,
    {
        self.serialize_data_structure(data_structure)?;

        Ok(self)
    }

    /// Write the location and headers for data serialization.
    ///
    /// The [`Worksheet::serialize()`] method, above, serializes Serde derived
    /// structs to worksheet cells. However, before you serialize the data you
    /// need to set the position in the worksheet where the headers will be
    /// written and where serialized data will be written.
    ///
    /// See [Setting serialization
    /// headers](crate::serializer#setting-serialization-headers) for
    /// more information.
    ///
    /// # Parameters
    ///
    /// * `row` - The zero indexed row number.
    /// * `col` - The zero indexed column number.
    /// * `data_structure` - A reference to a struct that implements the
    ///   [`serde::Serializer`] trait.
    ///
    /// # Errors
    ///
    /// * [`XlsxError::RowColumnLimitError`] - Row or column exceeds Excel's
    ///   worksheet limits.
    /// * [`XlsxError::MaxStringLengthExceeded`] - String exceeds Excel's limit
    ///   of 32,767 characters.
    /// * [`XlsxError::SerdeError`] - Errors encountered during the Serde
    ///   serialization.
    ///
    /// # Examples
    ///
    /// The following example demonstrates serializing instances of a Serde
    /// derived data structure to a worksheet.
    ///
    /// ```
    /// # // This code is available in examples/doc_worksheet_serialize_headers1.rs
    /// #
    /// # use rust_xlsxwriter::{Workbook, XlsxError};
    /// # use serde::Serialize;
    /// #
    /// # fn main() -> Result<(), XlsxError> {
    /// #     let mut workbook = Workbook::new();
    /// #
    /// #     // Add a worksheet to the workbook.
    /// #     let worksheet = workbook.add_worksheet();
    /// #
    ///     // Create a serializable test struct.
    ///     #[derive(Serialize)]
    ///     #[serde(rename_all = "PascalCase")]
    ///     struct Produce {
    ///         fruit: &'static str,
    ///         cost: f64,
    ///     }
    ///
    ///     // Create some data instances.
    ///     let item1 = Produce {
    ///         fruit: "Peach",
    ///         cost: 1.05,
    ///     };
    ///     let item2 = Produce {
    ///         fruit: "Plum",
    ///         cost: 0.15,
    ///     };
    ///     let item3 = Produce {
    ///         fruit: "Pear",
    ///         cost: 0.75,
    ///     };
    ///
    ///     // Set up the start location and headers of the data to be serialized using
    ///     // any temporary or valid instance.
    ///     worksheet.serialize_headers(0, 0, &item1)?;
    ///
    ///     // Serialize the data.
    ///     worksheet.serialize(&item1)?;
    ///     worksheet.serialize(&item2)?;
    ///     worksheet.serialize(&item3)?;
    /// #
    /// #     // Save the file.
    /// #     workbook.save("serialize.xlsx")?;
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// Output file:
    ///
    /// <img
    /// src="https://rustxlsxwriter.github.io/images/worksheet_serialize_headers1.png">
    ///
    /// This example demonstrates starting the serialization in a different
    /// position.
    ///
    /// ```
    /// # // This code is available in examples/doc_worksheet_serialize_headers2.rs
    /// #
    /// # use rust_xlsxwriter::{Workbook, XlsxError};
    /// # use serde::Serialize;
    /// #
    /// # fn main() -> Result<(), XlsxError> {
    /// #     let mut workbook = Workbook::new();
    /// #
    /// #     // Add a worksheet to the workbook.
    /// #     let worksheet = workbook.add_worksheet();
    /// #
    /// #     // Create a serializable test struct.
    /// #     #[derive(Serialize)]
    /// #     #[serde(rename_all = "PascalCase")]
    /// #     struct Produce {
    /// #         fruit: &'static str,
    /// #         cost: f64,
    /// #     }
    /// #
    /// #     // Create some data instances.
    /// #     let item1 = Produce {
    /// #         fruit: "Peach",
    /// #         cost: 1.05,
    /// #     };
    /// #     let item2 = Produce {
    /// #         fruit: "Plum",
    /// #         cost: 0.15,
    /// #     };
    /// #     let item3 = Produce {
    /// #         fruit: "Pear",
    /// #         cost: 0.75,
    /// #     };
    /// #
    ///     // Set up the start location and headers of the data to be serialized using
    ///     // any temporary or valid instance.
    ///     worksheet.serialize_headers(1, 2, &item1)?;
    ///
    ///     // Serialize the data.
    ///     worksheet.serialize(&item1)?;
    ///     worksheet.serialize(&item2)?;
    ///     worksheet.serialize(&item3)?;
    /// #
    /// #     // Save the file.
    /// #     workbook.save("serialize.xlsx")?;
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// Output file:
    ///
    /// <img
    /// src="https://rustxlsxwriter.github.io/images/worksheet_serialize_headers2.png">
    ///
    #[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
    pub fn serialize_headers<T>(
        &mut self,
        row: RowNum,
        col: ColNum,
        data_structure: &T,
    ) -> Result<&mut Worksheet, XlsxError>
    where
        T: Serialize,
    {
        self.serialize_headers_with_format(row, col, data_structure, &Format::default())
    }

    /// Write the location and headers for data serialization, with formatting.
    ///
    /// The [`Worksheet::serialize()`] method, above, serializes Serde derived
    /// structs to worksheet cells. However, before you serialize the data you
    /// need to set the position in the worksheet where the headers will be
    /// written and where serialized data will be written. This method also
    /// allows you to set the format for the headers.
    ///
    /// See [Setting serialization
    /// headers](crate::serializer#setting-serialization-headers) for more
    /// information.
    ///
    /// # Parameters
    ///
    /// * `row` - The zero indexed row number.
    /// * `col` - The zero indexed column number.
    /// * `data_structure` - A reference to a struct that implements the
    ///   [`serde::Serializer`] trait.
    /// * `format` - The [`Format`] property for the cell.
    ///
    /// # Errors
    ///
    /// * [`XlsxError::RowColumnLimitError`] - Row or column exceeds Excel's
    ///   worksheet limits.
    /// * [`XlsxError::MaxStringLengthExceeded`] - String exceeds Excel's limit
    ///   of 32,767 characters.
    /// * [`XlsxError::SerdeError`] - Errors encountered during the Serde
    ///   serialization.
    ///
    /// # Examples
    ///
    /// The following example demonstrates serializing instances of a Serde
    /// derived data structure to a worksheet.
    ///
    /// ```
    /// # // This code is available in examples/doc_worksheet_serialize.rs
    /// #
    /// # use rust_xlsxwriter::{Workbook, XlsxError, Format};
    /// # use serde::Serialize;
    /// #
    /// # fn main() -> Result<(), XlsxError> {
    /// #     let mut workbook = Workbook::new();
    /// #
    /// #     // Add a worksheet to the workbook.
    /// #     let worksheet = workbook.add_worksheet();
    /// #
    ///     // Add a simple format for the headers.
    ///     let format = Format::new().set_bold();
    ///
    ///     // Create a serializable test struct.
    ///     #[derive(Serialize)]
    ///     #[serde(rename_all = "PascalCase")]
    ///     struct Produce {
    ///         fruit: &'static str,
    ///         cost: f64,
    ///     }
    ///
    ///     // Create some data instances.
    ///     let item1 = Produce {
    ///         fruit: "Peach",
    ///         cost: 1.05,
    ///     };
    ///     let item2 = Produce {
    ///         fruit: "Plum",
    ///         cost: 0.15,
    ///     };
    ///     let item3 = Produce {
    ///         fruit: "Pear",
    ///         cost: 0.75,
    ///     };
    ///
    ///     // Set up the start location and headers of the data to be serialized using
    ///     // any temporary or valid instance.
    ///     worksheet.serialize_headers_with_format(0, 0, &item1, &format)?;
    ///
    ///     // Serialize the data.
    ///     worksheet.serialize(&item1)?;
    ///     worksheet.serialize(&item2)?;
    ///     worksheet.serialize(&item3)?;
    /// #
    /// #     // Save the file.
    /// #     workbook.save("serialize.xlsx")?;
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// Output file:
    ///
    /// <img
    /// src="https://rustxlsxwriter.github.io/images/worksheet_serialize.png">
    ///
    #[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
    pub fn serialize_headers_with_format<T>(
        &mut self,
        row: RowNum,
        col: ColNum,
        data_structure: &T,
        format: &Format,
    ) -> Result<&mut Worksheet, XlsxError>
    where
        T: Serialize,
    {
        // Serialize the struct to determine the type name and the fields.
        let mut headers = SerializerHeader {
            struct_name: String::new(),
            field_names: vec![],
        };
        data_structure.serialize(&mut headers)?;

        // Convert the field names to custom header structs.
        let custom_headers: Vec<CustomSerializeHeader> = headers
            .field_names
            .iter()
            .map(|name| CustomSerializeHeader::new_with_format(name, format))
            .collect();

        self.serialize_headers_with_options(row, col, headers.struct_name, &custom_headers)
    }

    /// Write the location and headers for data serialization, with additional
    /// options.
    ///
    /// The [`Worksheet::serialize()`] and
    /// [`Worksheet::serialize_headers_with_format()`] methods, above, set the
    /// serialization headers and location via an instance of the structure to
    /// be serialized. This will work for the majority of use cases, and for
    /// other cases you can adjust the output by using Serde Container or Field
    /// [Attributes]. See [Working with Serde](#working-with-serde).
    ///
    /// [Attributes]: https://serde.rs/attributes.html
    ///
    /// If these methods don't give you the output or flexibility you require
    /// you can use the `serialize_headers_with_options()` method with
    /// [`CustomSerializeHeader`] options. This allows you to reorder, rename,
    /// format or skip headers and also define formatting for field values.
    ///
    /// See [`CustomSerializeHeader`] for additional information and examples.
    ///
    /// # Parameters
    ///
    /// * `row` - The zero indexed row number.
    /// * `col` - The zero indexed column number.
    /// * `struct_name` - The type name for the target struct, as a string.
    /// * `custom_headers` - An array of [`CustomSerializeHeader`] values.
    ///
    /// # Errors
    ///
    /// * [`XlsxError::RowColumnLimitError`] - Row or column exceeds Excel's
    ///   worksheet limits.
    /// * [`XlsxError::MaxStringLengthExceeded`] - String exceeds Excel's limit
    ///   of 32,767 characters.
    /// * [`XlsxError::SerdeError`] - Errors encountered during the Serde
    ///   serialization.
    /// # Examples
    ///
    /// The following example demonstrates serializing instances of a Serde
    /// derived data structure to a worksheet.
    ///
    /// ```
    /// # // This code is available in examples/doc_worksheet_serialize_headers_with_options.rs
    /// #
    /// # use rust_xlsxwriter::{CustomSerializeHeader, Format, Workbook, XlsxError};
    /// # use serde::Serialize;
    /// #
    /// # fn main() -> Result<(), XlsxError> {
    /// #     let mut workbook = Workbook::new();
    /// #
    /// #     // Add a worksheet to the workbook.
    /// #     let worksheet = workbook.add_worksheet();
    /// #
    ///     // Add some formats to use with the serialization data.
    ///     let bold = Format::new().set_bold();
    ///     let currency = Format::new().set_num_format("$0.00");
    ///
    ///     // Create a serializable test struct.
    ///     #[derive(Serialize)]
    ///     struct Produce {
    ///         fruit: &'static str,
    ///         cost: f64,
    ///     }
    ///
    ///     // Create some data instances.
    ///     let items = [
    ///         Produce {
    ///             fruit: "Peach",
    ///             cost: 1.05,
    ///         },
    ///         Produce {
    ///             fruit: "Plum",
    ///             cost: 0.15,
    ///         },
    ///         Produce {
    ///             fruit: "Pear",
    ///             cost: 0.75,
    ///         },
    ///     ];
    ///
    ///     // Set up the start location and headers of the data to be serialized using
    ///     // custom headers.
    ///     let custom_headers = [
    ///         CustomSerializeHeader::new("fruit")
    ///             .rename("Fruit")
    ///             .set_header_format(&bold),
    ///         CustomSerializeHeader::new("cost")
    ///             .rename("Price")
    ///             .set_header_format(&bold)
    ///             .set_cell_format(&currency),
    ///     ];
    ///
    ///     worksheet.serialize_headers_with_options(0, 0, "Produce", &custom_headers)?;
    ///
    ///     // Serialize the data.
    ///     worksheet.serialize(&items)?;
    /// #
    /// #     // Save the file.
    /// #     workbook.save("serialize.xlsx")?;
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// Output file:
    ///
    /// <img
    /// src="https://rustxlsxwriter.github.io/images/worksheet_serialize_headers_with_options.png">
    ///
    #[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
    pub fn serialize_headers_with_options(
        &mut self,
        row: RowNum,
        col: ColNum,
        struct_name: impl Into<String>,
        custom_headers: &[CustomSerializeHeader],
    ) -> Result<&mut Worksheet, XlsxError> {
        // Check row and columns are in the allowed range.
        if !self.check_dimensions_only(row, col) {
            return Err(XlsxError::RowColumnLimitError);
        }

        // Check for empty struct name.
        let struct_name = struct_name.into();
        if struct_name.is_empty() {
            return Err(XlsxError::ParameterError(
                "struct_name parameter cannot be blank".to_string(),
            ));
        }

        // Check if the headers should be hidden.
        let hidden_headers = custom_headers.iter().any(|h| h.hide_headers);

        let col_initial = col;
        for (col_offset, custom_header) in custom_headers.iter().enumerate() {
            if custom_header.skip {
                continue;
            }

            let col = col_initial + col_offset as u16;

            let mut serializer_header = custom_header.clone();
            serializer_header.row = row;
            serializer_header.col = col;

            if !hidden_headers {
                match &serializer_header.header_format {
                    Some(format) => {
                        self.write_with_format(row, col, &serializer_header.header_name, format)?
                    }
                    None => self.write(row, col, &serializer_header.header_name)?,
                };

                serializer_header.row += 1;
            }

            self.serializer_state.headers.insert(
                (struct_name.clone(), (custom_header.field_name.clone())),
                serializer_header,
            );
        }

        Ok(self)
    }

    // Serialize the parent data structure to the worksheet.
    fn serialize_data_structure<T>(&mut self, data_structure: &T) -> Result<(), XlsxError>
    where
        T: Serialize,
    {
        data_structure.serialize(self)?;
        Ok(())
    }

    // Serialize individual data items to a worksheet cell.
    fn serialize_to_worksheet_cell(&mut self, data: impl IntoExcelData) -> Result<(), XlsxError> {
        if !self.serializer_state.is_known_field() {
            return Ok(());
        }

        let row = self.serializer_state.current_row;
        let col = self.serializer_state.current_col;

        match &self.serializer_state.cell_format.clone() {
            Some(format) => self.write_with_format(row, col, data, format)?,
            None => self.write(row, col, data)?,
        };

        Ok(())
    }
}

// -----------------------------------------------------------------------
// SerializerState, a struct to maintain row/column state and other metadata
// between serialized writes. This avoids passing around cell location
// information in the serializer.
// -----------------------------------------------------------------------
pub(crate) struct SerializerState {
    headers: HashMap<(String, String), CustomSerializeHeader>,
    current_struct: String,
    current_field: String,
    current_col: ColNum,
    current_row: RowNum,
    cell_format: Option<Format>,
}

impl SerializerState {
    // Create a new SerializerState struct.
    pub(crate) fn new() -> SerializerState {
        SerializerState {
            headers: HashMap::new(),
            current_struct: String::new(),
            current_field: String::new(),
            current_col: 0,
            current_row: 0,
            cell_format: None,
        }
    }

    // Check if the current struct/field have been selected to be serialized by
    // the user. If it has then set the row/col values for the next write() call.
    fn is_known_field(&mut self) -> bool {
        let Some(field) = self
            .headers
            .get_mut(&(self.current_struct.clone(), self.current_field.clone()))
        else {
            return false;
        };

        // Set the "current" cell values used to write the serialized data.
        self.current_col = field.col;
        self.current_row = field.row;
        self.cell_format = field.cell_format.clone();

        // Increment the row number for the next worksheet.write().
        field.row += 1;

        true
    }
}

// -----------------------------------------------------------------------
// CustomSerializeHeader.
// -----------------------------------------------------------------------

/// The `CustomSerializeHeader` struct represents a custom serializer
/// field/header.
///
/// `CustomSerializeHeader` can be used to set column headers to map serialized
/// data to. It allows you to reorder, rename, format or skip headers and also
/// define formatting for field values.
///
/// It is used in conjunction with the
/// [`Worksheet::serialize_headers_with_options()`] method.
///
/// # Examples
///
/// The following example demonstrates serializing instances of a Serde derived
/// data structure to a worksheet with custom headers and cell formatting.
///
/// ```
/// # // This code is available in examples/doc_worksheet_serialize_headers_custom.rs
/// #
/// # use rust_xlsxwriter::{CustomSerializeHeader, Format, FormatBorder, Workbook, XlsxError};
/// # use serde::Serialize;
/// #
/// # fn main() -> Result<(), XlsxError> {
/// #     let mut workbook = Workbook::new();
/// #
/// #     // Add a worksheet to the workbook.
/// #     let worksheet = workbook.add_worksheet();
/// #
///     // Add some formats to use with the serialization data.
///     let header_format = Format::new()
///         .set_bold()
///         .set_border(FormatBorder::Thin)
///         .set_background_color("C6EFCE");
///
///     let currency_format = Format::new().set_num_format("$0.00");
///
///     // Create a serializable test struct.
///     #[derive(Serialize)]
///     struct Produce {
///         fruit: &'static str,
///         cost: f64,
///     }
///
///     // Create some data instances.
///     let item1 = Produce {
///         fruit: "Peach",
///         cost: 1.05,
///     };
///
///     let item2 = Produce {
///         fruit: "Plum",
///         cost: 0.15,
///     };
///
///     let item3 = Produce {
///         fruit: "Pear",
///         cost: 0.75,
///     };
///
///     // Set up the start location and headers of the data to be serialized.
///     let custom_headers = [
///         CustomSerializeHeader::new("fruit")
///             .rename("Item")
///             .set_header_format(&header_format),
///         CustomSerializeHeader::new("cost")
///             .rename("Price")
///             .set_header_format(&header_format)
///             .set_cell_format(&currency_format),
///     ];
///
///     worksheet.serialize_headers_with_options(1, 1, "Produce", &custom_headers)?;
///
///     // Serialize the data.
///     worksheet.serialize(&item1)?;
///     worksheet.serialize(&item2)?;
///     worksheet.serialize(&item3)?;
/// #
/// #     // Save the file.
/// #     workbook.save("serialize.xlsx")?;
/// #
/// #     Ok(())
/// # }
/// ```
///
/// Output file:
///
/// <img
/// src="https://rustxlsxwriter.github.io/images/worksheet_serialize_headers_custom.png">
///
#[derive(Clone)]
#[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
pub struct CustomSerializeHeader {
    field_name: String,
    header_name: String,
    header_format: Option<Format>,
    cell_format: Option<Format>,
    skip: bool,
    hide_headers: bool,
    row: RowNum,
    col: ColNum,
}

impl CustomSerializeHeader {
    /// Create a custom serialize header.
    ///
    /// Create a `CustomSerializeHeader` to be used with
    /// [`Worksheet::serialize_headers_with_options()`]. The name should
    /// correspond to a struct field being serialized.
    ///
    /// # Parameters
    ///
    /// * `name` - The name of the serialized field to map to the header.
    ///
    #[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
    pub fn new(field_name: impl Into<String>) -> CustomSerializeHeader {
        let field_name = field_name.into();
        let header_name = field_name.clone();

        CustomSerializeHeader {
            field_name,
            header_name,
            header_format: None,
            cell_format: None,
            skip: false,
            hide_headers: false,
            row: 0,
            col: 0,
        }
    }

    /// Rename the field name displayed a custom serialize header.
    ///
    /// The field names of structs are serialized as column headers at the top
    /// of serialized data. The default field names may not be the header names
    /// that you want displayed in Excel in which case you can use one of the
    /// two main methods to rename the fields/headers:
    ///
    /// 1. Rename the field during serialization using the Serde [field
    ///    attribute] `#[serde(rename = "name")` or the [container attribute]
    ///    `#[serde(rename_all = "...")]`.
    /// 2. Rename the header (not field) when setting up custom serialization
    ///    headers via [`Worksheet::serialize_headers_with_options()`] and
    ///    [`CustomSerializeHeader::rename()`].
    ///
    /// [field attribute]: https://serde.rs/field-attrs.html
    /// [container attribute]: https://serde.rs/container-attrs.html
    ///
    /// See [Renaming fields when
    /// serializing](crate::serializer#renaming-fields-when-serializing) for
    /// more details.
    ///
    /// # Parameters
    ///
    /// * `name` - A string like name to use as the header.
    ///
    /// # Examples
    ///
    /// The following example demonstrates renaming fields during serialization
    /// by specifying custom headers and renaming them there. You must still
    /// specify the actual field name to serialize in the `new()` constructor.
    ///
    ///
    /// ```
    /// # // This code is available in examples/doc_worksheet_serialize_headers_rename2.rs
    /// #
    /// # use rust_xlsxwriter::{CustomSerializeHeader, Workbook, XlsxError};
    /// # use serde::Serialize;
    /// #
    /// # fn main() -> Result<(), XlsxError> {
    /// #     let mut workbook = Workbook::new();
    /// #
    /// #     // Add a worksheet to the workbook.
    /// #     let worksheet = workbook.add_worksheet();
    /// #
    ///     // Create a serializable test struct.
    ///     #[derive(Serialize)]
    ///     struct Produce {
    ///         fruit: &'static str,
    ///         cost: f64,
    ///     }
    ///
    ///     // Create some data instances.
    ///     let item1 = Produce {
    ///         fruit: "Peach",
    ///         cost: 1.05,
    ///     };
    ///
    ///     let item2 = Produce {
    ///         fruit: "Plum",
    ///         cost: 0.15,
    ///     };
    ///
    ///     let item3 = Produce {
    ///         fruit: "Pear",
    ///         cost: 0.75,
    ///     };
    ///
    ///     // Set up the custom headers.
    ///     let custom_headers = [
    ///         CustomSerializeHeader::new("fruit").rename("Item"),
    ///         CustomSerializeHeader::new("cost").rename("Price"),
    ///     ];
    ///
    ///     // Set the serialization location and custom headers.
    ///     worksheet.serialize_headers_with_options(0, 0, "Produce", &custom_headers)?;
    ///
    ///     // Serialize the data.
    ///     worksheet.serialize(&item1)?;
    ///     worksheet.serialize(&item2)?;
    ///     worksheet.serialize(&item3)?;
    /// #
    /// #     // Save the file.
    /// #     workbook.save("serialize.xlsx")?;
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// Output file:
    ///
    /// <img
    /// src="https://rustxlsxwriter.github.io/images/worksheet_serialize_headers_rename1.png">
    ///
    #[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
    pub fn rename(mut self, name: impl Into<String>) -> CustomSerializeHeader {
        self.header_name = name.into();
        self
    }

    /// Set the header format for a custom serialize header.
    ///
    /// See [`Format`] for more information on formatting.
    ///
    /// # Parameters
    ///
    /// * `format` - The [`Format`] property for the header.
    ///
    /// # Examples
    ///
    /// The following example demonstrates formatting headers during
    /// serialization.
    ///
    /// ```
    /// # // This code is available in examples/doc_worksheet_serialize_headers_format2.rs
    /// #
    /// # use rust_xlsxwriter::{CustomSerializeHeader, Format, FormatBorder, Workbook, XlsxError};
    /// # use serde::Serialize;
    /// #
    /// # fn main() -> Result<(), XlsxError> {
    /// #     let mut workbook = Workbook::new();
    /// #
    /// #     // Add a worksheet to the workbook.
    /// #     let worksheet = workbook.add_worksheet();
    /// #
    ///     // Set a header format.
    ///     let header_format = Format::new()
    ///         .set_bold()
    ///         .set_border(FormatBorder::Thin)
    ///         .set_background_color("C6EFCE");
    ///
    /// #     // Create a serializable test struct.
    /// #     #[derive(Serialize)]
    /// #     struct Produce {
    /// #         fruit: &'static str,
    /// #         cost: f64,
    /// #     }
    /// #
    /// #     // Create some data instances.
    /// #     let item1 = Produce {
    /// #         fruit: "Peach",
    /// #         cost: 1.05,
    /// #     };
    /// #
    /// #     let item2 = Produce {
    /// #         fruit: "Plum",
    /// #         cost: 0.15,
    /// #     };
    /// #
    /// #     let item3 = Produce {
    /// #         fruit: "Pear",
    /// #         cost: 0.75,
    /// #     };
    /// #
    ///     // Set the serialization location and headers.
    ///     let custom_headers = [
    ///         CustomSerializeHeader::new("fruit").set_header_format(&header_format),
    ///         CustomSerializeHeader::new("cost").set_header_format(&header_format),
    ///     ];
    ///
    ///     worksheet.serialize_headers_with_options(1, 1, "Produce", &custom_headers)?;
    /// #
    /// #     // Serialize the data.
    /// #     worksheet.serialize(&item1)?;
    /// #     worksheet.serialize(&item2)?;
    /// #     worksheet.serialize(&item3)?;
    /// #
    /// #     // Save the file.
    /// #     workbook.save("serialize.xlsx")?;
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// Output file:
    ///
    /// <img
    /// src="https://rustxlsxwriter.github.io/images/worksheet_serialize_headers_format1.png">
    ///
    #[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
    pub fn set_header_format(mut self, format: &Format) -> CustomSerializeHeader {
        self.header_format = Some(format.clone());
        self
    }

    /// Set the cell format for values corresponding to a serialize
    /// header/field.
    ///
    /// It is sometimes necessary to set the cell format for the values of
    /// fields being serialized. The most common use case is to set the Excel
    /// number format for number fields.
    ///
    /// See [`Format`] and [`Format::set_num_format()`] for more information on
    /// formatting.
    ///
    /// # Parameters
    ///
    /// * `format` - The [`Format`] property for cells corresponding to the
    ///   field/header.
    ///
    /// # Examples
    ///
    /// The following example demonstrates formatting cells during
    /// serialization. Note the currency format for the `cost` cells.
    ///
    ///
    /// ```
    /// # // This code is available in examples/doc_worksheet_serialize_headers_format3.rs
    /// #
    /// # use rust_xlsxwriter::{CustomSerializeHeader, Format, Workbook, XlsxError};
    /// # use serde::Serialize;
    /// #
    /// # fn main() -> Result<(), XlsxError> {
    /// #     let mut workbook = Workbook::new();
    /// #
    /// #     // Add a worksheet to the workbook.
    /// #     let worksheet = workbook.add_worksheet();
    /// #
    ///     // Set a header format.
    ///     let cell_format = Format::new().set_num_format("$0.00");
    ///
    /// #     // Create a serializable test struct.
    /// #     #[derive(Serialize)]
    /// #     struct Produce {
    /// #         fruit: &'static str,
    /// #         cost: f64,
    /// #     }
    /// #
    /// #     // Create some data instances.
    /// #     let item1 = Produce {
    /// #         fruit: "Peach",
    /// #         cost: 1.05,
    /// #     };
    /// #
    /// #     let item2 = Produce {
    /// #         fruit: "Plum",
    /// #         cost: 0.15,
    /// #     };
    /// #
    /// #     let item3 = Produce {
    /// #         fruit: "Pear",
    /// #         cost: 0.75,
    /// #     };
    /// #
    ///     // Set the serialization location and headers.
    ///     let custom_headers = [
    ///         CustomSerializeHeader::new("fruit"),
    ///         CustomSerializeHeader::new("cost").set_cell_format(&cell_format),
    ///     ];
    ///
    ///     worksheet.serialize_headers_with_options(0, 0, "Produce", &custom_headers)?;
    /// #
    /// #     // Serialize the data.
    /// #     worksheet.serialize(&item1)?;
    /// #     worksheet.serialize(&item2)?;
    /// #     worksheet.serialize(&item3)?;
    /// #
    /// #     // Save the file.
    /// #     workbook.save("serialize.xlsx")?;
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// Output file:
    ///
    /// <img
    /// src="https://rustxlsxwriter.github.io/images/worksheet_serialize_headers_format3.png">
    ///
    ///
    #[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
    pub fn set_cell_format(mut self, format: &Format) -> CustomSerializeHeader {
        self.cell_format = Some(format.clone());
        self
    }

    /// Skip a field when serializing.
    ///
    /// When serializing a struct you may not want all of the fields to be
    /// serialized. For example the struct may contain internal fields that
    /// aren't of interest to the end user. There are several ways to skip
    /// fields:
    ///
    /// 1. Using the Serde [field attributes] `#[serde(skip)]` or
    ///    `#[serde(skip_serializing)]`.
    /// 2. Explicitly omitted the field when setting up custom serialization
    ///    headers via [`Worksheet::serialize_headers_with_options()`].
    /// 3. Marking the field as skippable via custom headers and the `skip()`
    ///    method.
    ///
    /// [field attributes]: https://serde.rs/field-attrs.html
    ///
    /// This method is only required in a few edge cases where you want to
    /// reserialize a struct to different parts of the worksheet with different
    /// combinations of fields displayed. Otherwise option 2 above is better.
    ///
    /// See [Skipping fields when
    /// serializing](crate::serializer#skipping-fields-when-serializing) for
    /// more details.
    ///
    /// # Parameters
    ///
    /// * `enable` - Turn the property on/off. It is off by default.
    ///
    /// # Examples
    ///
    /// The following example demonstrates skipping fields during serialization
    /// by explicitly skipping them via custom headers.
    ///
    /// ```
    /// # // This code is available in examples/doc_worksheet_serialize_headers_skip3.rs
    /// #
    /// # use rust_xlsxwriter::{CustomSerializeHeader, Workbook, XlsxError};
    /// # use serde::Serialize;
    /// #
    /// # fn main() -> Result<(), XlsxError> {
    /// #     let mut workbook = Workbook::new();
    /// #
    /// #     // Add a worksheet to the workbook.
    /// #     let worksheet = workbook.add_worksheet();
    /// #
    ///     // Create a serializable test struct.
    ///     #[derive(Serialize)]
    ///     struct Produce {
    ///         fruit: &'static str,
    ///         cost: f64,
    ///         in_stock: bool,
    ///     }
    ///
    ///     // Create some data instances.
    ///     let item1 = Produce {
    ///         fruit: "Peach",
    ///         cost: 1.05,
    ///         in_stock: true,
    ///     };
    ///
    ///     let item2 = Produce {
    ///         fruit: "Plum",
    ///         cost: 0.15,
    ///         in_stock: true,
    ///     };
    ///
    ///     let item3 = Produce {
    ///         fruit: "Pear",
    ///         cost: 0.75,
    ///         in_stock: false,
    ///     };
    ///
    ///     // Set up the custom headers.
    ///     let custom_headers = [
    ///         CustomSerializeHeader::new("fruit"),
    ///         CustomSerializeHeader::new("cost"),
    ///         CustomSerializeHeader::new("in_stock").skip(true),
    ///     ];
    ///
    ///     // Set the serialization location and custom headers.
    ///     worksheet.serialize_headers_with_options(0, 0, "Produce", &custom_headers)?;
    ///
    ///     // Serialize the data.
    ///     worksheet.serialize(&item1)?;
    ///     worksheet.serialize(&item2)?;
    ///     worksheet.serialize(&item3)?;
    /// #
    /// #     // Save the file.
    /// #     workbook.save("serialize.xlsx")?;
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// Output file:
    ///
    /// <img
    /// src="https://rustxlsxwriter.github.io/images/worksheet_serialize_headers_skip1.png">
    ///
    #[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
    pub fn skip(mut self, enable: bool) -> CustomSerializeHeader {
        self.skip = enable;
        self
    }

    /// Hide all the headers.
    ///
    /// If you want to serialize data without outputting the headers above the
    /// data you can set the `hide_headers` parameters to any of the custom
    /// headers.
    ///
    /// # Parameters
    ///
    /// * `enable` - Turn the property on/off. It is off by default.
    ///
    /// # Examples
    ///
    /// The following example demonstrates serializing data without outputting the
    /// headers above the data.
    ///
    ///
    /// ```
    /// # // This code is available in examples/doc_worksheet_serialize_headers_hide.rs
    /// #
    /// # use rust_xlsxwriter::{CustomSerializeHeader, Workbook, XlsxError};
    /// # use serde::Serialize;
    /// #
    /// # fn main() -> Result<(), XlsxError> {
    /// #     let mut workbook = Workbook::new();
    /// #
    /// #     // Add a worksheet to the workbook.
    /// #     let worksheet = workbook.add_worksheet();
    /// #
    ///     // Create a serializable test struct.
    ///     #[derive(Serialize)]
    ///     struct Produce {
    ///         fruit: &'static str,
    ///         cost: f64,
    ///     }
    ///
    ///     // Create some data instances.
    ///     let item1 = Produce {
    ///         fruit: "Peach",
    ///         cost: 1.05,
    ///     };
    ///
    ///     let item2 = Produce {
    ///         fruit: "Plum",
    ///         cost: 0.15,
    ///     };
    ///
    ///     let item3 = Produce {
    ///         fruit: "Pear",
    ///         cost: 0.75,
    ///     };
    ///
    ///     // Set up the custom headers.
    ///     let custom_headers = [
    ///         CustomSerializeHeader::new("fruit").hide_headers(true),
    ///         CustomSerializeHeader::new("cost"),
    ///     ];
    ///
    ///     // Set the serialization location and custom headers.
    ///     worksheet.serialize_headers_with_options(0, 0, "Produce", &custom_headers)?;
    ///
    ///     // Serialize the data.
    ///     worksheet.serialize(&item1)?;
    ///     worksheet.serialize(&item2)?;
    ///     worksheet.serialize(&item3)?;
    /// #
    /// #     // Save the file.
    /// #     workbook.save("serialize.xlsx")?;
    /// #
    /// #     Ok(())
    /// # }
    /// ```
    ///
    /// Output file:
    ///
    /// <img src="https://rustxlsxwriter.github.io/images/worksheet_serialize_headers_hide.png">
    ///
    #[cfg_attr(docsrs, doc(cfg(feature = "serde")))]
    pub fn hide_headers(mut self, enable: bool) -> CustomSerializeHeader {
        self.hide_headers = enable;
        self
    }

    // Internal constructor.
    fn new_with_format(field_name: impl Into<String>, format: &Format) -> CustomSerializeHeader {
        CustomSerializeHeader::new(field_name).set_header_format(format)
    }
}

// -----------------------------------------------------------------------
// Worksheet Serializer. This is the implementation of the Serializer trait to
// serialized a serde derived struct to an Excel worksheet.
// -----------------------------------------------------------------------
#[allow(unused_variables)]
impl<'a> ser::Serializer for &'a mut Worksheet {
    #[doc(hidden)]
    type Ok = ();
    #[doc(hidden)]
    type Error = XlsxError;
    #[doc(hidden)]
    type SerializeSeq = Self;
    #[doc(hidden)]
    type SerializeTuple = Self;
    #[doc(hidden)]
    type SerializeTupleStruct = Self;
    #[doc(hidden)]
    type SerializeTupleVariant = Self;
    #[doc(hidden)]
    type SerializeMap = Self;
    #[doc(hidden)]
    type SerializeStruct = Self;
    #[doc(hidden)]
    type SerializeStructVariant = Self;

    // Serialize all the default number types that fit into Excel's f64 type.
    #[doc(hidden)]
    fn serialize_bool(self, data: bool) -> Result<(), XlsxError> {
        self.serialize_to_worksheet_cell(data)
    }

    #[doc(hidden)]
    fn serialize_i8(self, data: i8) -> Result<(), XlsxError> {
        self.serialize_to_worksheet_cell(data)
    }

    #[doc(hidden)]
    fn serialize_u8(self, data: u8) -> Result<(), XlsxError> {
        self.serialize_to_worksheet_cell(data)
    }

    #[doc(hidden)]
    fn serialize_i16(self, data: i16) -> Result<(), XlsxError> {
        self.serialize_to_worksheet_cell(data)
    }

    #[doc(hidden)]
    fn serialize_u16(self, data: u16) -> Result<(), XlsxError> {
        self.serialize_to_worksheet_cell(data)
    }

    #[doc(hidden)]
    fn serialize_i32(self, data: i32) -> Result<(), XlsxError> {
        self.serialize_to_worksheet_cell(data)
    }

    #[doc(hidden)]
    fn serialize_u32(self, data: u32) -> Result<(), XlsxError> {
        self.serialize_to_worksheet_cell(data)
    }

    #[doc(hidden)]
    fn serialize_i64(self, data: i64) -> Result<(), XlsxError> {
        self.serialize_to_worksheet_cell(data)
    }

    #[doc(hidden)]
    fn serialize_u64(self, data: u64) -> Result<(), XlsxError> {
        self.serialize_to_worksheet_cell(data)
    }

    #[doc(hidden)]
    fn serialize_f32(self, data: f32) -> Result<(), XlsxError> {
        self.serialize_to_worksheet_cell(data)
    }

    #[doc(hidden)]
    fn serialize_f64(self, data: f64) -> Result<(), XlsxError> {
        self.serialize_to_worksheet_cell(data)
    }

    // Serialize strings types.
    #[doc(hidden)]
    fn serialize_str(self, data: &str) -> Result<(), XlsxError> {
        self.serialize_to_worksheet_cell(data)
    }

    // Excel doesn't have a character type. Serialize a char as a
    // single-character string.
    #[doc(hidden)]
    fn serialize_char(self, data: char) -> Result<(), XlsxError> {
        self.serialize_str(&data.to_string())
    }

    // Excel doesn't have a type equivalent to a byte array.
    #[doc(hidden)]
    fn serialize_bytes(self, data: &[u8]) -> Result<(), XlsxError> {
        Ok(())
    }

    // Serialize Some(T) values.
    #[doc(hidden)]
    fn serialize_some<T>(self, data: &T) -> Result<(), XlsxError>
    where
        T: ?Sized + Serialize,
    {
        data.serialize(self)
    }

    // Empty/None/Null values in Excel are ignored unless the cell has
    // formatting in which case they are handled as a "blank" cell. For all of
    // these cases we write an empty string and the worksheet writer methods
    // will handle it correctly based on context.

    #[doc(hidden)]
    fn serialize_none(self) -> Result<(), XlsxError> {
        self.serialize_str("")
    }

    #[doc(hidden)]
    fn serialize_unit(self) -> Result<(), XlsxError> {
        self.serialize_none()
    }

    #[doc(hidden)]
    fn serialize_unit_struct(self, _name: &'static str) -> Result<(), XlsxError> {
        self.serialize_none()
    }

    // Excel doesn't have an equivalent for the structure so we ignore it.
    #[doc(hidden)]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<(), XlsxError> {
        Ok(())
    }

    // Try to handle this as a single value.
    #[doc(hidden)]
    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<(), XlsxError>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    // Excel doesn't have an equivalent for the structure so we ignore it.
    #[doc(hidden)]
    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<(), XlsxError>
    where
        T: ?Sized + Serialize,
    {
        Ok(())
    }

    // Compound types.
    //
    // The only compound types that map into the Excel data model are
    // structs and array/vector types as fields in a struct.

    // Structs are the main primary data type used to map data structures into
    // Excel.
    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, XlsxError> {
        // Store the struct type name to check against user defined structs.
        self.serializer_state.current_struct = name.to_string();

        self.serialize_map(Some(len))
    }

    #[doc(hidden)]
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, XlsxError> {
        Ok(self)
    }

    // Not used.
    #[doc(hidden)]
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, XlsxError> {
        self.serialize_seq(Some(len))
    }

    // Not used.
    #[doc(hidden)]
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, XlsxError> {
        self.serialize_seq(Some(len))
    }

    // Not used.
    #[doc(hidden)]
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, XlsxError> {
        variant.serialize(&mut *self)?;
        Ok(self)
    }

    // The field/values of structs are treated as a map.
    #[doc(hidden)]
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, XlsxError> {
        Ok(self)
    }

    // Not used.
    #[doc(hidden)]
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, XlsxError> {
        variant.serialize(&mut *self)?;
        Ok(self)
    }
}

// The following impls deal with the serialization of compound types.
// Currently we only support/use SerializeStruct and SerializeSeq.

// Structs are the main sequence type used by `rust_xlsxwriter`.
#[doc(hidden)]
impl<'a> ser::SerializeStruct for &'a mut Worksheet {
    type Ok = ();
    type Error = XlsxError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), XlsxError>
    where
        T: ?Sized + Serialize,
    {
        // Store the struct field name to allow us to map to the correct
        // header/column.
        self.serializer_state.current_field = key.to_string();

        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), XlsxError> {
        Ok(())
    }
}

// We also serialize sequences to map vectors/arrays to Excel.
#[doc(hidden)]
impl<'a> ser::SerializeSeq for &'a mut Worksheet {
    type Ok = ();
    type Error = XlsxError;

    // Serialize a single element of the sequence.
    fn serialize_element<T>(&mut self, value: &T) -> Result<(), XlsxError>
    where
        T: ?Sized + Serialize,
    {
        let ret = value.serialize(&mut **self);

        // Increment the row number for each element of the sequence.
        self.serializer_state.current_row += 1;

        ret
    }

    fn end(self) -> Result<(), XlsxError> {
        Ok(())
    }
}

// Serialize tuple sequences.
#[doc(hidden)]
impl<'a> ser::SerializeTuple for &'a mut Worksheet {
    type Ok = ();
    type Error = XlsxError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), XlsxError>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), XlsxError> {
        Ok(())
    }
}

// Serialize tuple struct sequences.
#[doc(hidden)]
impl<'a> ser::SerializeTupleStruct for &'a mut Worksheet {
    type Ok = ();
    type Error = XlsxError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), XlsxError>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), XlsxError> {
        Ok(())
    }
}

// Serialize tuple variant sequences.
#[doc(hidden)]
impl<'a> ser::SerializeTupleVariant for &'a mut Worksheet {
    type Ok = ();
    type Error = XlsxError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), XlsxError>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), XlsxError> {
        Ok(())
    }
}

// Serialize tuple map sequences.
#[doc(hidden)]
impl<'a> ser::SerializeMap for &'a mut Worksheet {
    type Ok = ();
    type Error = XlsxError;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), XlsxError>
    where
        T: ?Sized + Serialize,
    {
        key.serialize(&mut **self)
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), XlsxError>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), XlsxError> {
        Ok(())
    }
}

// Serialize struct variant sequences.
#[doc(hidden)]
impl<'a> ser::SerializeStructVariant for &'a mut Worksheet {
    type Ok = ();
    type Error = XlsxError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), XlsxError>
    where
        T: ?Sized + Serialize,
    {
        key.serialize(&mut **self)?;
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), XlsxError> {
        Ok(())
    }
}

// -----------------------------------------------------------------------
// SerializerHeader. A struct used to store header/field name during
// serialization of the headers.
// -----------------------------------------------------------------------
struct SerializerHeader {
    struct_name: String,
    field_names: Vec<String>,
}

// -----------------------------------------------------------------------
// Header Serializer. This is the a simplified implementation of the Serializer
// trait to capture the headers/field names only.
// -----------------------------------------------------------------------
#[allow(unused_variables)]
impl<'a> ser::Serializer for &'a mut SerializerHeader {
    type Ok = ();
    type Error = XlsxError;
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    // Serialize strings types to capture the field names but ignore all other
    // types.
    fn serialize_str(self, data: &str) -> Result<(), XlsxError> {
        self.field_names.push(data.to_string());
        Ok(())
    }

    // Store the struct type/name to allow us to disambiguate structs.
    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, XlsxError> {
        self.struct_name = name.to_string();
        self.serialize_map(Some(len))
    }

    // Ignore all other primitive types.
    fn serialize_bool(self, data: bool) -> Result<(), XlsxError> {
        Ok(())
    }

    fn serialize_i8(self, data: i8) -> Result<(), XlsxError> {
        Ok(())
    }

    fn serialize_u8(self, data: u8) -> Result<(), XlsxError> {
        Ok(())
    }

    fn serialize_i16(self, data: i16) -> Result<(), XlsxError> {
        Ok(())
    }

    fn serialize_u16(self, data: u16) -> Result<(), XlsxError> {
        Ok(())
    }

    fn serialize_i32(self, data: i32) -> Result<(), XlsxError> {
        Ok(())
    }

    fn serialize_u32(self, data: u32) -> Result<(), XlsxError> {
        Ok(())
    }

    fn serialize_i64(self, data: i64) -> Result<(), XlsxError> {
        Ok(())
    }

    fn serialize_u64(self, data: u64) -> Result<(), XlsxError> {
        Ok(())
    }

    fn serialize_f32(self, data: f32) -> Result<(), XlsxError> {
        Ok(())
    }

    fn serialize_f64(self, data: f64) -> Result<(), XlsxError> {
        Ok(())
    }

    fn serialize_char(self, data: char) -> Result<(), XlsxError> {
        Ok(())
    }

    fn serialize_bytes(self, data: &[u8]) -> Result<(), XlsxError> {
        Ok(())
    }

    fn serialize_some<T>(self, data: &T) -> Result<(), XlsxError>
    where
        T: ?Sized + Serialize,
    {
        Ok(())
    }

    fn serialize_none(self) -> Result<(), XlsxError> {
        Ok(())
    }

    fn serialize_unit(self) -> Result<(), XlsxError> {
        Ok(())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<(), XlsxError> {
        Ok(())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<(), XlsxError> {
        Ok(())
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<(), XlsxError>
    where
        T: ?Sized + Serialize,
    {
        Ok(())
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<(), XlsxError>
    where
        T: ?Sized + Serialize,
    {
        Ok(())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, XlsxError> {
        Ok(self)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, XlsxError> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, XlsxError> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, XlsxError> {
        variant.serialize(&mut *self)?;
        Ok(self)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, XlsxError> {
        Ok(self)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, XlsxError> {
        variant.serialize(&mut *self)?;
        Ok(self)
    }
}

// We are only interested in Struct fields. Other compound types are ignored.
impl<'a> ser::SerializeStruct for &'a mut SerializerHeader {
    type Ok = ();
    type Error = XlsxError;

    fn serialize_field<T>(&mut self, key: &'static str, _value: &T) -> Result<(), XlsxError>
    where
        T: ?Sized + Serialize,
    {
        // Serialize the key/field name but ignore the values.
        key.serialize(&mut **self)
    }

    fn end(self) -> Result<(), XlsxError> {
        Ok(())
    }
}

impl<'a> ser::SerializeSeq for &'a mut SerializerHeader {
    type Ok = ();
    type Error = XlsxError;

    fn serialize_element<T>(&mut self, _value: &T) -> Result<(), XlsxError>
    where
        T: ?Sized + Serialize,
    {
        Ok(())
    }

    fn end(self) -> Result<(), XlsxError> {
        Ok(())
    }
}

impl<'a> ser::SerializeTuple for &'a mut SerializerHeader {
    type Ok = ();
    type Error = XlsxError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), XlsxError>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<(), XlsxError> {
        Ok(())
    }
}

impl<'a> ser::SerializeTupleStruct for &'a mut SerializerHeader {
    type Ok = ();
    type Error = XlsxError;

    fn serialize_field<T>(&mut self, _value: &T) -> Result<(), XlsxError>
    where
        T: ?Sized + Serialize,
    {
        Ok(())
    }

    fn end(self) -> Result<(), XlsxError> {
        Ok(())
    }
}

impl<'a> ser::SerializeTupleVariant for &'a mut SerializerHeader {
    type Ok = ();
    type Error = XlsxError;

    fn serialize_field<T>(&mut self, _value: &T) -> Result<(), XlsxError>
    where
        T: ?Sized + Serialize,
    {
        Ok(())
    }

    fn end(self) -> Result<(), XlsxError> {
        Ok(())
    }
}

impl<'a> ser::SerializeMap for &'a mut SerializerHeader {
    type Ok = ();
    type Error = XlsxError;

    fn serialize_key<T>(&mut self, _key: &T) -> Result<(), XlsxError>
    where
        T: ?Sized + Serialize,
    {
        Ok(())
    }

    fn serialize_value<T>(&mut self, _value: &T) -> Result<(), XlsxError>
    where
        T: ?Sized + Serialize,
    {
        Ok(())
    }

    fn end(self) -> Result<(), XlsxError> {
        Ok(())
    }
}

impl<'a> ser::SerializeStructVariant for &'a mut SerializerHeader {
    type Ok = ();
    type Error = XlsxError;

    fn serialize_field<T>(&mut self, _key: &'static str, _value: &T) -> Result<(), XlsxError>
    where
        T: ?Sized + Serialize,
    {
        Ok(())
    }

    fn end(self) -> Result<(), XlsxError> {
        Ok(())
    }
}
