use std::{ffi::c_void, marker::PhantomData};
use windows::{
    core::{Error, IntoParam, HRESULT, PCWSTR, PWSTR},
    Win32::{
        Foundation::{ERROR_FILE_NOT_FOUND, ERROR_MORE_DATA},
        System::Registry::{self, *},
    },
};

pub use Registry::HKEY_CURRENT_USER;
pub use Registry::HKEY_LOCAL_MACHINE;

#[derive(Debug)]
pub struct Key {
    handle: HKEY,
    access: REG_SAM_FLAGS,
}

impl Key {
    pub fn create<K, P>(key: K, path: P) -> Result<Self, Error>
    where
        K: IntoParam<HKEY>,
        P: IntoParam<PCWSTR>,
    {
        unsafe {
            const ACCESS: REG_SAM_FLAGS = KEY_ALL_ACCESS;
            let mut handle: HKEY = Default::default();

            RegCreateKeyW(key, path.into_param().abi(), &mut handle)?;

            Ok(Key {
                handle,
                access: ACCESS,
            })
        }
    }

    pub fn open<K, P>(key: K, path: P) -> Result<Self, Error>
    where
        K: IntoParam<HKEY>,
        P: IntoParam<PCWSTR>,
    {
        unsafe {
            const ACCESS: REG_SAM_FLAGS = KEY_READ;
            let mut handle: HKEY = Default::default();

            RegOpenKeyExW(key, path.into_param().abi(), 0, ACCESS, &mut handle)?;

            Ok(Key {
                handle,
                access: ACCESS,
            })
        }
    }

    pub fn open_subkey<P>(&self, path: P) -> Result<Self, Error>
    where
        P: IntoParam<PCWSTR>,
    {
        unsafe {
            let mut handle = HKEY::default();

            RegOpenKeyExW(
                self.handle,
                path.into_param().abi(),
                0,
                self.access,
                &mut handle,
            )?;

            Ok(Key {
                handle,
                access: self.access,
            })
        }
    }

    pub fn keys<'a>(&'a self) -> Result<Keys<'a>, Error> {
        Keys::new(self.handle)
    }

    pub fn string_value<P>(&self, name: P) -> Result<Option<PCWSTR>, Error>
    where
        P: IntoParam<PCWSTR> + Copy,
    {
        unsafe {
            const E_FILE_NOT_FOUND: HRESULT = HRESULT(ERROR_FILE_NOT_FOUND.0 as i32);
            const E_MORE_DATA: HRESULT = HRESULT(ERROR_MORE_DATA.0 as i32);

            let mut size = 0u32;
            if let Err(err) = RegGetValueW(
                self.handle,
                PCWSTR::null(),
                name.into_param().abi(),
                RRF_RT_REG_EXPAND_SZ | RRF_RT_REG_MULTI_SZ | RRF_RT_REG_SZ,
                None,
                None,
                Some(&mut size),
            ) {
                match err.code() {
                    E_FILE_NOT_FOUND => return Ok(None),
                    E_MORE_DATA => {}
                    _ => return Err(err),
                }
            }

            let mut data: Vec<u16> = Vec::with_capacity(size as usize / 2);
            RegGetValueW(
                self.handle,
                PCWSTR::null(),
                name.into_param().abi(),
                RRF_RT_REG_EXPAND_SZ | RRF_RT_REG_MULTI_SZ | RRF_RT_REG_SZ,
                None,
                Some(data.as_mut_ptr() as *mut c_void),
                Some(&mut size),
            )?;

            Ok(Some(PCWSTR::from_raw(data.as_ptr())))
        }
    }
}

impl Clone for Key {
    fn clone(&self) -> Self {
        unsafe {
            let mut handle: HKEY = Default::default();

            RegOpenKeyExW(self.handle, None, 0, self.access, &mut handle).unwrap();
            Key {
                handle,
                access: self.access,
            }
        }
    }
}
impl Drop for Key {
    fn drop(&mut self) {
        unsafe {
            let _ = RegCloseKey(self.handle);
        }
    }
}

pub struct Keys<'a> {
    key: HKEY,
    count: u32,
    name: Vec<u16>,
    i: u32,
    _phantom: PhantomData<&'a ()>,
}

impl<'a> Keys<'a> {
    fn new(key: HKEY) -> Result<Self, Error> {
        unsafe {
            let mut count = 0u32;
            let mut name_size = 0x32;
            RegQueryInfoKeyW(
                key,
                PWSTR::null(),
                None,
                None,
                Some(&mut count),
                Some(&mut name_size),
                None,
                None,
                None,
                None,
                None,
                None,
            )?;

            Ok(Keys {
                key,
                count,
                name: Vec::with_capacity(name_size as usize + 1),
                i: 0,
                _phantom: PhantomData,
            })
        }
    }
}

impl<'a> Iterator for Keys<'a> {
    type Item = Key;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let name = PWSTR::from_raw(self.name.as_mut_ptr());
            let mut size = self.name.capacity() as u32;

            RegEnumKeyExW(
                self.key,
                self.i,
                name,
                &mut size,
                None,
                PWSTR::null(),
                None,
                None,
            )
            .ok()?;

            self.i += 1;

            let name = PCWSTR::from_raw(name.as_ptr());
            Key::open(self.key, name).ok()
        }
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.count as usize
    }
}
