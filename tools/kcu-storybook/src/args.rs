#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct StorybookArgs {
    pub(crate) smoke: bool,
}

impl StorybookArgs {
    pub(crate) fn parse(mut values: impl Iterator<Item = String>) -> Self {
        match (values.next().as_deref(), values.next()) {
            (None, None) => Self { smoke: false },
            (Some("--smoke"), None) => Self { smoke: true },
            _ => usage(),
        }
    }
}

fn usage() -> ! {
    eprintln!("usage: kcu-storybook [--smoke]");
    std::process::exit(2)
}

#[cfg(test)]
mod tests {
    use super::StorybookArgs;

    #[test]
    fn parses_interactive_and_smoke_modes() {
        assert_eq!(
            StorybookArgs::parse(Vec::<String>::new().into_iter()),
            StorybookArgs { smoke: false }
        );
        assert_eq!(
            StorybookArgs::parse(vec!["--smoke".to_string()].into_iter()),
            StorybookArgs { smoke: true }
        );
    }
}
