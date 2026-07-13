//! RunHaven-owned custom pet packages bundled with the TUI.
//!
//! Codex custom pets normally live under `$CODEX_HOME/pets/<id>`. RunHaven
//! ships Cubby as a verified default pet, so this module materializes that same
//! package shape before the Codex-vendored model and renderer load it.

use std::fs;
use std::fs::OpenOptions;
use std::io;
use std::io::Write as _;
use std::path::Path;

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;

use super::model::Pet;

pub(crate) const RUNHAVEN_BUNDLED_CUBBY_ID: &str = "runhaven-cubby";
pub(crate) const RUNHAVEN_BUNDLED_CUBBY_SELECTOR: &str = "custom:runhaven-cubby";
pub(crate) const RUNHAVEN_BUNDLED_CUBBY_DISPLAY_NAME: &str = "Cubby";
pub(crate) const RUNHAVEN_BUNDLED_CUBBY_DESCRIPTION: &str =
    "A cheerful glass cube mascot that keeps your agent tucked safely in its own little haven.";

const CUBBY_PET_JSON: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/cubby/pet.json"
));
const CUBBY_SPRITESHEET: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/cubby/spritesheet.webp"
));

pub(crate) fn is_runhaven_bundled_cubby_selector(selector: &str) -> bool {
    selector == RUNHAVEN_BUNDLED_CUBBY_ID || selector == RUNHAVEN_BUNDLED_CUBBY_SELECTOR
}

pub(crate) fn ensure_runhaven_bundled_pet_for_selector(
    selector: &str,
    codex_home: &Path,
) -> Result<()> {
    if is_runhaven_bundled_cubby_selector(selector) {
        ensure_runhaven_cubby(codex_home)?;
    }
    Ok(())
}

pub(crate) fn ensure_runhaven_cubby(codex_home: &Path) -> Result<()> {
    let pet_dir = codex_home.join("pets").join(RUNHAVEN_BUNDLED_CUBBY_ID);
    fs::create_dir_all(&pet_dir).with_context(|| format!("create {}", pet_dir.display()))?;
    write_bundled_file(&pet_dir.join("pet.json"), CUBBY_PET_JSON)?;
    write_bundled_file(&pet_dir.join("spritesheet.webp"), CUBBY_SPRITESHEET)?;
    Pet::load_with_codex_home(RUNHAVEN_BUNDLED_CUBBY_SELECTOR, Some(codex_home))
        .map(|_| ())
        .context("validate bundled Cubby pet")
}

fn write_bundled_file(path: &Path, bytes: &[u8]) -> Result<()> {
    match fs::read(path) {
        Ok(current) if current == bytes => return Ok(()),
        Ok(_) => {
            bail!(
                "bundled Cubby pet file already exists with different contents: {}",
                path.display()
            );
        }
        Err(err) if err.kind() == io::ErrorKind::NotFound => {}
        Err(err) => return Err(err).with_context(|| format!("read {}", path.display())),
    }

    let mut file = match OpenOptions::new().write(true).create_new(true).open(path) {
        Ok(file) => file,
        Err(err) if err.kind() == io::ErrorKind::AlreadyExists => {
            return write_bundled_file(path, bytes);
        }
        Err(err) => return Err(err).with_context(|| format!("create {}", path.display())),
    };

    if let Err(err) = file.write_all(bytes) {
        let _ = fs::remove_file(path);
        return Err(err).with_context(|| format!("write {}", path.display()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runhaven_cubby_materializes_codex_custom_pet_package() {
        let codex_home = tempfile::tempdir().unwrap();

        ensure_runhaven_cubby(codex_home.path()).unwrap();

        let pet_dir = codex_home
            .path()
            .join("pets")
            .join(RUNHAVEN_BUNDLED_CUBBY_ID);
        assert_eq!(fs::read(pet_dir.join("pet.json")).unwrap(), CUBBY_PET_JSON);
        assert_eq!(
            fs::read(pet_dir.join("spritesheet.webp")).unwrap(),
            CUBBY_SPRITESHEET
        );
        let pet =
            Pet::load_with_codex_home(RUNHAVEN_BUNDLED_CUBBY_SELECTOR, Some(codex_home.path()))
                .unwrap();
        assert_eq!(pet.id, "custom-runhaven-cubby");
        assert_eq!(pet.display_name, RUNHAVEN_BUNDLED_CUBBY_DISPLAY_NAME);
        assert_eq!(pet.frame_width, 192);
        assert_eq!(pet.frame_height, 208);
        assert_eq!(pet.columns, 8);
        assert_eq!(pet.rows, 9);
        assert_eq!(pet.frame_count(), 72);
    }

    #[test]
    fn runhaven_cubby_does_not_overwrite_existing_custom_pet() {
        let codex_home = tempfile::tempdir().unwrap();
        let pet_dir = codex_home
            .path()
            .join("pets")
            .join(RUNHAVEN_BUNDLED_CUBBY_ID);
        fs::create_dir_all(&pet_dir).unwrap();
        fs::write(pet_dir.join("pet.json"), b"different").unwrap();

        let err = ensure_runhaven_cubby(codex_home.path()).unwrap_err();

        assert!(
            err.to_string()
                .contains("already exists with different contents")
        );
        assert_eq!(fs::read(pet_dir.join("pet.json")).unwrap(), b"different");
    }
}
