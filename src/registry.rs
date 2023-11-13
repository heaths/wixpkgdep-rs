// Copyright 2023 Heath Stewart.
// Licensed under the MIT License. See LICENSE.txt in the project root for license information.

use std::fmt::Display;

use windows::{
    core::{IntoParam, Result, HRESULT, PCWSTR, PWSTR},
    Win32::{
        Foundation::{ERROR_FILE_NOT_FOUND, ERROR_MORE_DATA},
        System::Registry::{self, *},
    },
};

pub use Registry::HKEY_CURRENT_USER;
pub use Registry::HKEY_LOCAL_MACHINE;

pub const E_FILE_NOT_FOUND: HRESULT = HRESULT((0x80070000u32 | ERROR_FILE_NOT_FOUND.0) as i32);

#[derive(Debug)]
pub struct Key {
    handle: HKEY,
    access: REG_SAM_FLAGS,

    pub name: String,
}

impl Key {
    #[allow(dead_code)] // TODO
    pub fn create<K, P>(key: K, path: P) -> Result<Self>
    where
        K: IntoParam<HKEY>,
        P: IntoParam<PCWSTR>,
    {
        unsafe {
            const ACCESS: REG_SAM_FLAGS = KEY_ALL_ACCESS;
            let mut handle: HKEY = Default::default();

            let path: PCWSTR = path.into_param().abi();
            RegCreateKeyW(key, path, &mut handle)?;
            Ok(Key {
                handle,
                access: ACCESS,
                name: get_name(path),
            })
        }
    }

    pub fn open<K, P>(key: K, path: P) -> Result<Self>
    where
        K: IntoParam<HKEY>,
        P: IntoParam<PCWSTR>,
    {
        unsafe {
            const ACCESS: REG_SAM_FLAGS = KEY_READ;
            let mut handle: HKEY = Default::default();

            let path: PCWSTR = path.into_param().abi();
            RegOpenKeyExW(key, path, 0, ACCESS, &mut handle)?;
            Ok(Key {
                handle,
                access: ACCESS,
                name: get_name(path),
            })
        }
    }

    pub fn open_subkey<P>(&self, path: P) -> Result<Self>
    where
        P: IntoParam<PCWSTR>,
    {
        unsafe {
            let mut handle: HKEY = Default::default();

            let path: PCWSTR = path.into_param().abi();
            RegOpenKeyExW(self.handle, path, 0, self.access, &mut handle)?;
            Ok(Key {
                handle,
                access: self.access,
                name: get_name(path),
            })
        }
    }

    #[allow(dead_code)] // TODO
    pub fn keys(&self) -> Result<Keys<'_>> {
        Keys::new(&self.handle)
    }

    #[allow(dead_code)] // TODO
    pub fn values(&self) -> Result<Values<'_>> {
        Values::new(&self.handle)
    }

    pub fn value<P>(&self, name: P) -> Option<Value>
    where
        P: IntoParam<PCWSTR> + Copy,
    {
        const E_MORE_DATA: HRESULT = HRESULT((0x80070000u32 | ERROR_MORE_DATA.0) as i32);
        unsafe {
            let name: PCWSTR = name.into_param().abi();
            let mut data_type: REG_VALUE_TYPE = Default::default();
            let mut data_size = 0u32;

            if let Err(err) = RegGetValueW(
                self.handle,
                PCWSTR::null(),
                name,
                RRF_RT_ANY,
                Some(&mut data_type),
                None,
                Some(&mut data_size),
            ) {
                match err.code() {
                    E_MORE_DATA => {}
                    _ => return None,
                }
            }

            let mut data = vec![0x8; data_size as usize];
            RegGetValueW(
                self.handle,
                PCWSTR::null(),
                name,
                RRF_RT_ANY,
                None,
                Some(data.as_mut_ptr() as *mut std::ffi::c_void),
                Some(&mut data_size),
            )
            .ok()?;

            let name = String::from_utf16_lossy(name.into_param().abi().as_wide());
            Value::from(&name, &data, data_type)
        }
    }
}

impl Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Drop for Key {
    fn drop(&mut self) {
        unsafe {
            let _ = RegCloseKey(self.handle);
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Value {
    pub name: String,
    pub data: Data,
}

impl Value {
    fn from(name: &str, data: &[u8], data_type: REG_VALUE_TYPE) -> Option<Self> {
        Some(Self {
            name: name.to_string(),
            data: Data::from(data, data_type)?,
        })
    }
}

#[derive(Debug, PartialEq)]
pub enum Data {
    Binary(Vec<u8>),
    DWord(u32),
    MultiString(Vec<String>),
    QWord(u64),
    String(String),
}

impl Data {
    fn from(data: &[u8], data_type: REG_VALUE_TYPE) -> Option<Self> {
        match data_type {
            REG_BINARY => Some(Data::Binary(data.to_vec())),
            REG_DWORD => {
                let mut buffer = [0u8; 4];
                buffer.copy_from_slice(data);
                Some(Data::DWord(u32::from_le_bytes(buffer)))
            }
            REG_QWORD => {
                let mut buffer = [0u8; 8];
                buffer.copy_from_slice(data);
                Some(Data::QWord(u64::from_le_bytes(buffer)))
            }
            REG_SZ | REG_EXPAND_SZ => unsafe {
                if data.is_empty() {
                    return Some(Data::String("".to_string()));
                }
                let data = PCWSTR::from_raw(data.as_ptr() as *const u16);
                Some(Data::String(String::from_utf16_lossy(data.as_wide())))
            },
            REG_MULTI_SZ => unsafe {
                let data = std::slice::from_raw_parts(data.as_ptr() as *const u16, data.len() / 2);
                let data: Vec<String> = data
                    .split(|c| *c == 0u16)
                    .filter_map(|s| {
                        if s.is_empty() {
                            return None;
                        }

                        Some(String::from_utf16_lossy(s))
                    })
                    .collect();
                Some(Data::MultiString(data))
            },
            _ => None,
        }
    }
}

pub struct Keys<'a> {
    key: &'a HKEY,
    count: u32,
    name: Vec<u16>,
    i: u32,
}

impl<'a> Keys<'a> {
    fn new(key: &'a HKEY) -> Result<Self> {
        unsafe {
            let mut count = 0u32;
            let mut name_size = 0x32;
            RegQueryInfoKeyW(
                *key,
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
                name: vec![0u16; name_size as usize + 1],
                i: 0,
            })
        }
    }
}

impl<'a> Iterator for Keys<'a> {
    type Item = Key;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let name = PWSTR::from_raw(self.name.as_mut_ptr());
            let mut name_size = self.name.len() as u32;

            RegEnumKeyExW(
                *self.key,
                self.i,
                name,
                &mut name_size,
                None,
                PWSTR::null(),
                None,
                None,
            )
            .ok()?;

            self.i += 1;

            let name = PCWSTR::from_raw(name.as_ptr());
            Key::open(*self.key, name).ok()
        }
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.count as usize
    }
}

pub struct Values<'a> {
    key: &'a HKEY,
    count: u32,
    name: Vec<u16>,
    i: u32,
}

impl<'a> Values<'a> {
    fn new(key: &'a HKEY) -> Result<Self> {
        unsafe {
            let mut count = 0u32;
            let mut name_size = 0x32;
            RegQueryInfoKeyW(
                *key,
                PWSTR::null(),
                None,
                None,
                None,
                None,
                None,
                Some(&mut count),
                Some(&mut name_size),
                None,
                None,
                None,
            )?;

            Ok(Values {
                key,
                count,
                name: vec![0u16; name_size as usize + 1],
                i: 0,
            })
        }
    }
}

impl<'a> Iterator for Values<'a> {
    type Item = Value;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let name = PWSTR::from_raw(self.name.as_mut_ptr());
            let mut name_size = self.name.len() as u32;
            let mut data_type = 0u32;
            let mut data_size = 0u32;

            RegEnumValueW(
                *self.key,
                self.i,
                name,
                &mut name_size,
                None,
                Some(&mut data_type),
                None,
                Some(&mut data_size),
            )
            .ok()?;

            name_size += 1;
            let mut data = vec![0u8; data_size as usize];

            RegEnumValueW(
                *self.key,
                self.i,
                name,
                &mut name_size,
                None,
                None,
                Some(data.as_mut_ptr()),
                Some(&mut data_size),
            )
            .ok()?;

            self.i += 1;

            let name = String::from_utf16_lossy(name.as_wide());
            Value::from(&name, &data, REG_VALUE_TYPE(data_type))
        }
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.count as usize
    }
}

fn get_name<K>(path: K) -> String
where
    K: IntoParam<PCWSTR>,
{
    unsafe {
        let path = path.into_param().abi();
        String::from_utf16_lossy(path.as_wide())
            .trim_end_matches('\\')
            .rsplit('\\')
            .take(1)
            .last()
            .map(|s| s.to_string())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::core::w;

    #[test]
    fn get_name_terminated() {
        let path = w!("grandparent\\parent\\child\\");
        let name = get_name(path);
        assert_eq!(name, "child");
    }

    #[test]
    fn get_name_unterminated() {
        let path = w!("grandparent\\parent\\child");
        let name = get_name(path);
        assert_eq!(name, "child");
    }

    #[test]
    fn data_from_dword() {
        let data = vec![0, 1, 2, 3];
        let data = Data::from(&data, REG_DWORD).unwrap();
        assert_eq!(data, Data::DWord(50462976));
    }

    #[test]
    fn data_from_qword() {
        let data = vec![0, 1, 2, 3, 4, 5, 6, 7];
        let data = Data::from(&data, REG_QWORD).unwrap();
        assert_eq!(data, Data::QWord(506097522914230528));
    }

    #[test]
    fn data_from_binary() {
        let data = vec![0, 1, 2, 3];
        let data = Data::from(&data, REG_BINARY).unwrap();
        assert_eq!(data, Data::Binary(vec![0, 1, 2, 3]));
    }

    #[test]
    fn data_from_sz() {
        let data = b"h\0e\0l\0l\0o\0";
        let data = Data::from(data, REG_SZ).unwrap();
        assert_eq!(data, Data::String("hello".to_string()));
    }

    #[test]
    fn data_from_expand_sz() {
        let data = b"h\0e\0l\0l\0o\0";
        let data = Data::from(data, REG_EXPAND_SZ).unwrap();
        assert_eq!(data, Data::String("hello".to_string()));
    }

    #[test]
    fn data_from_multi_sz() {
        let data = b"h\0e\0l\0l\0o\0\0\0w\0o\0r\0l\0d\0\0\0\0\0";
        let data = Data::from(data, REG_MULTI_SZ).unwrap();
        assert_eq!(
            data,
            Data::MultiString(vec!["hello".to_string(), "world".to_string()])
        );
    }
}
