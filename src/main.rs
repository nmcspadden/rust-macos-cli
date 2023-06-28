use core_foundation_sys::preferences as CFPrefs;
use core_foundation_sys::string as CFString;
use core_foundation_sys::propertylist as CFPlist;
use core_foundation_sys::base as CFBase;

use std::ffi::{CString, CStr};

unsafe fn checkpref(key: &str, domain: &str) -> CFPlist::CFPropertyListRef {
    let application_id = convert_to_cfstring(key);
    let key = convert_to_cfstring(domain);
    return CFPrefs::CFPreferencesCopyAppValue(key, application_id);
}

unsafe fn convert_to_cfstring(input: &str) -> CFString::CFStringRef {
    let cstr_app_id = CString::new(input).unwrap(); // from a &str, creates a new allocation
    return CFString::CFStringCreateWithCString(CFBase::kCFAllocatorDefault, cstr_app_id.as_ptr(), CFString::kCFStringEncodingUTF8);
}

fn main() {
    println!("Fuck you");
    // We need to convert Rust strings to C Strings in order to pass them to CF functions
    // Thus, we must convert to a CString and then to a CFString
    let cstr_pref: CFPlist::CFPropertyListRef = unsafe { checkpref("SoftwareRepoURL", "ManagedInstalls") };
    // What kind of result did we get?
    // let cstr_result2 = cstr_pref.downcast::<CFString>().unwrap().to_string();
    // let result_typeid = unsafe { CFBase::CFGetTypeID(cstr_pref) };
    // println!("pre-crash?");
    // let cfstring_typieid = unsafe { CFString::CFStringGetTypeID() };
    // // now to get to figure out how to convert this back to a rust string
    // // what if we just assume it's a string?
    // if result_typeid != cfstring_typieid {
    //     panic!("This is not a string!");
    // }
    println!("Crash?");
    let cstr_result = unsafe { CStr::from_ptr(cstr_pref as *const _) };
    println!("Crash 2?");
    let str_result = cstr_result.to_string_lossy();
    println!("The SoftwareRepoURL for Munki is {str_result}");
}
