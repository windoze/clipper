//! Cross-platform security utilities for Clipper.
//!
//! This module provides functions to secure files and directories so they are
//! only accessible by the current user.
//!
//! - On Unix: Sets umask to 0o077 at process startup, and fixes existing permissions
//! - On Windows: Sets DACL on directories to grant access only to the current user

use std::io;
use std::path::Path;

/// Result of a security fix operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecurityFixResult {
    /// No fix was needed - permissions were already correct
    AlreadySecure,
    /// Permissions were fixed
    Fixed,
    /// The path doesn't exist (not an error for optional paths)
    NotFound,
}

/// Set restrictive umask on Unix systems.
///
/// This should be called early in main() before any files are created.
/// Sets umask to 0o077, which means:
/// - Newly created files will have mode 0600 (owner read/write only)
/// - Newly created directories will have mode 0700 (owner read/write/execute only)
///
/// On Windows, this is a no-op since Windows uses ACLs instead.
#[inline]
pub fn set_restrictive_umask() {
    #[cfg(unix)]
    {
        // SAFETY: umask is a simple system call that only affects the process's file creation mask.
        // It has no unsafe memory operations and cannot cause undefined behavior.
        unsafe {
            libc::umask(0o077);
        }
    }
}

/// Secure a directory so it's only accessible by the current user.
/// Also checks and fixes permissions if they are incorrect.
///
/// On Unix: Checks if the directory has mode 0700, and fixes it if not.
/// On Windows: Sets the directory's DACL to grant full control only to the
/// current user, with inheritance enabled for child objects.
///
/// # Arguments
/// * `path` - Path to the directory to secure
///
/// # Returns
/// * `Ok(SecurityFixResult)` indicating what action was taken
/// * `Err(io::Error)` if the operation failed
pub fn secure_directory(path: &Path) -> io::Result<SecurityFixResult> {
    if !path.exists() {
        return Ok(SecurityFixResult::NotFound);
    }

    #[cfg(unix)]
    {
        unix::secure_directory_unix(path)
    }

    #[cfg(windows)]
    {
        windows::secure_directory_windows(path)
    }

    #[cfg(not(any(unix, windows)))]
    {
        let _ = path;
        Ok(SecurityFixResult::AlreadySecure)
    }
}

/// Secure a file so it's only accessible by the current user.
/// Also checks and fixes permissions if they are incorrect.
///
/// On Unix: Checks if the file has mode 0600, and fixes it if not.
/// On Windows: Sets the file's DACL to grant full control only to the current user.
///
/// # Arguments
/// * `path` - Path to the file to secure
///
/// # Returns
/// * `Ok(SecurityFixResult)` indicating what action was taken
/// * `Err(io::Error)` if the operation failed
pub fn secure_file(path: &Path) -> io::Result<SecurityFixResult> {
    if !path.exists() {
        return Ok(SecurityFixResult::NotFound);
    }

    #[cfg(unix)]
    {
        unix::secure_file_unix(path)
    }

    #[cfg(windows)]
    {
        windows::secure_file_windows(path)
    }

    #[cfg(not(any(unix, windows)))]
    {
        let _ = path;
        Ok(SecurityFixResult::AlreadySecure)
    }
}

/// Recursively secure a directory and all its contents.
/// Logs warnings for each item that needs fixing.
///
/// # Arguments
/// * `path` - Path to the directory to secure recursively
/// * `warn_fn` - Function to call with warning messages when fixing permissions
///
/// # Returns
/// * `Ok(usize)` - Number of items that were fixed
/// * `Err(io::Error)` if a critical operation failed
pub fn secure_directory_recursive<F>(path: &Path, warn_fn: F) -> io::Result<usize>
where
    F: Fn(&str),
{
    secure_directory_recursive_inner(path, &warn_fn)
}

fn secure_directory_recursive_inner(path: &Path, warn_fn: &dyn Fn(&str)) -> io::Result<usize> {
    if !path.exists() {
        return Ok(0);
    }

    let mut fixed_count = 0;

    // First secure the root directory
    match secure_directory(path)? {
        SecurityFixResult::Fixed => {
            warn_fn(&format!(
                "Fixed permissions on directory: {}",
                path.display()
            ));
            fixed_count += 1;
        }
        SecurityFixResult::AlreadySecure | SecurityFixResult::NotFound => {}
    }

    // Then recursively process contents
    if path.is_dir() {
        match std::fs::read_dir(path) {
            Ok(entries) => {
                for entry in entries.flatten() {
                    let entry_path = entry.path();
                    if entry_path.is_dir() {
                        // Recursively secure subdirectories
                        fixed_count += secure_directory_recursive_inner(&entry_path, warn_fn)?;
                    } else if entry_path.is_file() {
                        // Secure files
                        match secure_file(&entry_path)? {
                            SecurityFixResult::Fixed => {
                                warn_fn(&format!(
                                    "Fixed permissions on file: {}",
                                    entry_path.display()
                                ));
                                fixed_count += 1;
                            }
                            SecurityFixResult::AlreadySecure | SecurityFixResult::NotFound => {}
                        }
                    }
                }
            }
            Err(e) => {
                warn_fn(&format!(
                    "Failed to read directory {}: {}",
                    path.display(),
                    e
                ));
            }
        }
    }

    Ok(fixed_count)
}

#[cfg(unix)]
mod unix {
    use std::fs;
    use std::io;
    use std::os::unix::fs::PermissionsExt;
    use std::path::Path;

    use super::SecurityFixResult;

    /// Expected mode for directories: rwx------ (0700)
    const SECURE_DIR_MODE: u32 = 0o700;
    /// Expected mode for files: rw------- (0600)
    const SECURE_FILE_MODE: u32 = 0o600;
    /// Mask to extract permission bits (ignore file type bits)
    const PERMISSION_MASK: u32 = 0o777;

    /// Secure a directory on Unix by ensuring it has mode 0700
    pub fn secure_directory_unix(path: &Path) -> io::Result<SecurityFixResult> {
        let metadata = fs::metadata(path)?;
        let current_mode = metadata.permissions().mode() & PERMISSION_MASK;

        if current_mode == SECURE_DIR_MODE {
            return Ok(SecurityFixResult::AlreadySecure);
        }

        // Fix the permissions
        let mut perms = metadata.permissions();
        perms.set_mode(SECURE_DIR_MODE);
        fs::set_permissions(path, perms)?;

        Ok(SecurityFixResult::Fixed)
    }

    /// Secure a file on Unix by ensuring it has mode 0600
    pub fn secure_file_unix(path: &Path) -> io::Result<SecurityFixResult> {
        let metadata = fs::metadata(path)?;
        let current_mode = metadata.permissions().mode() & PERMISSION_MASK;

        if current_mode == SECURE_FILE_MODE {
            return Ok(SecurityFixResult::AlreadySecure);
        }

        // Fix the permissions
        let mut perms = metadata.permissions();
        perms.set_mode(SECURE_FILE_MODE);
        fs::set_permissions(path, perms)?;

        Ok(SecurityFixResult::Fixed)
    }
}

#[cfg(windows)]
mod windows {
    use std::io;
    use std::path::Path;
    use std::ptr;

    use super::SecurityFixResult;
    use windows_sys::Win32::Foundation::{CloseHandle, LocalFree, HANDLE, INVALID_HANDLE_VALUE};
    use windows_sys::Win32::Security::Authorization::{SetNamedSecurityInfoW, SE_FILE_OBJECT};
    use windows_sys::Win32::Security::{
        AddAccessAllowedAceEx, GetTokenInformation, InitializeAcl, CONTAINER_INHERIT_ACE,
        DACL_SECURITY_INFORMATION, OBJECT_INHERIT_ACE, PROTECTED_DACL_SECURITY_INFORMATION, PSID,
        TOKEN_QUERY, TOKEN_USER, TokenUser, ACL as WIN_ACL, ACL_REVISION,
    };
    use windows_sys::Win32::System::Memory::LocalAlloc;
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    // Access rights for files and directories
    const FILE_ALL_ACCESS: u32 = 0x1F01FF;
    const LPTR: u32 = 0x0040;

    /// Get the SID of the current user
    fn get_current_user_sid() -> io::Result<Vec<u8>> {
        unsafe {
            // Open process token
            let mut token_handle: HANDLE = INVALID_HANDLE_VALUE;
            if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token_handle) == 0 {
                return Err(io::Error::last_os_error());
            }

            // Get token information size
            let mut token_info_len: u32 = 0;
            GetTokenInformation(token_handle, TokenUser, ptr::null_mut(), 0, &mut token_info_len);

            // Allocate buffer and get token information
            let token_info = LocalAlloc(LPTR, token_info_len as usize);
            if token_info.is_null() {
                CloseHandle(token_handle);
                return Err(io::Error::last_os_error());
            }

            let result = GetTokenInformation(
                token_handle,
                TokenUser,
                token_info,
                token_info_len,
                &mut token_info_len,
            );

            CloseHandle(token_handle);

            if result == 0 {
                LocalFree(token_info);
                return Err(io::Error::last_os_error());
            }

            // Extract SID from TOKEN_USER structure
            let token_user = &*(token_info as *const TOKEN_USER);
            let sid_ptr = token_user.User.Sid;

            // Get SID length and copy it
            let sid_len = get_length_sid(sid_ptr);
            let mut sid_vec = vec![0u8; sid_len];
            ptr::copy_nonoverlapping(sid_ptr as *const u8, sid_vec.as_mut_ptr(), sid_len);

            LocalFree(token_info);
            Ok(sid_vec)
        }
    }

    /// Get the length of a SID
    unsafe fn get_length_sid(sid: PSID) -> usize {
        // SID structure: Revision (1) + SubAuthorityCount (1) + IdentifierAuthority (6) + SubAuthorities (4 * count)
        // SAFETY: sid is a valid PSID pointer from the Windows API, and we're accessing
        // the SubAuthorityCount field at offset 1 which is guaranteed to exist.
        let sub_auth_count = unsafe { *((sid as *const u8).add(1)) };
        8 + (sub_auth_count as usize * 4)
    }

    /// Create an ACL that grants full access only to the specified SID
    fn create_user_only_acl(user_sid: &[u8], inherit: bool) -> io::Result<Vec<u8>> {
        unsafe {
            // Calculate ACL size: base ACL + one ACE
            // ACE size = ACE header (4) + access mask (4) + SID length
            let ace_size = 4 + 4 + user_sid.len();
            let acl_size = std::mem::size_of::<WIN_ACL>() + ace_size;

            // Allocate and initialize ACL
            let mut acl_buffer = vec![0u8; acl_size];
            let acl_ptr = acl_buffer.as_mut_ptr() as *mut WIN_ACL;

            if InitializeAcl(acl_ptr, acl_size as u32, ACL_REVISION as u32) == 0 {
                return Err(io::Error::last_os_error());
            }

            // Add access allowed ACE with inheritance flags if requested
            let ace_flags = if inherit {
                CONTAINER_INHERIT_ACE | OBJECT_INHERIT_ACE
            } else {
                0
            };

            if AddAccessAllowedAceEx(
                acl_ptr,
                ACL_REVISION as u32,
                ace_flags,
                FILE_ALL_ACCESS,
                user_sid.as_ptr() as PSID,
            ) == 0
            {
                return Err(io::Error::last_os_error());
            }

            Ok(acl_buffer)
        }
    }

    /// Convert a path to a wide string for Windows API
    fn path_to_wide(path: &Path) -> Vec<u16> {
        use std::os::windows::ffi::OsStrExt;
        path.as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }

    /// Secure a directory on Windows by setting a DACL that grants access only to the current user
    /// Note: On Windows we always set the ACL since checking current ACL is complex
    /// The operation is idempotent so this is safe
    pub fn secure_directory_windows(path: &Path) -> io::Result<SecurityFixResult> {
        let user_sid = get_current_user_sid()?;
        let acl = create_user_only_acl(&user_sid, true)?; // inherit for directories
        let wide_path = path_to_wide(path);

        unsafe {
            let result = SetNamedSecurityInfoW(
                wide_path.as_ptr(),
                SE_FILE_OBJECT,
                DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
                ptr::null_mut(),
                ptr::null_mut(),
                acl.as_ptr() as *const WIN_ACL,
                ptr::null_mut(),
            );

            if result != 0 {
                return Err(io::Error::from_raw_os_error(result as i32));
            }
        }

        // On Windows we always apply the ACL, report as Fixed
        // (ideally we'd check first, but that's complex)
        Ok(SecurityFixResult::Fixed)
    }

    /// Secure a file on Windows by setting a DACL that grants access only to the current user
    pub fn secure_file_windows(path: &Path) -> io::Result<SecurityFixResult> {
        let user_sid = get_current_user_sid()?;
        let acl = create_user_only_acl(&user_sid, false)?; // no inherit for files
        let wide_path = path_to_wide(path);

        unsafe {
            let result = SetNamedSecurityInfoW(
                wide_path.as_ptr(),
                SE_FILE_OBJECT,
                DACL_SECURITY_INFORMATION | PROTECTED_DACL_SECURITY_INFORMATION,
                ptr::null_mut(),
                ptr::null_mut(),
                acl.as_ptr() as *const WIN_ACL,
                ptr::null_mut(),
            );

            if result != 0 {
                return Err(io::Error::from_raw_os_error(result as i32));
            }
        }

        // On Windows we always apply the ACL, report as Fixed
        Ok(SecurityFixResult::Fixed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_restrictive_umask() {
        // Should not panic
        set_restrictive_umask();
    }

    #[test]
    fn test_secure_directory_nonexistent() {
        let result = secure_directory(Path::new("/nonexistent/path/12345"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), SecurityFixResult::NotFound);
    }

    #[test]
    fn test_secure_file_nonexistent() {
        let result = secure_file(Path::new("/nonexistent/path/12345/file.txt"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), SecurityFixResult::NotFound);
    }

    #[cfg(unix)]
    #[test]
    fn test_secure_directory_fixes_permissions() {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = std::env::temp_dir().join("clipper_security_test_dir");
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();

        // Set insecure permissions
        let mut perms = fs::metadata(&temp_dir).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&temp_dir, perms).unwrap();

        // Secure should fix it
        let result = secure_directory(&temp_dir).unwrap();
        assert_eq!(result, SecurityFixResult::Fixed);

        // Verify it's now 0700
        let mode = fs::metadata(&temp_dir).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o700);

        // Running again should report AlreadySecure
        let result = secure_directory(&temp_dir).unwrap();
        assert_eq!(result, SecurityFixResult::AlreadySecure);

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[cfg(unix)]
    #[test]
    fn test_secure_file_fixes_permissions() {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;

        let temp_file = std::env::temp_dir().join("clipper_security_test_file.txt");
        let _ = fs::remove_file(&temp_file);
        fs::write(&temp_file, "test").unwrap();

        // Set insecure permissions
        let mut perms = fs::metadata(&temp_file).unwrap().permissions();
        perms.set_mode(0o644);
        fs::set_permissions(&temp_file, perms).unwrap();

        // Secure should fix it
        let result = secure_file(&temp_file).unwrap();
        assert_eq!(result, SecurityFixResult::Fixed);

        // Verify it's now 0600
        let mode = fs::metadata(&temp_file).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);

        // Running again should report AlreadySecure
        let result = secure_file(&temp_file).unwrap();
        assert_eq!(result, SecurityFixResult::AlreadySecure);

        let _ = fs::remove_file(&temp_file);
    }
}
