#load "mem_utils.jai";
#load "vga.jai";
//#load "interrupts.jai";

#load "sync.jai";

hello :: "Hello World!";

main :: () {
    clear_vga();

    if cast(u64) cast(*void) main < 0x7FFFFFFF { // Do we have an equivalent of isize::MAX?
        // NOTE(bryce): We are in a lower half kernel, i.e. the bootloader just called us
        write_string("Got lower-half control from bootloader\n");
        write_string(hello);
        write_string("\n");

        //create_glob_idt();
        //#asm {
        //    sti;
        //}

        // create_high_mem();
    } else {
        // NOTE(bryce): We are in a higher half kernel so we can now play with memory
        write_string("Got higher-half control from kernel\n");

        /*
            unsafe {
                create_glob_idt();
                sti();

                find_pcie();
            }

            println!("{}\nSomething else", HELLO);
        }
        */
    }
}

__system_entry_point :: (argc: s32, argv: **u8) -> s32 #c_call #runtime_support;

#program_export _start :: () #c_call {
    argc: s32 = 0;
    argv: [] *u8 = .[];
    __system_entry_point(argc, argv.data);
    while true {
        #asm {
            hlt;
        }
    }
}
