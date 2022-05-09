#program_export "pthread_mutex_lock_impl" pthread_mutex_lock :: (lock: *void) #c_call {
    // Currently does nothing
} @StubImpl
#program_export "pthread_mutex_unlock_impl" pthread_mutex_unlock :: (lock: *void) #c_call {
    // Currently does nothing
} @StubImpl

#program_export "pthread_mutexattr_init_impl" pthread_mutexattr_init :: (attr: *void) #c_call {
    // Currently does nothing
} @StubImpl

#program_export "pthread_mutexattr_settype_impl" pthread_mutexattr_settype :: (attr: *void, type: s32) #c_call {
    // Currently does nothing
} @StubImpl

#program_export "pthread_mutex_init_impl" pthread_mutex_init :: (mutex: *void) #c_call {
    // Currently does nothing
} @StubImpl
