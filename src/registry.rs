use windows::{
    core::{IntoParam, Result, HRESULT, PCWSTR, PWSTR},
    Win32::{
        Foundation::ERROR_MORE_DATA,
        System::Registry::{self, *},
    },
};

pub use Registry::HKEY_CURRENT_USER;
pub use Registry::HKEY_LOCAL_MACHINE;

const E_MORE_DATA: HRESULT = HRESULT((0x80070000u32 | ERROR_MORE_DATA.0 as u32) as i32);

#[derive(Debug)]
pub struct Key {
    handle: HKEY,
    access: REG_SAM_FLAGS,
}

impl Key {
    pub fn create<K, P>(key: K, path: P) -> Result<Self>
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

    pub fn open<K, P>(key: K, path: P) -> Result<Self>
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

    pub fn open_subkey<P>(&self, path: P) -> Result<Self>
    where
        P: IntoParam<PCWSTR>,
    {
        unsafe {
            let mut handle: HKEY = Default::default();

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

    pub fn keys<'a>(&'a self) -> Result<Keys<'a>> {
        Keys::new(&self.handle)
    }

    pub fn values<'a>(&'a self) -> Result<Values<'a>> {
        Values::new(&self.handle)
    }

    pub fn value<P>(&self, name: P) -> Option<Value>
    where
        P: IntoParam<PCWSTR> + Copy,
    {
        unsafe {
            let mut data_type: REG_VALUE_TYPE = Default::default();
            let mut data_size = 0u32;

            if let Err(err) = RegGetValueW(
                self.handle,
                PCWSTR::null(),
                name.into_param().abi(),
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
                name.into_param().abi(),
                RRF_RT_ANY,
                None,
                Some(data.as_mut_ptr() as *mut std::ffi::c_void),
                Some(&mut data_size),
            )
            .ok()?;

            Value::from(&data, data_type)
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

#[derive(Debug, PartialEq)]
pub enum Value {
    Binary(Vec<u8>),
    DWord(u32),
    MultiString(Vec<String>),
    QWord(u64),
    String(String),
}

impl Value {
    fn from(data: &[u8], data_type: REG_VALUE_TYPE) -> Option<Self> {
        match data_type {
            REG_BINARY => Some(Value::Binary(data.to_vec())),
            REG_DWORD => {
                let mut buffer = [0u8; 4];
                buffer.copy_from_slice(data);
                Some(Value::DWord(u32::from_le_bytes(buffer)))
            }
            REG_QWORD => {
                let mut buffer = [0u8; 8];
                buffer.copy_from_slice(data);
                Some(Value::QWord(u64::from_le_bytes(buffer)))
            }
            REG_SZ | REG_EXPAND_SZ => unsafe {
                if data.len() == 0 {
                    return Some(Value::String("".to_string()));
                }
                let data = PCWSTR::from_raw(data.as_ptr() as *const u16);
                Some(Value::String(String::from_utf16_lossy(data.as_wide())))
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
                Some(Value::MultiString(data))
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

            Value::from(&data, REG_VALUE_TYPE(data_type))
        }
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.count as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_from_dword() {
        let data = vec![0, 1, 2, 3];
        let value = Value::from(&data, REG_DWORD).unwrap();
        assert_eq!(value, Value::DWord(50462976));
    }

    #[test]
    fn value_from_qword() {
        let data = vec![0, 1, 2, 3, 4, 5, 6, 7];
        let value = Value::from(&data, REG_QWORD).unwrap();
        assert_eq!(value, Value::QWord(506097522914230528));
    }

    #[test]
    fn value_from_binary() {
        let data = vec![0, 1, 2, 3];
        let value = Value::from(&data, REG_BINARY).unwrap();
        assert_eq!(value, Value::Binary(vec![0, 1, 2, 3]));
    }

    #[test]
    fn value_from_sz() {
        let data = "h\0e\0l\0l\0o\0".as_bytes();
        let value = Value::from(data, REG_SZ).unwrap();
        assert_eq!(value, Value::String("hello".to_string()));
    }

    #[test]
    fn value_from_expand_sz() {
        let data = "h\0e\0l\0l\0o\0".as_bytes();
        let value = Value::from(data, REG_EXPAND_SZ).unwrap();
        assert_eq!(value, Value::String("hello".to_string()));
    }

    #[test]
    fn value_from_multi_sz() {
        let data = "h\0e\0l\0l\0o\0\0\0w\0o\0r\0l\0d\0\0\0\0\0".as_bytes();
        let value = Value::from(data, REG_MULTI_SZ).unwrap();
        assert_eq!(
            value,
            Value::MultiString(vec!["hello".to_string(), "world".to_string()])
        );
    }
}
