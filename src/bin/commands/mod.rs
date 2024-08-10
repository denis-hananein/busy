use clap::{Args, Subcommand};

#[derive(Subcommand)]
pub enum Commands {
  /// Start a new task
  Start(Start),
  /// Stop the current task
  Stop,
  /// Pause the current task
  Pause,
  /// Resume the last task
  Resume,
  /// Continue a stopped task. It will start a new task with the same title and tags
  Continue(Continue),

  /// List all tags
  Tags,
  /// List all projects
  Projects,

  /// Add a new task
  Add(Add),
  /// Remove a task
  #[clap(alias = "rm")]
  Remove(Remove),
  /// Edit a task
  Edit(Edit),

  /// Show the current tasks status
  Status,
  /// Show the log
  Log(Log),
  /// Show toady's log. Shortcut for `log --today`
  #[clap(alias = "td")]
  Today(Today),
  /// Show the stat
  Stat(Stat),

  /// Sync with remote. To use remote repo you need to set the `BUSY_REMOTE` env variable
  Sync(Sync),

  Complete(clap_complete::dynamic::CompleteArgs),
}

#[derive(Debug, Args)]
pub struct Add {
  /// Project name
  pub project_name: String,
  /// Task title
  pub task_title: String,
  /// Tags
  pub tags: Vec<String>,
  /// Start time in format: HH:MM or YYYY-mm-dd HH:MM
  pub start_time: String,
  /// Finish time in format: HH:MM or YYYY-mm-dd HH:MM
  pub finish_time: String,
}

#[derive(Debug, Args)]
pub struct Remove {
  /// Task id
  pub short_task_id: String,
}

#[derive(Debug, Args)]
pub struct Edit {
  #[clap(long)]
  pub all: bool,
  #[clap(long)]
  pub all_tags: bool,
  #[clap(long)]
  pub task_id: Option<Vec<String>>,
  #[clap(long)]
  pub project_id: Option<Vec<String>>,
  #[clap(long)]
  pub tag_id: Option<Vec<String>>,
}

#[derive(Debug, Args)]
pub struct Start {
  /// Project name
  pub project_name: String,
  /// Task title
  pub task_title: String,
  /// Tags
  pub tags: Vec<String>,
  /// Override start time in format: HH:MM or YYYY-mm-dd HH:MM
  #[clap(short, long)]
  pub start_time: Option<String>,
}

#[derive(Debug, Args)]
pub struct Continue {
  pub short_task_id: String,
}

#[derive(Debug, Args)]
pub struct Today {
  #[clap(flatten)]
  pub log_params: LogCommonParams,
}

#[derive(Debug, Args)]
pub struct Log {
  #[clap(long)]
  pub days: Option<i64>,
  #[clap(long)]
  pub today: bool,
  #[clap(flatten)]
  pub log_params: LogCommonParams,
}

#[derive(Debug, Args, Clone)]
pub struct LogCommonParams {
  #[clap(long)]
  pub dont_clear: bool,
  #[clap(long)]
  pub project: Vec<String>,
  #[clap(long)]
  pub tag: Vec<String>,
  #[clap(long)]
  pub full: bool,
}

#[derive(Debug, Args)]
pub struct Stat {
  #[clap(long)]
  pub days: Option<i64>,
  #[clap(long)]
  pub today: bool,
  #[clap(long)]
  pub with_tags: bool,
  #[clap(flatten)]
  pub log_params: LogCommonParams,
}

#[derive(Debug, Args)]
pub struct Sync {
  #[clap(long)]
  pub push_force: bool,
  #[clap(long)]
  pub pull_force: bool,
}
