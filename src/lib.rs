///! DynLib
///
/// Provides a relatively _safe_ interface to interact with windows dll
/// loading. This crate should only be used on windows platforms. If you
/// attempt to use it on non-windows platform you will likely get a
/// compiler error.
///
/// The _entry point_ to this crate is `LoadWinDynLib`. This allows the 
/// developer to set configuration flags. There are some details contained
/// within the type page. Most specifcally one should check which flags
/// are and are not allowed. There is some compatibility issues between
/// flags, as they'll modify _where_ files are located.
///
/// The loaded library does not have a built in drop flag. You are
/// responsible for dropping your loaded DLL's. Generally speaking
/// you have to be responsible for this. When the DLL is unloaded from
/// memory the symbols/functions you have loaded are invalidated,
/// attempts to call them will result in a memory fault, and likely
/// process termination.
///
/// An example project to generate a win dll [is
/// here](https://github.com/valarauca/dynlib/examples)]. A few cargo
/// directives are required, as well you must ensure you are building with
/// the `rust-msvc` compiler.


extern crate winapi;
extern crate kernel32;

use std::ffi::OsString;
use winapi::minwindef::HMODULE;
use winapi::c_void;
use kernel32::{LoadLibraryExA,GetProcAddress, FreeLibrary};
use std::io::Error;
use std::mem;

///An untyped pointer returned from an attempt to load a function.
///The develper will have to use `unsafe{ mem::transmute(VoidPtr)}` to recover
///the typing, and make this location executable.
pub type VoidPtr = *const c_void;


///Options for loading a Dynlib/Exe into windows.
///
///For full documentation see [The
///Docs](https://msdn.microsoft.com/en-us/library/windows/desktop/ms684179(v=vs.85).aspx)
///A cursory explaination is provided before each function.
#[derive(Copy,Clone)]
pub struct LoadWinDynLib {
    x: u32,
}
impl LoadWinDynLib {

    ///Create a new options value
    pub fn new() -> Self {
        LoadWinDynLib {
            x: 0
        }
    }

    ///Ignore Code Auth Levels
    ///
    ///This value tells the loader not to check App Locker or Software
    ///Software Restriction Polices for the DLL/EXE you are loading.
    ///
    ///If that DLL/EXE's dependencies DO have Code Auth Level restrictions you
    ///may will encounter an error
    pub fn ignore_code_authz_level<'a>(&'a mut self) -> &'a mut Self {
        self.x |= 0x00000010u32;
        self
    }

    ///Library As Data File
    ///
    ///The library will be loaded into virtual memory. The memory will not be
    ///flagged as executable, so calling functions from the library will
    ///result in errors.
    ///
    ///This is used in conjuction with other flags/functions not for dynamic
    ///linking.
    pub fn as_datafile<'a>( &'a mut self) -> &'a mut Self {
        self.x |=0x00000002u32;
        self
    }

    ///Library as Data File Exclusive
    ///
    ///Same as the above but this ensures you have WRITE permission to the
    ///virtual memory space the DLL/EXE is loaded into
    pub fn as_exclusive_datafile<'a>(&'a mut self) -> &'a mut Self {
        self.x |= 0x00000040u32;
        self
    }

    ///Library as image resource
    ///
    ///Loads a DLL as a mapped file with read only access into memory. This
    ///will by-pass any and all startup routines the DLL may invoke.
    pub fn as_image_resource<'a>(&'a mut self) -> &'a mut Self {
        self.x |= 0x00000020u32;
        self
    }

    ///Search Application Dir
    ///
    ///Allows the developer to not provide a path, but only a DLL name. The
    ///windows loader will search the local application directory for the 
    ///specified DLL name. 
    ///
    ///No other directories will be searched.
    pub fn search_application_dir<'a>(&'a mut self) -> &'a mut Self {
        self.x |= 0x00000200u32;
        self
    }

    ///Search Default Dirs
    ///
    ///Search ANYWHERE the DLL could be. Application Dir, System32, User Dir,
    ///AND environment path descripted loations.
    pub fn search_default_dirs<'a>(&'a mut self) -> &'a mut Self {
        self.x |= 0x00001000u32;
        self
    }

    ///Search DLL Load Dir
    ///
    ///This modifies the state of future searches. If you find a DLL with this
    ///flag then only the directory specified is searched. No others.
    pub fn search_dll_load_dir<'a>(&'a mut self) -> &'a mut Self {
        self.x |= 0x00000100u32;
        self
    }

    ///Search System32
    ///
    ///Search the %windows%\system32 directory for the DLL
    pub fn search_system32<'a>(&'a mut self) -> &'a mut Self {
        self.x |= 0x00000800u32;
        self
    }

    ///Search user dirs
    ///
    ///Search directors that are added with `AddDllDirectory` or the
    ///`SetDllDirectory` functions. 
    pub fn search_user_dirs<'a>(&'a mut self) -> &'a mut Self {
        self.x |= 0x00000400u32;
        self
    }

    ///Altered Search Path
    ///
    ///If you give me a full path I'm going to ignore it. You cannot combine
    ///this with any other search flag.
    pub fn altered_search_path<'a>(&'a mut self) -> &'a mut Self {
        self.x |= 0x00000008u32;
        self
    }

    ///Attempt the Load
    ///
    ///Give the system a path. This function will re-allocate the given
    ///string, and append a null terminator to the end.
    pub fn load(&self, path: &str)
    -> Result<DynLibWin,Error>
    {
        let mut os_str = OsString::from(path);
        os_str.push("\x00");
        unsafe{
            let (ptr,_,_): (*const i8,usize,usize) = mem::transmute(os_str);
            let handle = LoadLibraryExA(
                            ptr,
                            mem::transmute(0usize),
                            self.x);
            let val: usize = mem::transmute(handle);
            if val == 0usize {
                Err(Error::last_os_error())
            } else {
                Ok(DynLibWin{ handle: handle } )
            }
        }
    }
}
        

    

///DynLib
///
///The object that represents a dynamic library loaded into memory. This object
///is special to Microsoft Windows. You'll want to compile this program with
///the msvc tool chain. And when deploying ensure the Visual C++ runtime is
///deployed to the target machine.
///
///DynLib does not implement drop. So it will not be de-allocated. You must
///manually drop dynamic links.
pub struct DynLibWin {
    handle: HMODULE
}
impl DynLibWin {
    
    ///Load a function
    ///
    ///Ensure the names are mangled correctly. The null terminator will
    ///be appened in this function. The symbol you pass will be re-allocated
    ///within this function.
    ///
    ///One needs to cast the `VoidPtr` type to a a function. This is
    ///accomplished via the:
    ///
    ///`let func: extern "Rust" fn([YOUR ARGS] -> [YOUR RESULT] = unsafe{ mem::transmute([VOIDPTR]);`
    ///
    ///One should not the ABI is not burned in stone. You will have to
    ///change the value based on _what_ you are loading. The values
    ///supported by Rust are:
    ///
    ///`cdecl`, `stdcall`, `fastcall`, `vectorcall`, `aapcs`, ``win64`,
    ///`sysv64`, `Rust`, `C`, `system`, `rust-intrinsic`, `rust-all`, 
    ///and `platform-intrinsic`. Ensure your caps are correct.
    pub fn load_function(&self, symbol: &str)
    -> Result<VoidPtr,Error>
    {
        let mut os_str = OsString::from(symbol);
        os_str.push("\x00");
        unsafe {
            let (p,_,_): (*const i8, usize, usize) = mem::transmute(os_str);
            let ptr = GetProcAddress(
                            self.handle,
                            p);
            let val: usize = mem::transmute(ptr);
            if val == 0usize {
                Err(Error::last_os_error())
            } else {
                Ok(ptr)
            }
        }
    }

    ///Free the Library
    ///
    ///This will unload the DLL and invalidate all loaded functions. Use
    ///with care.
    pub fn free(self) -> Result<(),Error> {
        let handle = self.handle;
        let flag = unsafe{ FreeLibrary(handle) };
        if flag != 0 {
            Ok(())
        } else {
            Err(Error::last_os_error())
        }
    }
}



#[test]
fn test_it_all() {
    let p: &'static str = "C:\\users\\wlaeder\\documents\\rust_stuff\\dynlib\\test_dll.dll";
    let lib = match LoadWinDynLib::new()
        .load(p) {
        Ok(x) => x,
        Err(e) => {
            panic!("\n\n\nFailed to load library\n{:?}\n\n\n",e);
        }
    };
    println!("Loaded Library");
    let ptr = match lib.load_function("addition") {
        Ok(x) => x,
        Err(e) => {
            panic!("\n\n\nFailed to load symbol\n{:?}\n\n\n",e);
        }
    };
    println!("Found Symbol");
    let func: extern "Rust" fn(u64,u64)-> u64 = unsafe{ mem::transmute(ptr) };
    println!("Function Cast, calling...");
    assert_eq!(func(2,2), 4u64);
    println!("Success!");
}
