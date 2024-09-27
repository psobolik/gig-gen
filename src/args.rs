#[derive(Debug, clap::Args)]
pub(super) struct FilterArgs {
    /// Filter to apply to list of templates
    pub(super) filter: Option<String>,
}

#[derive(Debug, clap::Args)]
pub(super) struct TemplateArgs {
    /// One or more gitignore templates
    #[arg(name = "template", required = true)]
    pub(super) templates: Vec<String>,
}

#[derive(Debug, clap::Subcommand)]
pub(super) enum Commands {
    /// List available template names with an optional filter.
    List(FilterArgs),
    /// Generate a `.gitignore` file from templates.
    Generate(TemplateArgs),
    /// Pick templates interactively and generate a .gitignore file (default).
    Interactive,
}

#[derive(Debug, clap::Parser)]
#[command(version, about, long_about = None)]
pub(super) struct Args {
    /// Optional subcommand
    #[clap(subcommand)]
    pub(super) command: Option<Commands>,
}
