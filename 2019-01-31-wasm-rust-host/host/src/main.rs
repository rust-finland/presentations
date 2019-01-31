use std::collections::HashMap;
use std::env;
use wasmi::{
    Error as InterpreterError, Externals, FuncInstance, FuncRef, ModuleImportResolver, ModuleRef,
    RuntimeArgs, RuntimeValue, Signature, Trap, ValueType,
};

mod utils;

// Simple app state
#[derive(Debug, Eq, PartialEq)]
struct State(i32);

impl State {
    fn add(&mut self, a: i32) {
        self.0 += a;
    }
}

struct Runtime {
    states: HashMap<i32, State>,
}

impl Runtime {
    fn multiply(&self, a: i32, b: i32) -> i32 {
        a * b
    }
}

const MULT_FUNC_INDEX: usize = 0;
const STATE_ADD_FUNC_INDEX: usize = 1;

// Runtime implements Externals to execute host functions that are called from Wasm module.
// The arguments will be present if we correctly defined the types of the functions
// in RuntimeModuleImportResolver
impl Externals for Runtime {
    fn invoke_index(
        &mut self,
        index: usize,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, Trap> {
        match index {
            // multiply
            MULT_FUNC_INDEX => {
                let a: i32 = args.nth(0);
                let b: i32 = args.nth(1);
                let c = self.multiply(a, b);
                Ok(Some(RuntimeValue::I32(c)))
            }
            // state_add
            STATE_ADD_FUNC_INDEX => {
                let state_ptr: i32 = args.nth(0);
                let a: i32 = args.nth(1);
                self.states.get_mut(&state_ptr).unwrap().add(a);
                Ok(None)
            }
            _ => panic!("unknown function index"),
        }
    }
}

struct RuntimeModuleImportResolver;

// RuntimeModuleImportResolver implements import resolver to map host function calls.
// We have to correctly map the types of the host functions.
impl<'a> ModuleImportResolver for RuntimeModuleImportResolver {
    fn resolve_func(
        &self,
        field_name: &str,
        _signature: &Signature,
    ) -> Result<FuncRef, InterpreterError> {
        let func_ref = match field_name {
            "multiply" => FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32, ValueType::I32][..], Some(ValueType::I32)),
                MULT_FUNC_INDEX,
            ),
            "state_add" => FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32, ValueType::I32][..], None),
                STATE_ADD_FUNC_INDEX,
            ),
            _ => {
                return Err(InterpreterError::Function(format!(
                    "host module doesn't export function with name {}",
                    field_name
                )));
            }
        };
        Ok(func_ref)
    }
}

fn call(instance: &ModuleRef, func_name: &str, args: &[RuntimeValue], externals: &mut Runtime) -> Result<Option<RuntimeValue>, InterpreterError> {
    let res = instance.invoke_export(func_name, args, externals)?;
    println!("[{}] args: {:?} result: {:?}", func_name, args, res);
    Ok(res)
}

fn play(instance: ModuleRef) -> Result<(), utils::Error> {
    let mut runtime = Runtime {
        states: HashMap::new(),
    };

    // Import Wasm memory
    let internal_mem = instance
        .export_by_name("memory")
        .expect("Module expected to have 'memory' export")
        .as_memory()
        .cloned()
        .expect("'memory' export should be a memory");

    /////////////////////////////////////////////////////////////////////////////
    // Example 1: Add numbers
    //
    // We pass two numbers and expect a return value
    /////////////////////////////////////////////////////////////////////////////

    println!("\nExample 1: Add numbers");

    let res = call(&instance, "add", &[3.into(), 6.into()], &mut runtime)?;
    assert_eq!(res, Some(9.into()));

    /////////////////////////////////////////////////////////////////////////////
    // Example 2: Count string length
    //
    // We set a string to Wasm memory directly and pass a pointer and length
    // of string to the module.
    /////////////////////////////////////////////////////////////////////////////

    println!("\nExample 2: Count string length");

    let s = "hello world";

    // This is prone to errors
    // A better way to do it is to allocate memory from WebAssembly and
    // write to that chunk from host.
    internal_mem
        .set(0, s.as_bytes())
        .expect("set string to mem");

    let res = call(&instance, "count_str", &[0.into(), (s.len() as i32).into()], &mut runtime)?;
    assert_eq!(res, Some(11.into()));

    /////////////////////////////////////////////////////////////////////////////
    // Example 3: Return str from WebAssembly
    //
    // We can read a string from memory because we returned CString which is null
    // terminated string. Otherwise, we had to receive pointer to string and
    // length of the string.
    /////////////////////////////////////////////////////////////////////////////

    println!("\nExample 3: Return str from WebAssembly");

    let res = call(&instance, "return_str", &[], &mut runtime)?;
    let offset = if let RuntimeValue::I32(offset) = res.unwrap() {
        offset
    } else {
        panic!("Wrong return value");
    };

    // Read from Wasm memory directly
    let str = internal_mem.get_value::<utils::WasmString>(offset as u32);
    println!("[return_str] string {:?}", str);
    assert_eq!(
        str.unwrap(),
        utils::WasmString("hello from webassembly".into())
    );

    /////////////////////////////////////////////////////////////////////////////
    // Example 4: Calculate number by calling host function
    //
    // Wasm module calls host function to manipulate the number.
    /////////////////////////////////////////////////////////////////////////////

    println!("\nExample 4: Calculate number by calling host function");

    let res = call(&instance, "calc_host", &[3.into()], &mut runtime)?;
    assert_eq!(res, Some(18.into()));

    /////////////////////////////////////////////////////////////////////////////
    // Example 5: Call a method on complex object
    //
    // We hold a state in the map. When host function to manipulate the state
    // is called we find the state in the map and mutate it.
    //
    // Alternatively, we can store the state in the heap and find it using Box.
    //   Box::into_raw(Box::new(ptr))
    /////////////////////////////////////////////////////////////////////////////

    println!("\nExample 5: Call a method on complex object");

    let state_ptr = 1;
    runtime.states.insert(state_ptr, State(0));

    let res = call(&instance, "host_state_add", &[state_ptr.into()], &mut runtime)?;
    println!(
        "[host_state_add] state {:?}",
        runtime.states.get(&state_ptr)
    );
    assert_eq!(runtime.states.get(&state_ptr), Some(&State(5)));
    assert_eq!(res, None);

    // Done
    Ok(())
}

fn main() -> Result<(), utils::Error> {
    let args: Vec<_> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <module>", args[0]);
        return Err(utils::Error::Usage);
    }

    let instance =
        utils::instantiate(&args[1], &RuntimeModuleImportResolver).expect("Module to load");

    play(instance)
}
