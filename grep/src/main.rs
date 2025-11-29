use std::io::{IsTerminal, stdin};

use grep::{AppConfig, MatchReporter, PatternSearcher, Result, create_grep_command};

fn main() -> Result<()> {
    let command = create_grep_command();
    let args = command.get_matches();
    let AppConfig {
        input,
        regex,
        output,
        format,
    } = AppConfig::from_args(&args)?;

    {
        // .lock in nested scope to release lock automatically when not used
        let reader = stdin().lock();
        if reader.is_terminal() {
            return MatchReporter::report_matches_in_terminal(reader, &regex);
        }
    }

    let searcher = PatternSearcher::try_new(input)?;
    let matches = searcher.find_matches(&regex)?;

    let mut reporter = MatchReporter::try_new(output, format)?;
    reporter.report(matches)?;

    Ok(())
}
