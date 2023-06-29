use core_foundation_sys::{ base::*, string::*, propertylist::* };

fn get_setting(key: &str, domain: &str) -> Option<String> {
    // Stolen from https://github.com/gyroflow/gyroflow/blob/c4241301ff427f9db58bebb0eb271a6bf2d9b9e1/src/core/util.rs#L152
    unsafe {
        extern "C" {
            pub fn CFPreferencesCopyValue(key: CFStringRef, applicationID: CFStringRef, userName: CFStringRef, hostName: CFStringRef) -> CFPropertyListRef;
        }
        unsafe fn cfstr(v: &str) -> CFStringRef {
            CFStringCreateWithBytes(kCFAllocatorDefault, v.as_ptr(), v.len() as CFIndex, kCFStringEncodingUTF8, false as Boolean)
        }
        let cfstr_key = cfstr(key);
        let cfstr_app = cfstr(domain);
        let user = cfstr("kCFPreferencesAnyUser");
        let host = cfstr("kCFPreferencesCurrentHost");
        let ret = CFPreferencesCopyValue(cfstr_key, cfstr_app, user, host);
        CFRelease(cfstr_key as CFTypeRef);
        CFRelease(cfstr_app as CFTypeRef);
        CFRelease(user as CFTypeRef);
        CFRelease(host as CFTypeRef);
        if !ret.is_null() {
            let typ = CFGetTypeID(ret);
            if typ == CFStringGetTypeID() {
                let c_string = CFStringGetCStringPtr(ret as CFStringRef, kCFStringEncodingUTF8);
                if !c_string.is_null() {
                    let v = std::ffi::CStr::from_ptr(c_string).to_string_lossy().to_string();
                    CFRelease(ret as CFTypeRef);
                    return Some(v);
                } else {
                    CFRelease(ret as CFTypeRef);
                }
            } else {
                CFRelease(ret as CFTypeRef);
            }
        }
        println!("Result is empty");
        None
    }
}

fn main() {
    let str_result = get_setting("SoftwareRepoURL", "ManagedInstalls");
    let display_str = str_result.as_deref().unwrap_or_default();
    println!("The SoftwareRepoURL for Munki is {display_str}");
}
