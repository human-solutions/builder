use std::{
    fmt,
    fs::{self, OpenOptions},
    io::Write,
    str::FromStr,
};

const GIT_HOOKS_DIR: &str = ".git/hooks";

pub enum HookStage {
    /// https://git-scm.com/docs/githooks#_applypatch_msg
    ApplyPatchMsg,
    /// https://git-scm.com/docs/githooks#_pre_applypatch
    PreApplyPatch,
    /// https://git-scm.com/docs/githooks#_post_applypatch
    PostApplyPatch,
    /// https://git-scm.com/docs/githooks#_pre_commit
    PreCommit,
    /// https://git-scm.com/docs/githooks#_pre_merge_commit
    PreMergeCommit,
    /// https://git-scm.com/docs/githooks#_prepare_commit_msg
    PrepareCommitMsg,
    /// https://git-scm.com/docs/githooks#_commit_msg
    CommitMsg,
    /// https://git-scm.com/docs/githooks#_post_commit
    PostCommit,
    /// https://git-scm.com/docs/githooks#_pre_rebase
    PreRebase,
    /// https://git-scm.com/docs/githooks#_post_checkout
    PostCheckout,
    /// https://git-scm.com/docs/githooks#_post_merge
    PostMerge,
    /// https://git-scm.com/docs/githooks#_pre_push
    PrePush,
    /// https://git-scm.com/docs/githooks#_pre_receive
    PreReceive,
    /// https://git-scm.com/docs/githooks#_update
    Update,
    /// https://git-scm.com/docs/githooks#_post_receive
    ProcReceive,
    /// https://git-scm.com/docs/githooks#_post_receive
    PostReceive,
    /// https://git-scm.com/docs/githooks#_post_update
    PostUpdate,
    /// https://git-scm.com/docs/githooks#_reference_transaction
    ReferenceTransaction,
    /// https://git-scm.com/docs/githooks#_push_to_checkout
    PushToCheckout,
    /// https://git-scm.com/docs/githooks#_pre_auto_gc
    PreAutoGc,
    /// https://git-scm.com/docs/githooks#_post_rewrite
    PostRewrite,
    /// https://git-scm.com/docs/githooks#_sendemail_validate
    SendemailValidate,
    /// https://git-scm.com/docs/githooks#_fsmonitor_watchman
    FsmonitorWatchman,
    /// https://git-scm.com/docs/githooks#_p4_change_list,
    P4Changelist,
    /// https://git-scm.com/docs/githooks#_p4_prepare_changelist,
    P4PrepareChangelist,
    /// https://git-scm.com/docs/githooks#_p4_post_change_list,
    P4PostChangeList,
    /// https://git-scm.com/docs/githooks#_p4_pre_submit,
    P4PreSubmit,
    /// https://git-scm.com/docs/githooks#_post_index_change,
    PostIndexChange,
}

impl FromStr for HookStage {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "applypatch-msg" => Ok(Self::ApplyPatchMsg),
            "pre-applypatch" => Ok(Self::PreApplyPatch),
            "post-applypatch" => Ok(Self::PostApplyPatch),
            "pre-commit" => Ok(Self::PreCommit),
            "pre-merge-commit" => Ok(Self::PreMergeCommit),
            "prepare-commit-msg" => Ok(Self::PrepareCommitMsg),
            "commit-msg" => Ok(Self::CommitMsg),
            "post-commit" => Ok(Self::PostCommit),
            "pre-rebase" => Ok(Self::PreRebase),
            "post-checkout" => Ok(Self::PostCheckout),
            "post-merge" => Ok(Self::PostMerge),
            "pre-push" => Ok(Self::PrePush),
            "pre-receive" => Ok(Self::PreReceive),
            "update" => Ok(Self::Update),
            "proc-receive" => Ok(Self::ProcReceive),
            "post-receive" => Ok(Self::PostReceive),
            "post-update" => Ok(Self::PostUpdate),
            "reference-transaction" => Ok(Self::ReferenceTransaction),
            "push-to-checkout" => Ok(Self::PushToCheckout),
            "pre-auto-gc" => Ok(Self::PreAutoGc),
            "post-rewrite" => Ok(Self::PostRewrite),
            "sendemail-validate" => Ok(Self::SendemailValidate),
            "fsmonitor-watchman" => Ok(Self::FsmonitorWatchman),
            "p4-changelist" => Ok(Self::P4Changelist),
            "p4-prepare-changelist" => Ok(Self::P4PrepareChangelist),
            "p4-post-change-list" => Ok(Self::P4PostChangeList),
            "p4-pre-submit" => Ok(Self::P4PreSubmit),
            "post-index-change" => Ok(Self::PostIndexChange),
            _ => Err(format!("invalid hook: {}", s)),
        }
    }
}

impl fmt::Display for HookStage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::ApplyPatchMsg => write!(f, "applypatch-msg"),
            Self::PreApplyPatch => write!(f, "pre-applypatch"),
            Self::PostApplyPatch => write!(f, "post-applypatch"),
            Self::PreCommit => write!(f, "pre-commit"),
            Self::PreMergeCommit => write!(f, "pre-merge-commit"),
            Self::PrepareCommitMsg => write!(f, "prepare-commit-msg"),
            Self::CommitMsg => write!(f, "commit-msg"),
            Self::PostCommit => write!(f, "post-commit"),
            Self::PreRebase => write!(f, "pre-rebase"),
            Self::PostCheckout => write!(f, "post-checkout"),
            Self::PostMerge => write!(f, "post-merge"),
            Self::PrePush => write!(f, "pre-push"),
            Self::PreReceive => write!(f, "pre-receive"),
            Self::Update => write!(f, "update"),
            Self::ProcReceive => write!(f, "proc-receive"),
            Self::PostReceive => write!(f, "post-receive"),
            Self::PostUpdate => write!(f, "post-update"),
            Self::ReferenceTransaction => write!(f, "reference-transaction"),
            Self::PushToCheckout => write!(f, "push-to-checkout"),
            Self::PreAutoGc => write!(f, "pre-auto-gc"),
            Self::PostRewrite => write!(f, "post-rewrite"),
            Self::SendemailValidate => write!(f, "sendemail-validate"),
            Self::FsmonitorWatchman => write!(f, "fsmonitor-watchman"),
            Self::P4Changelist => write!(f, "p4-changelist"),
            Self::P4PrepareChangelist => write!(f, "p4-prepare-changelist"),
            Self::P4PostChangeList => write!(f, "p4-post-change-list"),
            Self::P4PreSubmit => write!(f, "p4-pre-submit"),
            Self::PostIndexChange => write!(f, "post-index-change"),
        }
    }
}

pub struct Hook {
    pub stage: HookStage,
    pub script: String,
}

impl fmt::Display for Hook {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}\n{}\n{}",
            self.start_tag(),
            self.script,
            self.end_tag()
        )
    }
}

impl Hook {
    pub fn start_tag(&self) -> String {
        format!("### CUSTOM {} script\n", self.stage)
    }
    pub fn end_tag(&self) -> String {
        format!("### END CUSTOM {} script\n", self.stage)
    }

    pub fn git_path(&self) -> String {
        format!("{GIT_HOOKS_DIR}/{}", self.stage)
    }

    /// returns the existing script if any
    pub fn is_installed(&self) -> Option<String> {
        let git_script_content =
            fs::read_to_string(self.git_path()).expect("failed to read git script file");

        if let Some(script) = git_script_content.split(&self.start_tag()).next() {
            return Some(script.to_string());
        };
        None
    }

    pub fn install(&self, create: bool) -> Result<(), String> {
        let existing_script = if !create {
            if let Some(script) = self.is_installed() {
                format!("{}\n", script.replace("#!/bin/sh\n", "").trim())
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        let mut git_script = OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(create)
            .open(self.git_path())
            .unwrap();

        let bash_tag = "#!/bin/sh";

        writeln!(git_script, "{}\n{}\n{}", bash_tag, existing_script, self)
            .map_err(|e| format!("failed to install {} script due to : {e}", self.stage))
    }
}
