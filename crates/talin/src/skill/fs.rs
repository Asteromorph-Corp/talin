use std::borrow::Cow;
use std::ffi::OsString;
use std::path::{Component, Path, PathBuf};

use async_trait::async_trait;
use tokio::fs;
use tokio::sync::OnceCell;

use crate::skill::{SKILL_FILE, Skill, SkillError, SkillMetadata};

const CARGO_MANIFEST_DIR: &str = "CARGO_MANIFEST_DIR";
const SKILL_HOME_ENV_VAR: &str = "TALIN_SKILLS_HOME";
const SKILL_HOME_DEFAULT: &str = "skills";

/// A [`Skill`] in a directory on the local filesystem.
#[derive(Debug)]
pub struct FsSkill {
    /// The name of the directory.
    name: String,
    location: Location,
    loaded: OnceCell<Loaded>,
}

/// Errors specific to filesystem skills.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum FsSkillError {
    /// A filesystem operation failed.
    #[error("filesystem error at `{}`: {source}", path.display())]
    Io {
        /// The path the operation was performed on.
        path: PathBuf,
        /// The underlying I/O error.
        #[source]
        source: std::io::Error,
    },

    /// The skill name does not match the name of its directory.
    #[error("skill name `{name}` does not match its directory name `{dir}`")]
    NameMismatch {
        /// The name declared in the frontmatter.
        name: String,
        /// The name of the containing directory.
        dir: String,
    },

    /// The skill is not in any of the directories it was searched for in.
    #[error("skill `{name}` was not found in any of: {}", list(searched))]
    NotFound {
        /// The name of the skill.
        name: String,
        /// The directories that were searched.
        searched: Vec<PathBuf>,
    },

    /// A requested file path is absolute or resolves outside the skill directory.
    #[error("path `{0}` is outside the skill directory")]
    PathEscapesRoot(String),
}

/// Where the directory of an [`FsSkill`] is.
#[derive(Debug)]
enum Location {
    /// Somewhere in the skills homes, found by the name of the skill when it is first used.
    Home,
    /// The given path.
    Path(PathBuf),
}

#[derive(Debug)]
struct Loaded {
    /// The canonical path of the directory.
    root: PathBuf,
    metadata: SkillMetadata,
    instructions: String,
}

impl FsSkill {
    /// Creates a skill for the directory `dir`, without reading it.
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        let dir = dir.into();
        let name = dir
            .file_name()
            .map_or_else(|| dir.to_string_lossy(), |name| name.to_string_lossy())
            .into_owned();
        Self {
            name,
            location: Location::Path(dir),
            loaded: OnceCell::new(),
        }
    }

    /// Creates a skill called `name` that is in the skills homes:
    /// - If the environment variable `TALIN_SKILLS_HOME` is set, the specified path is used as a
    ///   skill home. Multiple paths can be specified by separating them with the platform-specific
    ///   path separator (`:` on Unix, `;` on Windows). When specifying multiple paths, the paths
    ///   are looked up in order and the first skill is used.
    /// - When running with `cargo run` or `cargo test`, the skill home is `skills` in the manifest
    ///   directory (`CARGO_MANIFEST_DIR`).
    /// - Otherwise, the skill home is `skills` in the current working directory.
    pub fn in_home(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            location: Location::Home,
            loaded: OnceCell::new(),
        }
    }

    /// Creates a skill for the directory `dir`, and reads and validates it right away.
    pub async fn load(self) -> Result<Self, SkillError> {
        self.loaded().await?;
        Ok(self)
    }

    async fn loaded(&self) -> Result<&Loaded, SkillError> {
        self.loaded.get_or_try_init(|| self.read()).await
    }

    async fn find_dir(&self) -> Result<Cow<'_, Path>, SkillError> {
        match &self.location {
            Location::Path(dir) => Ok(Cow::Borrowed(dir)),
            Location::Home => search(&skills_homes(), &self.name).await.map(Cow::Owned),
        }
    }

    async fn read(&self) -> Result<Loaded, SkillError> {
        let dir = self.find_dir().await?;
        let dir = dir.as_ref();
        let file = dir.join(SKILL_FILE);
        let markdown = fs::read_to_string(&file)
            .await
            .map_err(|e| FsSkillError::io(&file, e))?;
        let root = fs::canonicalize(dir)
            .await
            .map_err(|e| FsSkillError::io(dir, e))?;

        let (metadata, body) = SkillMetadata::from_markdown(&markdown)?;

        let dir_name = root
            .file_name()
            .map_or_else(|| root.to_string_lossy(), |name| name.to_string_lossy());
        if dir_name != metadata.name() {
            return Err(SkillError::from(FsSkillError::NameMismatch {
                name: metadata.name().to_owned(),
                dir: dir_name.into_owned(),
            }));
        }

        Ok(Loaded {
            root,
            metadata,
            instructions: body.trim().to_owned(),
        })
    }
}

#[async_trait]
impl Skill for FsSkill {
    fn name(&self) -> &str {
        &self.name
    }

    async fn metadata(&self) -> Result<SkillMetadata, SkillError> {
        Ok(self.loaded().await?.metadata.clone())
    }

    async fn instructions(&self) -> Result<String, SkillError> {
        Ok(self.loaded().await?.instructions.clone())
    }

    async fn files(&self) -> Result<Vec<String>, SkillError> {
        let root = &self.loaded().await?.root;
        let mut files = Vec::new();
        let mut stack = vec![root.clone()];
        while let Some(dir) = stack.pop() {
            let mut entries = fs::read_dir(&dir)
                .await
                .map_err(|e| FsSkillError::io(&dir, e))?;
            while let Some(entry) = entries
                .next_entry()
                .await
                .map_err(|e| FsSkillError::io(&dir, e))?
            {
                let path = entry.path();
                let is_dir = fs::metadata(&path)
                    .await
                    .map_err(|e| FsSkillError::io(&path, e))?
                    .is_dir();
                if is_dir {
                    stack.push(path);
                } else if let Ok(relative) = path.strip_prefix(root)
                    && relative != Path::new(SKILL_FILE)
                {
                    files.push(
                        relative
                            .components()
                            .map(|c| c.as_os_str().to_string_lossy())
                            .collect::<Vec<_>>()
                            .join("/"),
                    );
                }
            }
        }
        files.sort();
        Ok(files)
    }

    /// `path` must stay inside the skill directory: absolute paths, `..` components, and symlinks
    /// that resolve outside the directory are rejected.
    async fn read_file(&self, relative: &str) -> Result<String, SkillError> {
        let escapes = || SkillError::from(FsSkillError::PathEscapesRoot(relative.to_owned()));

        let requested = Path::new(relative);
        if !requested
            .components()
            .all(|c| matches!(c, Component::Normal(_) | Component::CurDir))
        {
            return Err(escapes());
        }

        let root = &self.loaded().await?.root;
        let path = root.join(requested);
        let resolved = fs::canonicalize(&path)
            .await
            .map_err(|e| FsSkillError::io(&path, e))?;
        if !resolved.starts_with(root) {
            return Err(escapes());
        }
        fs::read_to_string(&resolved)
            .await
            .map_err(|e| FsSkillError::io(&resolved, e))
    }

    fn notes(&self) -> Option<String> {
        // Only the loaded skill knows its canonical path, and notes are only read once it has been
        // used.
        let root = &self.loaded.get()?.root;
        Some(format!(
            "Relative paths in this skill are relative to `{}`.",
            root.display()
        ))
    }
}

impl FsSkillError {
    fn io(path: &Path, source: std::io::Error) -> SkillError {
        SkillError::from(Self::Io {
            path: path.to_path_buf(),
            source,
        })
    }
}

impl From<FsSkillError> for SkillError {
    fn from(err: FsSkillError) -> Self {
        SkillError::Source(Box::new(err))
    }
}

fn list(paths: &[PathBuf]) -> String {
    paths
        .iter()
        .map(|path| format!("`{}`", path.display()))
        .collect::<Vec<_>>()
        .join(", ")
}

/// The directories that [`FsSkill::in_home`](crate::skill::FsSkill::in_home) looks for skills in.
fn skills_homes() -> Vec<PathBuf> {
    homes_from(
        std::env::var_os(SKILL_HOME_ENV_VAR),
        std::env::var_os(CARGO_MANIFEST_DIR),
    )
}

fn homes_from(homes: Option<OsString>, manifest_dir: Option<OsString>) -> Vec<PathBuf> {
    let homes = homes
        .iter()
        .flat_map(std::env::split_paths)
        .filter(|home| !home.as_os_str().is_empty())
        .collect::<Vec<_>>();
    if !homes.is_empty() {
        return homes;
    }
    match manifest_dir {
        Some(dir) => vec![PathBuf::from(dir).join(SKILL_HOME_DEFAULT)],
        None => vec![PathBuf::from(SKILL_HOME_DEFAULT)],
    }
}

/// The directory of the skill `name` in the first of the `homes` that has it.
async fn search(homes: &[PathBuf], name: &str) -> Result<PathBuf, SkillError> {
    let dirs = homes.iter().map(|home| home.join(name)).collect::<Vec<_>>();
    for dir in &dirs {
        if fs::try_exists(dir.join(SKILL_FILE)).await.unwrap_or(false) {
            return Ok(dir.clone());
        }
    }
    Err(SkillError::from(FsSkillError::NotFound {
        name: name.to_owned(),
        searched: dirs,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn write_skill(parent: &Path, name: &str, header: &str) -> PathBuf {
        let dir = parent.join(name);
        fs::create_dir_all(dir.join("references")).await.unwrap();
        fs::write(
            dir.join(SKILL_FILE),
            format!("---\n{header}\n---\n# Body\nDo it.\n"),
        )
        .await
        .unwrap();
        fs::write(dir.join("references/guide.md"), "guide")
            .await
            .unwrap();
        dir
    }

    fn homes(homes: Option<&str>, manifest_dir: Option<&str>) -> Vec<PathBuf> {
        homes_from(homes.map(OsString::from), manifest_dir.map(OsString::from))
    }

    async fn tempdir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("pe-fs-skill-{}", uuid::Uuid::new_v4()));
        fs::create_dir_all(&dir).await.unwrap();
        dir
    }

    /// The filesystem error behind a [`SkillError::Source`], if that is what it is.
    fn fs_error(err: &SkillError) -> Option<&FsSkillError> {
        match err {
            SkillError::Source(source) => source.downcast_ref(),
            _ => None,
        }
    }

    #[test]
    fn homes_at_specified_paths_with_env_var() {
        let var = std::env::join_paths(["/a", "/b"]).unwrap();
        let list = homes_from(Some(var), Some("/project".into()));
        assert_eq!(list, [PathBuf::from("/a"), PathBuf::from("/b")]);
    }

    #[test]
    fn homes_at_package_or_cwd_without_env_var() {
        assert_eq!(
            homes(None, Some("/project")),
            [PathBuf::from("/project/skills")]
        );
        assert_eq!(homes(None, None), [PathBuf::from("skills")]);
        // An empty variable counts as unset.
        assert_eq!(homes(Some(""), None), [PathBuf::from("skills")]);
    }

    #[tokio::test]
    async fn search_uses_first_home_with_skill() {
        let tmp = tempdir().await;
        let (first, second, third) = (tmp.join("first"), tmp.join("second"), tmp.join("third"));
        write_skill(&second, "pdf", "name: pdf\ndescription: from second").await;
        write_skill(&third, "pdf", "name: pdf\ndescription: from third").await;

        let dir = search(&[first.clone(), second.clone(), third], "pdf")
            .await
            .unwrap();
        assert_eq!(dir, second.join("pdf"));

        // A skill the homes do not have names every directory it was looked for in.
        let err = search(&[first, second], "xlsx").await.unwrap_err();
        assert!(matches!(
            fs_error(&err),
            Some(FsSkillError::NotFound { .. })
        ));
        let message = err.to_string();
        assert!(message.contains("xlsx"), "{message}");
        assert!(message.contains("first/xlsx"), "{message}");
        assert!(message.contains("second/xlsx"), "{message}");
        fs::remove_dir_all(tmp).await.unwrap();
    }

    #[tokio::test]
    async fn loads_skill() {
        let tmp = tempdir().await;
        let dir = write_skill(&tmp, "pdf", "name: pdf\ndescription: Handle PDFs").await;
        let skill = FsSkill::new(&dir).load().await.unwrap();

        assert_eq!(skill.name(), "pdf");
        assert_eq!(skill.metadata().await.unwrap().description(), "Handle PDFs");
        assert_eq!(skill.instructions().await.unwrap(), "# Body\nDo it.");
        assert_eq!(skill.files().await.unwrap(), ["references/guide.md"]);
        let canonical = fs::canonicalize(&dir).await.unwrap();
        assert!(
            skill
                .notes()
                .unwrap()
                .contains(&canonical.to_string_lossy().into_owned())
        );
        fs::remove_dir_all(tmp).await.unwrap();
    }

    #[tokio::test]
    async fn lookup_on_first_use() {
        let tmp = tempdir().await;
        let skill = FsSkill::new(tmp.join("missing"));
        assert_eq!(skill.name(), "missing");
        // No notes until the skill has been read.
        assert!(skill.notes().is_none());

        let err = skill.metadata().await.unwrap_err();
        assert!(matches!(fs_error(&err), Some(FsSkillError::Io { .. })));
        assert!(err.to_string().contains("missing"));

        // Once the directory exists, the failure was not cached.
        write_skill(&tmp, "missing", "name: missing\ndescription: d").await;
        assert_eq!(skill.metadata().await.unwrap().name(), "missing");
        fs::remove_dir_all(tmp).await.unwrap();
    }

    #[tokio::test]
    async fn read_once() {
        let tmp = tempdir().await;
        let dir = write_skill(&tmp, "pdf", "name: pdf\ndescription: d").await;
        let skill = FsSkill::new(&dir).load().await.unwrap();

        // Later changes to the file are not picked up.
        fs::write(
            dir.join(SKILL_FILE),
            "---\nname: pdf\ndescription: changed\n---\nnew body",
        )
        .await
        .unwrap();
        assert_eq!(skill.metadata().await.unwrap().description(), "d");
        assert_eq!(skill.instructions().await.unwrap(), "# Body\nDo it.");
        fs::remove_dir_all(tmp).await.unwrap();
    }

    #[tokio::test]
    async fn rejects_name_directory_mismatch() {
        let tmp = tempdir().await;
        let dir = write_skill(&tmp, "other", "name: pdf\ndescription: d").await;
        let err = FsSkill::new(&dir).load().await.unwrap_err();
        assert!(matches!(
            fs_error(&err),
            Some(FsSkillError::NameMismatch { .. })
        ));
        fs::remove_dir_all(tmp).await.unwrap();
    }

    #[tokio::test]
    async fn read_file_stays_inside_root() {
        let tmp = tempdir().await;
        let dir = write_skill(&tmp, "pdf", "name: pdf\ndescription: d").await;
        fs::write(tmp.join("secret.txt"), "secret").await.unwrap();
        let skill = FsSkill::new(&dir).load().await.unwrap();

        assert_eq!(
            skill.read_file("references/guide.md").await.unwrap(),
            "guide"
        );
        for path in ["../secret.txt", "/etc/passwd"] {
            let err = skill.read_file(path).await.unwrap_err();
            assert!(
                matches!(fs_error(&err), Some(FsSkillError::PathEscapesRoot(_))),
                "{path} gave {err}"
            );
        }
        fs::remove_dir_all(tmp).await.unwrap();
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn read_file_rejects_escaping_symlink() {
        let tmp = tempdir().await;
        let dir = write_skill(&tmp, "pdf", "name: pdf\ndescription: d").await;
        fs::write(tmp.join("secret.txt"), "secret").await.unwrap();
        std::os::unix::fs::symlink(tmp.join("secret.txt"), dir.join("link.txt")).unwrap();
        let skill = FsSkill::new(&dir).load().await.unwrap();

        let err = skill.read_file("link.txt").await.unwrap_err();
        assert!(matches!(
            fs_error(&err),
            Some(FsSkillError::PathEscapesRoot(_))
        ));
        fs::remove_dir_all(tmp).await.unwrap();
    }
}
