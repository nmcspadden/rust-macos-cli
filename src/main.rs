use core_foundation_sys::{ base::*, string::*, propertylist::*, number::* };

use clap::Parser;
use libc::c_void;

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

/* Here's how this should generally work:

We get a value of a preference key from CFPreferencesCopyAppValue.
This value can generally be a couple of different things, determined
by enum PlistDataType. The primitive data types are int, bool, and 
string. However, ObjectiveC returns a CFPropertyListRef, which needs
to be matched to a specific data type before it can be used.

If it's a string, we turn that into a CString pointer and then turn
that into a Rust string via PlistDataType::Str(v).

If it's an integer, we pass in a C-style pointer and get the value from
CFNumber, and then cast that into an integer via PlistDataType::Int(v), which
then needs unwrap_int() called on it.


*/

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

/// Plists (and yaml) can contain only limited possible values
#[derive(Debug, Clone)]
pub enum PlistDataType {
    // ArrayOfDicts(Vec<HashMap<String, String>>),
    // ArrayOfStrs(Vec<String>),
    Bool(bool),
    // DictOfDicts(HashMap<String, PlistDataType>),
    // DictOfStrs(HashMap<String, String>),
    Str(String),
    Int(i64),
}

impl PlistDataType {
    pub fn unwrap_int(&self) -> i64 {
        match self {
            Self::Int(x) => *x,
            _ => panic!(),
        }
    }
}

fn figure_out_type(ret: CFPropertyListRef) -> Option<PlistDataType> {
    unsafe {
        if !ret.is_null() {
            let typ = CFGetTypeID(ret);
            if typ == CFStringGetTypeID() {
                // It's a string!
                let c_string = CFStringGetCStringPtr(ret as CFStringRef, kCFStringEncodingUTF8);
                if !c_string.is_null() {
                    let v = std::ffi::CStr::from_ptr(c_string).to_string_lossy().to_string();
                    CFRelease(ret as CFTypeRef);
                    return Some(PlistDataType::Str(v));
                } else {
                    CFRelease(ret as CFTypeRef);
                }
            } else if typ == CFNumberGetTypeID() {
                // It's a number!
                println!("It's a number!");
                // let p: *mut c_void = ptr::null_mut();
                // let _success: bool = CFNumberGetValue(ret as CFNumberRef, kCFNumberIntType, p);
                // println!("I'm guessing this crashes");
                // let my_num: i32 = *(p as *mut i32); // this crashes right now, hilarious
                // libc::free(p as *mut libc::c_void);
                let mut val: i64 = 0;
                if CFNumberGetValue(ret.cast(), kCFNumberIntType, &mut val as *mut i64 as *mut c_void) {
                    println!("Got a number!");
                }
                return Some(PlistDataType::Int(val));
            } else {
                CFRelease(ret as CFTypeRef);
            }
        }
        // println!("Result is empty");
        None
    }
}

fn get_int_pref_copyappvalue(key: &str, domain: &str) -> i64 {
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
            // CFRelease(ret as CFTypeRef);
            return figure_out_type(ret).unwrap().unwrap_int();
        }
        // println!("Result is empty");
        return 0
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
    let bool_result = get_pref_forced(&args.key, &args.domain) as Boolean;
    println!("Value is forced: Domain {} / {} = {}", args.domain, args.key, bool_result);

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

    // This currently crashes if you try to read a string as an int;
    // What needs to happen is it should read the value and then figure out how to cast it
    
    // let int_result = get_int_pref_copyappvalue(&args.key, &args.domain);
    // // display_str = str_result.as_deref().unwrap_or_default();
    // println!("CopyIntAppValue: Domain {} / {} = {}", args.domain, args.key, int_result);

}
