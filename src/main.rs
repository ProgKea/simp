extern crate termion;

use std::io::{stdin, stdout, Stdin, Stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;

use anyhow::Result;
use std::env;
use std::fs;
use std::process::exit;

fn parse_args() -> String {
    let args = env::args().map(|x| x.to_string()).collect::<Vec<String>>();
    if args.len() <= 1 || !fs::metadata(&args[1]).is_ok() {
        eprintln!("No file path was provided");
        exit(1);
    }

    return args.iter().nth(1).unwrap().to_string();
}

#[derive(Debug)]
struct Pager {
    content: String,
    content_size: usize,
    term_height: u16,
    shown: String,
    index: usize,
}

impl Pager {
    fn new() -> Result<Self> {
        let file = parse_args();
        let content = fs::read_to_string(file)?.replace("\n", "\r\n");
        let content_size = content.lines().count();
        return Ok(Self {
            content,
            content_size,
            term_height: termion::terminal_size()?.1,
            shown: String::new(),
            index: 1,
        });
    }

    fn get_shown(&self) -> String {
        let mut shown = String::new();
        for (i, line) in self.content.lines().enumerate() {
            if i >= self.index && i < self.term_height as usize + self.index {
                let mut s = line.to_string();
                s.push_str("\r\n");
                shown.push_str(&s);
            }
        }

        return shown;
    }

    fn scroll_down(&mut self) {
        if self.index < self.content_size - self.term_height as usize {
            self.index += 1;
        }
        self.shown = self.get_shown();
    }

    fn scroll_up(&mut self) {
        if self.index > 1 {
            self.index -= 1;
        }
        self.shown = self.get_shown();
    }

    fn run_pager(&mut self, stdin: Stdin, stdout: &mut Stdout) -> Result<()> {
        for c in stdin.keys() {
            write!(
                stdout,
                "{}{}",
                termion::cursor::Goto(1, 1),
                termion::clear::All,
            )?;
            match c? {
                Key::Esc | Key::Char('q') => break,
                Key::Down | Key::Char('j') => {
                    self.scroll_down();
                }
                Key::Up | Key::Char('k') => {
                    self.scroll_up();
                }
                _ => {}
            }
            write!(stdout, "{}", self.shown)?;
            stdout.flush().unwrap();
        }

        write!(stdout, "{}", termion::cursor::Show,).unwrap();
        return Ok(());
    }

    fn run(&mut self, stdin: Stdin, stdout: &mut Stdout) -> Result<()> {
        if self.content.lines().collect::<Vec<&str>>().len() <= self.term_height as usize {
            write!(stdout, "{}", self.content)?;
            stdout.flush()?;
        } else {
            write!(
                stdout,
                "{}{}{}{}",
                termion::clear::All,
                termion::cursor::Goto(1, 1),
                termion::cursor::Hide,
                self.get_shown()
            )?;
            stdout.flush()?;

            self.run_pager(stdin, stdout)?;
        }
        return Ok(());
    }
}

fn main() -> Result<()> {
    let mut pager = Pager::new()?;
    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode()?;

    pager.run(stdin, &mut stdout)?;

    return Ok(());
}
