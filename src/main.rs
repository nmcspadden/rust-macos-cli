use core_foundation_sys::{ base::*, string::*, propertylist::* };

use clap::Parser;

// Ask for a key and domain
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Preference domain/applicationID
    #[arg(short, long)]
    domain: String,

    /// Key name
    #[arg(short, long)]
    key: String,
}

fn get_pref_forced(key: &str, domain: &str) -> Boolean {
    unsafe {
        extern "C" {
            pub fn CFPreferencesAppValueIsForced(key: CFStringRef, applicationID: CFStringRef) -> Boolean;
        }
        unsafe fn cfstr(v: &str) -> CFStringRef {
            CFStringCreateWithBytes(kCFAllocatorDefault, v.as_ptr(), v.len() as CFIndex, kCFStringEncodingUTF8, false as Boolean)
        }
        let cfstr_key = cfstr(key);
        let cfstr_app = cfstr(domain);
        let ret = CFPreferencesAppValueIsForced(cfstr_key, cfstr_app);
        CFRelease(cfstr_key as CFTypeRef);
        CFRelease(cfstr_app as CFTypeRef);
        return ret
    }
}

fn get_pref_copyvalue(key: &str, domain: &str, user: &str, host: &str) -> Option<String> {
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
        let user = cfstr(user);
        let host = cfstr(host);
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
        // println!("Result is empty");
        None
    }
}

fn get_pref_copyappvalue(key: &str, domain: &str) -> Option<String> {
    // Stolen from https://github.com/gyroflow/gyroflow/blob/c4241301ff427f9db58bebb0eb271a6bf2d9b9e1/src/core/util.rs#L152
    unsafe {
        extern "C" {
            pub fn CFPreferencesCopyAppValue(key: CFStringRef, applicationID: CFStringRef) -> CFPropertyListRef;
        }
        unsafe fn cfstr(v: &str) -> CFStringRef {
            CFStringCreateWithBytes(kCFAllocatorDefault, v.as_ptr(), v.len() as CFIndex, kCFStringEncodingUTF8, false as Boolean)
        }
        let cfstr_key = cfstr(key);
        let cfstr_app = cfstr(domain);
        let ret = CFPreferencesCopyAppValue(cfstr_key, cfstr_app);
        CFRelease(cfstr_key as CFTypeRef);
        CFRelease(cfstr_app as CFTypeRef);
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
        // println!("Result is empty");
        None
    }
}

fn main() {
    let args = Args::parse();
    /*         
        {'file': ('/var/root/Library/Preferences/ByHost/'
                  '%s.xxxx.plist' % domain),
         'user': kCFPreferencesCurrentUser,
         'host': kCFPreferencesCurrentHost
        },
        {'file': '/var/root/Library/Preferences/%s.plist' % domain,
         'user': kCFPreferencesCurrentUser,
         'host': kCFPreferencesAnyHost
        },
        {'file': ('/var/root/Library/Preferences/ByHost/'
                  '.GlobalPreferences.xxxx.plist'),
         'user': kCFPreferencesCurrentUser,
         'host': kCFPreferencesCurrentHost
        },
        {'file': '/var/root/Library/Preferences/.GlobalPreferences.plist',
         'user': kCFPreferencesCurrentUser,
         'host': kCFPreferencesAnyHost
        },
        {'file': '/Library/Preferences/%s.plist' % domain,
         'user': kCFPreferencesAnyUser,
         'host': kCFPreferencesCurrentHost
        }, 
    */
    let mut str_result = get_pref_copyvalue(&args.key, &args.domain, "kCFPreferencesCurrentUser", "kCFPreferencesAnyHost");
    let mut display_str = str_result.as_deref().unwrap_or_default();
    println!("CopyValue: Domain {} / {} = {}", args.domain, args.key, display_str);

    str_result = get_pref_copyvalue(&args.key, &args.domain, "kCFPreferencesCurrentUser", "kCFPreferencesCurrentHost");
    display_str = str_result.as_deref().unwrap_or_default();
    println!("CopyValue: Domain {} / {} = {}", args.domain, args.key, display_str);

    str_result = get_pref_copyvalue(&args.key, &args.domain, "kCFPreferencesAnyUser", "kCFPreferencesCurrentHost");
    display_str = str_result.as_deref().unwrap_or_default();
    println!("CopyValue: Domain {} / {} = {}", args.domain, args.key, display_str);

    str_result = get_pref_copyvalue(&args.key, &args.domain, "kCFPreferencesAnyUser", "kCFPreferencesAnyHost");
    display_str = str_result.as_deref().unwrap_or_default();
    println!("CopyValue: Domain {} / {} = {}", args.domain, args.key, display_str);

    str_result = get_pref_copyappvalue(&args.key, &args.domain);
    display_str = str_result.as_deref().unwrap_or_default();
    println!("CopyAppValue: Domain {} / {} = {}", args.domain, args.key, display_str);

    let bool_result = get_pref_forced(&args.key, &args.domain);
    println!("Value is forced: Domain {} / {} = {}", args.domain, args.key, bool_result);
}
