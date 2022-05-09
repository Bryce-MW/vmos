#import "Basic";
#import "Process";
#import "File";
#import "POSIX"; // File size of file I have not opened
#import "Compiler";
#import "Machine_X64";
#import "Hash_Table";

VERBOSE :: false;

bootloader_file :: "./build/bootloader";
kernel_file :: "./build/kernel";
kernel_stripped_file :: "./build/kernel-s";
image_file :: "./build/image";

build :: () {
    // DEBUG
    {
        using context.print_style.default_format_struct;
        draw_type_name = true;
        use_long_form_if_more_than_this_many_members = -1;
        use_newlines_if_long_form = true;
    }
    defer set_build_options_dc(.{do_output=false});

    w := compiler_create_workspace("Kernel");
    if !w {
        print("Workspace creation failed.\n");
        return;
    }
    target_options := get_build_options(w);

    args := target_options.compile_time_command_line;
    should_run := false;
    for args if it == "run" should_run = true;

    // Before build
    print("Starting before build\n");
    make_directory_if_it_does_not_exist("./build");


    {
        using target_options;
        stack_trace = false; // Consider if we want this?
        array_bounds_check = .OFF; // Consider these as well...
        cast_bounds_check = .OFF;
        math_bounds_check = .OFF;
        null_pointer_check = .OFF;
        output_path = "./build/";
        intermediate_path = "./build/";
        output_executable_name = "kernel";
        os_target = .NONE;
        {
            //using x86_Feature_Flag;
            //machine_options.x86_features.leaves = .[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
            //features := x86_Feature_Flag.[
            //    // Figure out how to turn on soft floats?
            //    TSC, APIC, CMOV, CLFLUSH, SSE2,  // !MMX !SSE !SSE2 // SEP? 
            //    CX16, MOVBE, POPCNT, RDRAND,     // !SSE3 !SSSE3 !SSE4_1 !SSE4_2 // XSAVE? OSXSAVE?
            //                                     // 
            //    RDTSCP, _64BIT_MODE,             // // SYSCALL_SYSRET? PDPE1GB?
            //    LAHF_SAHF, ABM, _3DNOW_PREFETCH, //
            //                                     // 
            //    BMI1, ENHANCED_REP, RDSEED, ADX  //
            //                                     // 
            //                                     // 
            //                                     // 
            //                                     // 
            //    // There may be other flags we want at some point
            //];
            //for features enable_feature(*machine_options.x86_features.leaves, it);
        }
        #if VERBOSE {
            print("CPU FLAGS:\n%\n", get_cpu_info());
            print("Enabled FLAGS:\n%\n", machine_options); // We may want to change this to disallow stuff like floats
        }
        use_custom_link_command = true;
        info_flags = .POLYMORPH_DEDUPLICATE;
        {
            using x64_options;
            enable_register_allocation = true;
        }
        {
            using llvm_options;
            merge_functions = true;
            code_gen_size_level = 3;
        }
        //additional_linker_arguments = .["-image-base=0"];
        llvm_options.target_system_triple = "x86_64-unknown-none-elf";
        /*
          "-Crelocation-model=pie",
          "-Ctarget-feature=+crt-static",
          "disable-redzone": true,
          "features": "-mmx,-sse,-sse2,-sse3,-ssse3,-sse4.1,-sse4.2,-3dnow,-3dnowa,-avx,-avx2,+soft-float",
        */
    }
    set_build_options(target_options, w);

    compiler_begin_intercept(w);
    add_build_file(tprint("%/kernel.jai", #filepath), w);
    message_loop(w);
    compiler_end_intercept(w);

    print("\nDone!\n\n");


    // After build
    print("Starting after build\n");
    if assemble() {
        if should_run {
            print("Running\n");
            run();
        }
    }

    print("Done\n");
}

#run build();

elsewhere: Table(string, string);

message_loop :: (w: Workspace) {

    while true {
        message := compiler_wait_for_message();
        if !message break;

        if message.kind == {
            case .FILE;
                message_file := cast(*Message_File) message;
                #if VERBOSE {
                    print("  Loading file '%'.\n", message_file.fully_pathed_filename);
                }
            
            case .IMPORT;
                import := cast(*Message_Import) message;
                #if VERBOSE {
                    print("  Importing module '%'.\n", import.module_name);
                }
            
            case .PHASE;
                phase := cast(*Message_Phase) message;
                #if VERBOSE {
                    print("--- Entering phase %. ---\n", phase.phase);
                }

                if phase.phase == {
                    case .POST_WRITE_EXECUTABLE;
                        print("\n");
                        if phase.executable_write_failed {
                            print("FAILED writing '%'! Linker exit code was %.\n", phase.executable_name, phase.linker_exit_code);
                        } else {
                            print("Success writing '%'! Linker exit code was %.\n", phase.executable_name, phase.linker_exit_code);
                        }
                    case .READY_FOR_CUSTOM_LINK_COMMAND;
                        print("\nRunning custom link command:\n");
                        #if VERBOSE {
                            print("Parms:\ncompiler_generated_object_files: %\nsupport_object_files: %\nsystem_libraries: %\nuser_libraries: %\n", phase.compiler_generated_object_files, phase.support_object_files, phase.system_libraries, phase.user_libraries);
                        }
                        command: [..] string;
                        array_add(*command, "ld.lld");
                        array_add(*command, "-flavor");
                        array_add(*command, "Gnu");
                        array_add(*command, "--eh-frame-hdr");
                        array_add(*command, "--color-diagnostics");
                        array_add(*command, "--pie");
                        array_add(*command, "-O");
                        array_add(*command, "3");
                        array_add(*command, "-z");
                        array_add(*command, "notext");
                        array_add(*command, "--lto-O3");
                        array_add(*command, "--discard-none");
                        array_add(*command, "-o");
                        array_add(*command, "./build/kernel");
                        for phase.compiler_generated_object_files array_add(*command, it);
                        for elsewhere array_add(*command, it);
                        array_add(*command, "-image-base=0");
                        for command print("% ", it);
                        print("\n");
                        os_run_command(..command);
                        compiler_custom_link_command_is_complete(w);
                    case;
                }
            
            case .TYPECHECKED;
                typechecked := cast(*Message_Typechecked) message;
                #if VERBOSE {
                    n := typechecked.all.count;
                    if n == 1 {
                        print("  1 thing has been typechecked.\n");
                    } else if n != 0 {
                        print("  % things have been typechecked.\n", n);
                    }
                }
                for typechecked.procedure_headers {
                    header := it.expression;
                    #if VERBOSE {
                        print("Func: % has flags: %\n", header.name, header.procedure_flags);
                    }
                    if header.procedure_flags & .ELSEWHERE {
                        if !table_find_pointer(*elsewhere, header.foreign_function_name) {
                            table_add(*elsewhere, header.foreign_function_name, tprint("--defsym=%1=%1_stub", header.foreign_function_name));
                            bstring := tprint("#program_export %_stub :: () {#asm{int3;}}", header.foreign_function_name);
                            #if VERBOSE {
                                print("%\n", bstring);
                            }
                            add_build_string(bstring, w);
                        }
                    } else {
                        for header.notes {
                            if it.text == "StubImpl" {
                                table_set(*elsewhere, header.name, tprint("--defsym=%1=%1_impl", header.name));
                                break;
                            }
                        }
                    }
                }

            case .ERROR;
                print("Error:\n%\n", <<message);

            case .COMPLETE;
                break;
            
            case;
                compiler_report(tprint("Unhandled message type %!\n", message.kind));
        }
    }
}

assemble :: () -> bool {
    os_run_command(
        "llvm-strip",
        "--strip-unneeded",
        "-o", kernel_stripped_file,
        kernel_file
    );

    k_stat: stat_t;
    if (stat(kernel_stripped_file, *k_stat)) {
        print("Couldn't find out the size of the stripped kernel!\n");
        return false;
    }
    k_size := k_stat.st_size;

    os_run_command(
        "nasm",
        "-f", "bin",
        "-g",
        "-w+orphan-labels",
        tprint("-dKERNEL_LENGTH=%", k_size),
        "-dELF=0",
        "-o", bootloader_file,
        "start.asm"
    );

    os_run_command(
        "nasm",
        "-f", "elf",
        "-g",
        "-F", "dwarf",
        "-w+orphan-labels",
        tprint("-dKERNEL_LENGTH=%", k_size),
        "-dELF=1",
        "-o", tprint("%.elf", bootloader_file),
        "start.asm"
    );

    _0, _1, image := os_run_command( // 2/3
        "cat",
        bootloader_file,
        kernel_stripped_file,
        capture_and_return_output=true
    );
    write_entire_file(image_file, image);
    return true;
}

run :: () {
    #if true {
        print("Starting QEMU...\n");
        os_run_command(
            "qemu-system-x86_64",
            "-machine", "q35,accel=tcg,vmport=off",
            "-m", "4G",
            "-s",
            "-cpu", "max",
            "-smp", "cpus=8",
            "-name", "vmos",
            "-device", "ioh3420,id=root_port1",
            "-drive", tprint("file=%,media=disk,if=none,format=raw,id=disk", image_file),
            "-device", "virtio-blk-pci,bus=root_port1,drive=disk,cyls=1,heads=1,secs=1",
            "-chardev", "vc,id=seabios", "-device", "isa-debugcon,iobase=0x402,chardev=seabios",
            "-monitor", "stdio",
            "-vga", "virtio"
        );
    } else {
        print("Printing QEMU...\n");
        print(#string END
qemu-system-x86_64 \
    -machine q35,accel=tcg,vmport=off \
    -m 4G \
    -s \
    -cpu max \
    -smp cpus=8 \
    -name vmos \
    -device ioh3420,id=root_port1 \
    -drive file=%,media=disk,if=none,format=raw,id=disk \
    -device virtio-blk-pci,bus=root_port1,drive=disk,cyls=1,heads=1,secs=1 \
    -chardev vc,id=seabios -device isa-debugcon,iobase=0x402,chardev=seabios \
    -monitor stdio \
    -vga virtio
END
        , image_file);
    }
}
