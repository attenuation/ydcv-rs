//! main module of ydcv-rs
extern crate rustc_serialize;

#[macro_use] extern crate log;
extern crate env_logger;
extern crate getopts;
extern crate rustyline;
extern crate libc;
extern crate url;
extern crate hyper;
#[cfg(feature="notify-rust")] extern crate notify_rust;

use rustyline::Editor;
use libc::isatty;
pub use hyper::Client;


pub mod ydresponse;
pub mod ydclient;
pub mod formatters;

use ydclient::YdClient;
use formatters::{Formatter, PlainFormatter, AnsiFormatter, HtmlFormatter};


fn lookup_explain(client: &mut Client, word: &str, fmt: &mut Formatter, raw: bool){
    if raw {
        println!("{}", client.lookup_word(word, true).unwrap().raw_result());
    }else{
        match client.lookup_word(word, false){
            Ok(ref result) => {
                let exp = result.explain(fmt);
                fmt.print(word, &exp);
            }
            Err(err) => fmt.print(word,
                &format!("Error looking-up word {}: {:?}", word, err))
        }
    }
}

fn get_clipboard() -> String {
    if let Ok(out) = std::process::Command::new("xsel").arg("-o").output() {
        if let Ok(result) = String::from_utf8(out.stdout) {
            return result;
        }
    }
    "".to_string()
}


#[allow(dead_code)]
fn main() {
    env_logger::init().unwrap();

    let args: Vec<String> = std::env::args().collect();
    let mut opts = getopts::Options::new();
    opts.optflag("h", "help", "print this help menu");
    opts.optflag("x", "selection", "show explaination of current selection");
    opts.optflag("H", "html", "HTML-style output");
    opts.optflag("n", "notify", "send desktop notifications (implies -H)");
    opts.optflag("r", "raw", "dump raw json reply from server");
    opts.optopt("c", "color", "[auto, always, never] use color", "auto");

    let matches = match opts.parse(&args[1..]){
        Ok(m) => m,
        Err(f) => panic!(f.to_owned())
    };

    if matches.opt_present("h") {
        let brief = format!("Usage: {} [options] words", args[0]);
        print!("{}", opts.usage(&brief));
        return;
    }

    let mut client = Client::new();

    let mut html = HtmlFormatter::new(matches.opt_present("n"));
    let mut ansi = AnsiFormatter;
    let mut plain = PlainFormatter;

    let fmt :&mut Formatter = if matches.opt_present("H") || matches.opt_present("n") {
        &mut html
    } else if let Some(c) = matches.opt_str("c") {
        if c == "always" || unsafe{ isatty(1) == 1 } && c != "never" {
            &mut ansi
        } else {
            &mut plain
        }
    } else if unsafe { isatty(1) == 1 } {
        &mut ansi
    } else {
        &mut plain
    };

    let raw = matches.opt_present("r");

    if matches.free.is_empty() {
        if matches.opt_present("x") {
            let mut last = get_clipboard();
            println!("Waiting for selection> ");
            loop {
                std::thread::sleep(std::time::Duration::from_millis(100));
                let curr = get_clipboard();
                if curr != last {
                    last = curr.clone();
                    if !last.is_empty() {
                        lookup_explain(&mut client, &curr, fmt, raw);
                        println!("Waiting for selection> ");
                    }
                }
            }
        } else {
            let mut reader = Editor::<()>::new();
            while let Ok(word) = reader.readline("> ") {
                reader.add_history_entry(&word);
                lookup_explain(&mut client, &word, fmt, raw);
            }
        }
    } else {
        for word in matches.free {
            lookup_explain(&mut client, &word, fmt, raw);
        }
    }
    return;
}
