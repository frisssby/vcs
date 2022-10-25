use clap::{ArgGroup, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Initialise VCS repository
    ///
    /// Creates a .vcs directory in the path provided with objects and
    /// refs/heads subdirectories. Files STATE that contains information about
    /// current branch and commit and index that stores tracked file paths and
    /// their object names are also created.
    ///
    /// Creates a commit with "Initial commit" message including all files in
    /// the provided directory (if any).
    Init {
        #[arg(long, value_name = "directory_path")]
        path: PathBuf,
    },
    /// Show the working tree status
    ///
    /// Displays the branch name you are currently on and changes between the
    /// working tree and the branch head commit tree.
    Status,
    /// Record changes to the repository
    ///
    /// Creates a new commit from all the changes in the working tree with the
    /// given message describing the changes. Committing is only possible from
    /// the current branch head. The current branch head and STATE's current
    /// commit are updated to point to a newly created commit.
    ///
    /// The commit belongs to the branch it was created on.
    Commit {
        #[arg(long)]
        message: String,
    },
    /// Switch branch or restore the working tree
    ///
    /// Updates files in the working tree so that they correspond to the
    /// specified snapshot. STATE file is also updated by setting the
    /// specified branch as the current branch OR the specified commit as the
    /// current commit and the branch it belongs to as the current branch.
    ///
    /// Aborts if there are uncommitted changes in the working tree.
    #[command(group(
        ArgGroup::new("object")
            .required(true)
            .args(["branch", "commit"])
        ))]
    Jump {
        #[arg(long, value_name = "branch_name", conflicts_with = "commit")]
        branch: Option<String>,
        #[arg(long, value_name = "commit_hash")]
        commit: Option<String>,
    },
    /// Create a new branch
    ///
    /// Creates a new branch with the given name and jumps to it. Modifications
    /// to the files in the working tree are kept. Branching is only possible
    /// from the MASTER head commit.  
    #[command(name = "new_branch")]
    NewBranch {
        #[arg(long, value_name = "branch_name")]
        name: String,
    },
    /// Merge commit history from a side branch into the master branch
    ///
    /// Incorporates the committed changes from <branch_name> into MASTER
    /// branch. Creates a new commit including all changes from <branch_name>.
    /// <branch_name> and its commits are deleted after the successful merge.
    ///
    /// You're supposed to be on MASTER head to use this command.
    ///
    /// Aborts if there are uncommitted changes. Aborts if there is a merge
    /// conflict (at least one file is modified both in MASTER and <branch_name>
    /// since its branch from MASTER).
    Merge {
        #[arg(long, value_name = "branch_name")]
        branch: String,
    },
    /// Show commit logs
    ///
    /// Shows the commit logs in reverse chronological order starting with the
    /// current commit and ending with its branch root commit.
    Log,
}
