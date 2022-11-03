//! Many simpler Modbus devices don't provide registers/coils for reading and writing. 
//! Instead, they specify the bites to send over a serial connection. They still use Modbus RTU format,
//! specifically the bytestring layout.
//! 
//! See [this document for more information](https://modbus.org/docs/Modbus_over_serial_line_V1_02.pdf)
pub mod bytestring;
pub mod instrument;

pub use bytestring::Bytestring;
pub use instrument::SerialInstrument;