use holochain_wasmer_common::*;

/// Allocate bytes that won't be dropped by the allocator.
/// Return the pointer to the leaked allocation so the host can write to it.
#[no_mangle]
#[inline(always)]
pub extern "C" fn __allocate(len: Len) -> GuestPtr {
    write_bytes(Vec::with_capacity(len as usize))
}

/// Free an allocation.
/// Needed because we leak memory every time we call `__allocate` and `write_bytes`.
#[no_mangle]
#[inline(always)]
pub extern "C" fn __deallocate(guest_ptr: GuestPtr, len: Len) {
    let _ = consume_bytes(guest_ptr, len);
}

/// Attempt to consume bytes from a known guest_ptr and len.
///
/// Consume in this context means take ownership of previously forgotten data.
///
/// This needs to work for bytes written into the guest from the host and for bytes written with
/// the write_bytes() function within the guest.
#[inline(always)]
pub fn consume_bytes<'a>(guest_ptr: GuestPtr, len: Len) -> Vec<u8> {
    // This must be a Vec and not only a slice because slices will fail to
    // deallocate memory properly when dropped.
    // Assumes length and capacity are the same which is true if __allocate is
    // used to allocate memory for the vector.
    unsafe { std::vec::Vec::from_raw_parts(guest_ptr as *mut u8, len as usize, len as usize) }
}

/// Attempt to write a slice of bytes.
///
/// This is identical to the following:
/// - host has some slice of bytes
/// - host calls __allocate with the slice length
/// - guest returns GuestPtr to the host
/// - host writes the bytes into the guest at GuestPtr location
/// - host hands the GuestPtr back to the guest
///
/// In this case everything happens within the guest and a GuestPtr is returned if successful.
///
/// This also leaks the written bytes, exactly like the above process.
///
/// This facilitates the guest handing a GuestPtr back to the host as the _return_ value of guest
/// functions so that the host can read the _output_ of guest logic from a pointer.
///
/// The host MUST ensure either __deallocate is called or the entire wasm memory is dropped.
#[inline(always)]
pub fn write_bytes(v: Vec<u8>) -> GuestPtr {
    v.leak().as_ptr() as GuestPtr
}
