pub const MARKDOWN: &str = include_str!(env!("TICGIT_AGENTS_MD_PATH"));

pub fn print() {
    print!("{MARKDOWN}");
}
