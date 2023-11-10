use std::ffi::c_void;
use windows::{
    core::{Error, IntoParam, PCWSTR},
    Win32::{
        Foundation::ERROR_FILE_NOT_FOUND,
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
    pub fn open<K>(key: K, path: &PCWSTR) -> Result<Self, Error>
    where
        K: IntoParam<HKEY>,
    {
        unsafe {
            const ACCESS: REG_SAM_FLAGS = KEY_READ;
            let mut handle: HKEY = Default::default();

            RegOpenKeyExW(key, path.to_owned(), 0, ACCESS, &mut handle)?;

            Ok(Key {
                handle,
                access: ACCESS,
            })
        }
    }

    pub fn open_subkey(&self, path: &PCWSTR) -> Result<Self, Error> {
        unsafe {
            const ACCESS: REG_SAM_FLAGS = KEY_READ;
            let mut handle = HKEY::default();

            RegOpenKeyExW(self.handle, path.to_owned(), 0, KEY_READ, &mut handle)?;

            Ok(Key {
                handle,
                access: ACCESS,
            })
        }
    }

    pub fn string_value(&self, name: &PCWSTR) -> Result<Option<PCWSTR>, Error> {
        unsafe {
            let mut size = 0u32;

            if let Err(err) = RegGetValueW(
                self.handle,
                PCWSTR::null(),
                name.to_owned(),
                RRF_RT_REG_EXPAND_SZ | RRF_RT_REG_MULTI_SZ | RRF_RT_REG_SZ,
                None,
                None,
                Some(&mut size),
            ) {
                if err.code() == ERROR_FILE_NOT_FOUND.to_hresult() {
                    return Ok(None);
                }
            }

            let mut data: Vec<u16> = Vec::with_capacity(size as usize);
            RegGetValueW(
                self.handle,
                PCWSTR::null(),
                name.to_owned(),
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
