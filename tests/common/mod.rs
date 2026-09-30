use std::process::Command;
use std::sync::OnceLock;

pub struct Oracle {
    pub executable: String,
    pub is_bsd: bool,
}

pub fn oracle() -> &'static Oracle {
    static ORACLE: OnceLock<Oracle> = OnceLock::new();
    ORACLE.get_or_init(|| {
        let executable = std::env::var("RGREP_ORACLE").unwrap_or_else(|_| "grep".into());
        let output = Command::new(&executable)
            .arg("--version")
            .output()
            .expect("cannot execute RGREP_ORACLE");
        let version = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            version.contains("GNU grep") || version.contains("BSD grep"),
            "unsupported oracle {executable}: {version}"
        );
        let is_bsd = version.contains("BSD grep");
        eprintln!(
            "[oracle] {executable}: {}",
            version.lines().next().unwrap_or("")
        );
        if is_bsd {
            eprintln!("[coverage] GNU property comparisons are skipped on BSD");
        }
        Oracle { executable, is_bsd }
    })
}
