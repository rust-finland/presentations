use std::fmt;
use std::fs::File;
use wasmi::{
    Error as InterpreterError, HostError, ImportsBuilder,
    LittleEndianConvert, ModuleImportResolver,
    ModuleInstance, ModuleRef,
};

#[derive(Debug)]
pub enum Error {
    Interpreter(InterpreterError),
    Usage,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<InterpreterError> for Error {
    fn from(e: InterpreterError) -> Self {
        Error::Interpreter(e)
    }
}

impl HostError for Error {}

pub fn instantiate(path: &str, module_import_resolver: &ModuleImportResolver) -> Result<ModuleRef, Error> {
    let module = {
        use std::io::prelude::*;
        let mut file = File::open(path).unwrap();
        let mut wasm_buf = Vec::new();
        file.read_to_end(&mut wasm_buf).unwrap();
        wasmi::Module::from_buffer(&wasm_buf)?
    };

    let mut imports = ImportsBuilder::new();
    imports.push_resolver("env", module_import_resolver);

    let instance = ModuleInstance::new(&module, &imports)?.assert_no_start();

    Ok(instance)
}

#[derive(Debug, Eq, PartialEq)]
pub struct WasmString(pub String);

impl LittleEndianConvert for WasmString {
    fn from_little_endian(buffer: &[u8]) -> Result<Self, ::wasmi::ValueError> {
        use std::ffi::CStr;

        // Read until null
        let mut chunk: Vec<u8> = buffer.iter().take_while(|b| **b != 0).cloned().collect();
        // Push null to the end
        chunk.push(0);

        let s = CStr::from_bytes_with_nul(&chunk);
        let owned = s.unwrap().to_string_lossy().into_owned();
        Ok(WasmString(owned))
    }

    fn into_little_endian(self, _buffer: &mut [u8]) {
        unimplemented!();
    }
}
