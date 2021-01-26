use clap::Clap;
use figlet_rs::FIGfont;
use humantime::format_duration;
use parse_duration::parse;
use std::time::Duration;

use termion::input::TermRead;
use termion::raw::IntoRawMode;
use tokio::time;

#[derive(Clap)]
#[clap(version = "1.0", author = "Marcus B. <me@mbuffett.com>")]
struct Opts {
    #[clap(multiple(true))]
    input: String,
}

#[tokio::main]
async fn main() {
    let opts: Opts = Opts::parse();
    let input = opts.input;
    let standard_font = FIGfont::standand().unwrap();
    let duration = parse(&input).map(|d| chrono::Duration::from_std(d).unwrap());
    let (width, height) = termion::terminal_size().unwrap();
    let width = width as usize;
    let height = height as usize;
    let mut paused = false;
    let reset_terminal = || {
        println!("{}", termion::cursor::Show);
        print!("{}", termion::cursor::Goto(1, 1));
    };
    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        reset_terminal();
        std::process::exit(1);
    });
    let mut stdin = termion::async_stdin().keys();

    if let Ok(duration) = duration {
        print!("{}", termion::cursor::Hide);
        let mut end_time = chrono::Utc::now() + duration;
        let mut interval = time::interval(Duration::from_millis(10));
        let mut current_time = chrono::Utc::now();
        while end_time > chrono::Utc::now() {
            {
                let mut _stdout = std::io::stdout().into_raw_mode().unwrap();
                let input = stdin.next();
                if let Some(Ok(termion::event::Key::Char(' '))) = input {
                    paused = !paused;
                }
            }
            interval.tick().await;

            let elapsed = chrono::Utc::now() - current_time;
            current_time = chrono::Utc::now();
            if paused {
                end_time = end_time + elapsed;
            }
            let remaining_duration = end_time - chrono::Utc::now();
            let remaining_duration = Duration::from_secs(remaining_duration.num_seconds() as u64);
            let formatted_duration = format_duration(remaining_duration).to_string();
            let duration_figure = standard_font
                .convert(&formatted_duration)
                .unwrap()
                .to_string();
            let figure_height = duration_figure.lines().count();
            let figure_width = duration_figure.lines().map(|l| l.len()).max().unwrap();
            let padding_top: usize = height / 2 - figure_height / 2;
            let padding_left: usize = width / 2 - figure_width / 2;
            let duration_figure = "\n".repeat(padding_top) + &duration_figure;
            let duration_figure = duration_figure
                .lines()
                .map(|s| " ".repeat(padding_left) + s)
                .collect::<Vec<_>>()
                .join("\n");
            if paused {
                println!("{}", termion::color::Fg(termion::color::Cyan));
            } else {
                println!("{}", termion::color::Fg(termion::color::White));
            }
            print!(
                "{}{}{}",
                termion::clear::All,
                termion::cursor::Goto(1, 1),
                duration_figure
            );
            interval.tick().await;
        }
    }
}
