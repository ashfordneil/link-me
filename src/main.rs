use std::{
    env,
    fmt::Write as _,
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::{Context, Result};
use git2::{BranchType, Repository};
use git_url_parse::GitUrl;
use structopt::StructOpt;

/// Get a shareable link to a section of source code.
#[derive(Debug, StructOpt)]
struct Arguments {
    /// Which file do you want to link to?
    file_path: PathBuf,

    /// Which line in that file do you want to link to?
    #[structopt(short, long)]
    line_number: Option<usize>,

    /// Which way do you want to link to this? Options are "branch" and "commit".
    #[structopt(short, long)]
    ref_type: RefType,
}

/// Different ways to refer to a git reference.
#[derive(Debug)]
enum RefType {
    Branch,
    Commit,
}

impl FromStr for RefType {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "branch" => Ok(RefType::Branch),
            "commit" => Ok(RefType::Commit),
            _ => Err(r#"Unsupported ref type, acceptable values are "branch" and "commit""#),
        }
    }
}

#[derive(Debug)]
struct UrlInfo<'a> {
    repository_path: &'a str,
    reference: String,
    file_path: &'a Path,
    line_number: Option<usize>,
}

impl<'a> UrlInfo<'a> {
    fn github_url(&self) -> Result<String> {
        let mut output = format!(
            "https://github.com/{repository_path}/blob/{reference}/{file_path}",
            repository_path = self.repository_path,
            reference = self.reference,
            file_path = self
                .file_path
                .to_str()
                .context("File path is not valid UTF-8")?
        );
        if let Some(line_number) = self.line_number {
            write!(output, "#L{}", line_number)?;
        }

        Ok(output)
    }
}

fn get_reference(repository: &Repository, ref_type: RefType) -> Result<String> {
    match ref_type {
        RefType::Commit => {
            let head = repository.revparse_single("HEAD")?;
            let last_commit = head.peel_to_commit()?;
            let sha = last_commit.id();

            let mut output = String::new();
            for byte in sha.as_bytes() {
                write!(&mut output, "{:0X}", byte)?;
            }
            Ok(output)
        }
        RefType::Branch => {
            let branches = repository.branches(Some(BranchType::Local))?;
            for branch in branches {
                let (branch, _branch_type) = branch?;
                if branch.is_head() {
                    let name = branch.name()?.context("Branch name not valid UTF-8")?;
                    return Ok(name.to_string());
                }
            }
            anyhow::bail!(r#"No branch points to the current HEAD. Try using "commit""#);
        }
    }
}

fn main() -> Result<()> {
    let args = Arguments::from_args();
    let repository = Repository::discover(".")?;

    let origin = repository.find_remote("origin")?;
    let url = GitUrl::parse(origin.url().context("Origin URL was not valid UTF-8")?)?;
    let repository_path = &url.fullname;

    let reference = get_reference(&repository, args.ref_type)?;

    if !args.file_path.exists() {
        anyhow::bail!("The referenced file does not exist.");
    }

    let git_workdir = repository.workdir().context("Unexpected bare git repo.")?;
    let full_file_path = {
        let mut pwd = env::current_dir()?;
        pwd.push(&args.file_path);
        pwd
    };
    let file_path = full_file_path.strip_prefix(git_workdir)?;

    let info = UrlInfo {
        repository_path,
        reference,
        file_path,
        line_number: args.line_number,
    };

    let url = match url.host.context("No host found in origin URL")?.as_str() {
        "github" | "github.com" => info.github_url()?,
        _ => anyhow::bail!("Only github origins are supported right now"),
    };

    println!("{}", url);

    Ok(())
}
