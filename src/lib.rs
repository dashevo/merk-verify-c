use merk::execute_proof;
use std::slice;
use std::mem;
use std::ptr;

#[repr(C)]
pub struct Element {
    pub key_length: usize,
    pub key: *mut u8, //32 bytes
    pub exists: bool, //1 byte
    pub value_length: usize, //8 bytes
    pub value: *mut u8, //value_length bytes
}

#[repr(C)]
pub struct ExecuteProofResult {
    pub hash: *mut [u8; 32],  //32 bytes
    pub element_count: usize,  //8 bytes
    pub elements: *mut *mut Element, //sizeof(pointer)
}

fn vec_to_raw_pointer<T>(mut vec: Vec<T>) -> *mut T {
    // Take the pointer
    let pointer = vec.as_mut_ptr();

    // Release the ownership
    // mem::forget releases ownership without deallocating memory.
    // This essentially gives the ownership to the c caller. Rust needs to get
    // the pointer to the struct back in order to properly discard it later.
    mem::forget(vec);

    // Return the pointer
    pointer
}

#[no_mangle]
pub extern fn execute_proof_c(c_array: *const u8, length: usize) -> *mut ExecuteProofResult {
    let rust_array: &[u8] = unsafe {
        slice::from_raw_parts(c_array, length as usize)
    };

    let execute_proof_result = execute_proof(rust_array);

    match execute_proof_result {
        Err(_) => ptr::null_mut(),
        Ok((hash, map)) => {
            let elements: Vec<*mut Element> = map.all().map(|(key, (exists, value))| {
                let element = Element {
                    key_length: key.len(),
                    key: vec_to_raw_pointer(key.clone()),
                    exists: *exists,
                    value_length: value.len(),
                    value: vec_to_raw_pointer(value.clone())
                };

                Box::into_raw(Box::new(element))
            }).collect();

            let result = ExecuteProofResult {
                hash: Box::into_raw(Box::new(hash)),
                element_count: elements.len(),
                elements: vec_to_raw_pointer(elements),
            };

            Box::into_raw(Box::new(result))
        }
    }
}

#[no_mangle]
pub unsafe extern fn destroy_proof_c(proof_result: *mut ExecuteProofResult) {
    let result = Box::from_raw(proof_result);
    let _ = Box::from_raw(result.hash);
    let vec = Vec::from_raw_parts(result.elements, result.element_count, result.element_count);
    for &x in vec.iter() {
        let _ = Box::from_raw(x);
    }
}
