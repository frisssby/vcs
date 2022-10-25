# Version Control System

Simple command-line tool for local version control written in Rust.

## Installation
To get started using `vcs`, install it via
```bash
cargo install --git vcs_git_repository
```

### Requirements
To install the crate and build a binary, you'll need a recent version of `Rust`.
If you don't have it yet, you can get it following the [official instruction](https://www.rust-lang.org/tools/install).


## Commands

### Init
Initialise VCS repository

```
vcs init --path=<directory_path>
```

`<directory_path>` must exist in the filesystem and mustn't contain **.vcs** directory!

Creates a **.vcs** directory in the path provided with
**objects** and **refs/heads** subdirectories. Files **STATE** that contains information about current branch and commit and **index** that stores tracked file paths and their object names are also created.

Creates a commit with ***"Initial commit"*** message including all files in the provided directory (if any).


### Status

Show the working tree status

```
vcs status
```

Displays the branch name you are currently on and changes between the working
tree and the branch head commit tree.

### Commit
Record changes to the repository

```
vcs commit --message=<commit_message>
```

Creates a new commit from all the changes in the working tree with the given
message describing the changes. Committing is only possible from the current
branch head. The current branch head and STATE's current commit are updated to
point to a newly created commit.

The commit *belongs* to the branch it was created on.

### Jump

Switch branch or restore the working tree

```
vcs jump [--branch=<branch_name> | --commit=<commit_id>]
```

Updates files in the working tree so that they correspond to the specified
snapshot. STATE file is also updated by setting the specified branch as the
current branch *or* the specified commit as the current commit and the branch it
belongs to as the current branch.

Aborts if there are uncommitted changes in the working tree.  


### New Branch
Create a new branch 

```
vcs new_branch --branch=<branch_name>
```

Creates a new branch with the given name and jumps to it. Modifications to the
files in the working tree are kept. Branching is only possible from the MASTER
 head commit.

Aborts if a branch with the given name already exists.

### Merge

Merge commit history from a side branch into the master branch

```
vcs merge --branch=<branch_name>
```

Incorporates the committed changes from `<branch_name>` into MASTER branch. Creates a new commit including all changes from `<branch_name>`. `<branch_name>` and its commits are deleted after the successful merge. 

You're supposed to be on MASTER head to use this command.

Aborts if there are uncommitted changes. Aborts if there is a merge conflict (at least one file is modified both in MASTER and `<branch_name>` since its branch from MASTER).

### Log

Show commit logs

```
vcs log
```

Shows the commit logs in reverse chronological order starting with the current commit and ending with its branch root commit.