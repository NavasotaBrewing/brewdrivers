//! Many simpler Modbus devices don't provide registers/coils for reading and writing. 
//! Instead, they specify the bites to send over a serial connection. They still use Modbus RTU format,
//! specifically the bytestring layout.
//! 
//! See [this document for more information](https://modbus.org/docs/Modbus_over_serial_line_V1_02.pdf)
pub(crate) mod bytestring;
pub(crate) mod instrument;

pub(crate) use bytestring::Bytestring;
pub(crate) use instrument::SerialInstrument;