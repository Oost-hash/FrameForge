#[tauri::command]
pub(crate) fn get_system_locale() -> String {
    let mut buf = [0u16; 85]; // LOCALE_NAME_MAX_LENGTH
    let len = unsafe {
        windows_sys::Win32::Globalization::GetUserDefaultLocaleName(
            buf.as_mut_ptr(),
            buf.len() as i32,
        )
    };
    if len > 1 {
        String::from_utf16_lossy(&buf[..(len as usize - 1)])
    } else {
        "en-US".to_string()
    }
}
