copy :: (src: *u8, dst: *u8, n: u64) #no_context {
    while n != 0 {
        <<src = <<dst;
        n -= 1;
        src += 1;
        dst += 1;
    }
}

// TODO(bryce): Allow other non-signed ints and cast?
copy :: inline (src: *$T, dst: *T, n: u64) #no_context {
    copy(cast(*u8)src, cast(*u8)dst, n * size_of(T));
}

clear :: (src: *u8, n: u64) #no_context {
    while n != 0 {
        <<src = 0;
        n -= 1;
        src += 1;
    }
}

clear :: inline (src: *$T, n: u64) #no_context {
    clear(cast(*u8)src, n * size_of(T));
}

#program_export "memcpy_impl" memcpy :: (dst: *u8, src: *u8, n: u64) #c_call {
    for 0..n-1 {
        <<dst = <<src;
        dst += 1;
        src += 1;
    }
} @StubImpl
#program_export "memset_impl" memset :: (dst: *u8, val: s32, n: u64) #c_call {
    for 0..n-1 {
        <<dst = xx val;
        dst += 1;
    }
} @StubImpl
