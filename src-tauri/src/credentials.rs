/// Save the WFM access token to Windows Credential Manager (encrypted by the OS).
/// Stored under "FrameForge_WFM" — username field = the email, blob = the token.
#[tauri::command]
#[cfg(target_os = "windows")]
pub(crate) fn wfm_save_credentials(email: String, token: String) -> Result<(), String> {
    use windows_sys::Win32::Security::Credentials::{
        CredWriteW, CREDENTIALW, CRED_TYPE_GENERIC, CRED_PERSIST_LOCAL_MACHINE,
    };
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    let target: Vec<u16> = OsStr::new("FrameForge_WFM").encode_wide().chain(Some(0)).collect();
    let user:   Vec<u16> = OsStr::new(&email).encode_wide().chain(Some(0)).collect();
    let token_bytes = token.as_bytes();

    let cred = CREDENTIALW {
        Flags: 0,
        Type: CRED_TYPE_GENERIC,
        TargetName: target.as_ptr() as *mut _,
        Comment: std::ptr::null_mut(),
        LastWritten: unsafe { std::mem::zeroed() },
        CredentialBlobSize: token_bytes.len() as u32,
        CredentialBlob: token_bytes.as_ptr() as *mut _,
        Persist: CRED_PERSIST_LOCAL_MACHINE,
        AttributeCount: 0,
        Attributes: std::ptr::null_mut(),
        TargetAlias: std::ptr::null_mut(),
        UserName: user.as_ptr() as *mut _,
    };
    let ok = unsafe { CredWriteW(&cred, 0) };
    if ok == 0 { Err("Failed to save to Windows Credential Manager".into()) } else { Ok(()) }
}

/// Load WFM credentials from Windows Credential Manager.
#[tauri::command]
#[cfg(target_os = "windows")]
pub(crate) fn wfm_load_credentials() -> Result<Option<(String, String)>, String> {
    use windows_sys::Win32::Security::Credentials::{
        CredReadW, CredFree, CREDENTIALW, CRED_TYPE_GENERIC,
    };
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::slice;

    let target: Vec<u16> = OsStr::new("FrameForge_WFM").encode_wide().chain(Some(0)).collect();
    let mut cred_ptr: *mut CREDENTIALW = std::ptr::null_mut();
    let ok = unsafe { CredReadW(target.as_ptr(), CRED_TYPE_GENERIC, 0, &mut cred_ptr) };
    if ok == 0 || cred_ptr.is_null() { return Ok(None); }

    let cred = unsafe { &*cred_ptr };
    let email = unsafe {
        let ptr = cred.UserName;
        if ptr.is_null() { String::new() } else {
            let len = (0..).take_while(|&i| *ptr.offset(i) != 0).count();
            String::from_utf16_lossy(slice::from_raw_parts(ptr, len))
        }
    };
    let token = unsafe {
        if cred.CredentialBlob.is_null() || cred.CredentialBlobSize == 0 { String::new() } else {
            String::from_utf8_lossy(slice::from_raw_parts(cred.CredentialBlob, cred.CredentialBlobSize as usize)).to_string()
        }
    };
    unsafe { CredFree(cred_ptr as *mut _); }
    Ok(Some((email, token)))
}

/// Delete saved WFM credentials from Windows Credential Manager.
///
/// Async only so that every platform's delete has one shape for `wfm_logout` to
/// await; `CredDeleteW` itself returns immediately and never prompts.
#[tauri::command]
#[cfg(target_os = "windows")]
pub(crate) async fn wfm_delete_credentials() -> Result<(), String> {
    use windows_sys::Win32::Security::Credentials::{CredDeleteW, CRED_TYPE_GENERIC};
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    let target: Vec<u16> = OsStr::new("FrameForge_WFM").encode_wide().chain(Some(0)).collect();
    unsafe { CredDeleteW(target.as_ptr(), CRED_TYPE_GENERIC, 0); }
    Ok(())
}
