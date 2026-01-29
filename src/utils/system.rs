//! System-level utilities for cross-platform support.

use crate::error::Result;

/// Check if a process with the given PID is still running.
/// Works on Linux, macOS, and falls back to kill(0) on other Unix systems.
pub fn is_process_alive(pid: u32) -> bool {
    #[cfg(unix)]
    {
        // On Linux, check /proc/{pid} for efficiency and safety
        #[cfg(target_os = "linux")]
        {
            let proc_path = format!("/proc/{pid}");
            if Path::new("/proc").is_dir() {
                return Path::new(&proc_path).exists();
            }
        }

        // On other Unix systems (including macOS) or if /proc is missing, use `kill -0`
        // This avoids unsafe blocks required for libc::kill, satisfying -F unsafe-code
        use std::process::Command;
        match Command::new("kill").arg("-0").arg(pid.to_string()).output() {
            Ok(output) => {
                if output.status.success() {
                    true
                } else {
                    // If exit code is non-zero, it might be "No such process" (dead)
                    // or "Permission denied" (alive but owned by another user).
                    // kill -0 typically returns 1 for both.
                    // We check stderr for clues, though this is heuristic.
                    let stderr = String::from_utf8_lossy(&output.stderr).to_lowercase();
                    stderr.contains("denied") || stderr.contains("permitted")
                }
            }
            Err(_) => {
                // If we can't run kill, assume alive to avoid breaking valid locks
                true
            }
        }
    }

    #[cfg(not(unix))]
    {
        // On non-Unix systems (Windows), we can't easily check process liveness 
        // without extra dependencies. Assume the process is alive.
        true
    }
}

/// Get the current process's resident set size (RSS) in megabytes.
pub fn get_current_rss_mb() -> Result<f64> {
    #[cfg(target_os = "linux")]
    {
        if let Ok(statm) = std::fs::read_to_string("/proc/self/statm") {
            let parts: Vec<&str> = statm.split_whitespace().collect();
            if let Some(rss_pages) = parts.get(1) {
                if let Ok(pages) = rss_pages.parse::<u64>() {
                    let page_size = 4096; // Standard page size assumption
                    return Ok((pages * page_size) as f64 / (1024.0 * 1024.0));
                }
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let pid = std::process::id().to_string();
        let output = Command::new("ps")
            .arg("-o")
            .arg("rss=")
            .arg("-p")
            .arg(&pid)
            .output();

        if let Ok(out) = output {
            if out.status.success() {
                let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if let Ok(rss_kb) = s.parse::<f64>() {
                    return Ok(rss_kb / 1024.0);
                }
            }
        }
    }

    // Fallback or other OS
    Ok(0.0)
}
