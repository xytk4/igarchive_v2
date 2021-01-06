// started @ Sat 2-Jan-2021 05:18 AM
// til about Sat 2-Jan-2021 08:26 PM
#[macro_use]
extern crate prettytable;
extern crate ctrlc;
use chrono::{Duration, NaiveDateTime};
use colored::*;
use hhmmss::Hhmmss;
use prettytable::Table;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::fs::File;
use std::io::prelude::*;
use text_io::read;

// TODO: make this more portable for other people?
//       maybe do a first time setup and store this and the messages.json location
//       in a config file next to it or something like that
const SELF_NAME: &str = "twohumansentertainment";

fn main() {
    println!("Old Instagram Archived Messages Browser version 2");

    ctrlc::set_handler(move || {
        std::process::exit(0);
    })
    .expect("failed to set ctrl-c handler..??!!"); // this should never happen

    println!("* loading messages.json ...");
    let mut file = File::open("messages.json")
        .expect("Failed to open messages.json! Make sure it's next to the program.");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("Failed to read messages.json! Make sure it's next to the program and you have read-access.");
    println!("* deserializing messages into memory ...");
    let archive: Vec<Thread> = serde_json::from_str(&contents).unwrap();
    println!("* loaded {} threads ...", archive.len());

    let mut current_thread = 0;

    loop {
        // Prompt
        let ui = prompt("> ".to_string());
        if ui.len() == 1 {
            continue;
        }
        // Prompt
        let bits = ui.split_whitespace().collect::<Vec<&str>>();
        if bits.len() == 0 {
            continue;
        }
        match bits[0] {
            "lat" => {
                // List all threads
                for i in 0..archive.len() {
                    if archive[i].participants.len() == 1 {
                        println!(
                            "    thread {} with just yourself",
                            i.to_string().blue().bold()
                        );
                        continue;
                    }
                    let mut s_parts = archive[i].participants.clone();
                    s_parts.remove(
                        archive[i]
                            .participants
                            .iter()
                            .position(|x| *x == SELF_NAME)
                            .unwrap(),
                    );
                    if s_parts.len() == 1 {
                        if i == current_thread {
                            println!(
                                ">>> thread {} with {}",
                                i.to_string().blue().bold(),
                                s_parts[0].blue().bold()
                            );
                        } else {
                            println!(
                                "    thread {} with {}",
                                i.to_string().blue().bold(),
                                s_parts[0].blue().bold()
                            );
                        }
                        continue;
                    }
                    if i == current_thread {
                        println!(
                            ">>> thread {} with {} people including {}, {} ",
                            i.to_string().blue().bold(),
                            archive[i].participants.len(),
                            &s_parts[0].blue().bold(),
                            &s_parts[1].blue().bold()
                        );
                    } else {
                        println!(
                            "    thread {} with {} people including {}, {} ",
                            i.to_string().blue().bold(),
                            archive[i].participants.len(),
                            &s_parts[0].blue().bold(),
                            &s_parts[1].blue().bold()
                        );
                    }
                }
            }
            "p" => {
                // List all thread participants for current thread
                println!(
                    "Thread {} has {} participants:",
                    current_thread,
                    archive[current_thread].participants.len()
                );
                for p in &archive[current_thread].participants {
                    println!("   * {}", p);
                }
            }
            "ct" => {
                // Currently selected thread?
                println!("Thread {} currently selected.", current_thread);
            }
            "ti" => {
                // Thread info
                let thread = if bits.len() < 2 {
                    current_thread
                } else {
                    let n = bits[1].parse::<usize>();
                    match n {
                        Ok(n) => {
                            // check if it's within scope
                            if n <= archive.len() {
                                n
                            } else {
                                println!(
                                    "thread id must be <= total threads (here {})",
                                    archive.len()
                                );
                                current_thread
                            }
                        }
                        Err(_) => {
                            println!("thread id must be a valid int.");
                            current_thread
                        }
                    }
                };
                println!("Info for thread {}:", thread);
                println!("# Participants: {}", archive[thread].participants.len());
                println!(
                    "First message date:  {}",
                    archive[thread].conversation[archive[thread].conversation.len() - 1].created_at
                );
                println!(
                    "Latest message date: {}",
                    archive[thread].conversation[0].created_at
                );
                println!("Number of messages: {}", archive[thread].conversation.len());
            }
            "ams" => {
                // Advanced Message Statistics (leaderboard-style)
                println!("Advanced Message Statistics for thread {}!", current_thread);
                let mut user_totals: HashMap<String, usize> = HashMap::new();
                let mut grand_total = 0;
                for user in &archive[current_thread].participants {
                    user_totals.insert(user.to_owned(), 0);
                    for message in &archive[current_thread].conversation {
                        if &message.sender == user {
                            *user_totals.get_mut(user).unwrap() += 1;
                            grand_total += 1;
                        }
                    }
                    if *user_totals.get(user).unwrap() == 0 {
                        user_totals.remove_entry(user);
                    }
                }

                let mut count_vec: Vec<(&String, &usize)> = user_totals.iter().collect();
                count_vec.sort_by(|a, b| b.1.cmp(a.1));

                let mut table = Table::new();
                table.add_row(row!["username", "count", "percentage"]);
                for user in count_vec {
                    let percentage = *user.1 as f64 / grand_total as f64 * 100.0;
                    table.add_row(row![user.0, user.1, percentage.ceil() as usize]);
                }
                table.printstd();
            }
            "ams-mot" => {
                // Advanced Message Stats: Messages Over Time
                let mut interval = "m";
                if bits.len() > 1 {
                    match bits[1] {
                        "d" => {
                            interval = "d";
                        } // daily
                        "m" => {
                            interval = "m";
                        } // monthly
                        "y" => {
                            interval = "y";
                        } // yearly
                        _ => {} // default? or err
                    }
                }
                println!(
                    "Advanced Message Stats for thread {}: MESSAGES OVER TIME",
                    current_thread
                );
                let t = &archive[current_thread];
                let mut time_totals: BTreeMap<String, usize> = BTreeMap::new();
                for message in &t.conversation {
                    let ts_bits: Vec<&str> = message.created_at.split("T").collect::<Vec<&str>>()
                        [0]
                    .split("-")
                    .collect();
                    let mut ts: String = "".to_string();
                    match interval {
                        "d" => {
                            ts = format!("{}-{}-{}", ts_bits[0], ts_bits[1], ts_bits[2]);
                        }
                        "m" => {
                            ts = format!("{}-{}", ts_bits[0], ts_bits[1]);
                        }
                        "y" => {
                            ts = format!("{}", ts_bits[0]);
                        }
                        _ => {}
                    }
                    *time_totals.entry(ts).or_insert(1) += 1;
                }
                println!("{:#?}", time_totals);
            }
            "ams-dt" => {
                // Advanced Message Stats: Delta-time
                // Longest-shortest time between messages and stuff
                let mut longest_time = Duration::min_value(); // Longest  time between 2 messages
                let mut longest_time_pos = 0;

                let mut shortest_time = Duration::max_value(); // Shortest time between 2 messages

                //let mut longest_time_btwn = Duration::min_value(); // Longest  time between 2 messages from dif people

                //let mut shortest_time_btwn = Duration::max_value(); // Shortest time between 2 messages from dif people

                println!(
                    "Advanced Message Stats for thread {}: Delta-Time calculation",
                    current_thread
                );
                let t = &archive[current_thread];
                for i in (0..t.conversation.len() - 1).rev() {
                    let ts = &t.conversation[i].to_ndt();
                    let sincelast = ts.signed_duration_since(t.conversation[i + 1].to_ndt());
                    if sincelast > longest_time {
                        longest_time = sincelast;
                        longest_time_pos = t.conversation.len() - i - 2;
                    } else if sincelast < shortest_time {
                        shortest_time = sincelast;
                    }
                    // now the tricky bit
                    // TODO: make this work
                    /*
                    for j in i..t.conversation.len() - 1 {
                        if &t.conversation[j].sender != &t.conversation[i].sender {
                            if sincelast > longest_time_btwn {
                                longest_time_btwn = sincelast;
                            }
                            break;
                        }
                    } */
                }
                let longest_time_info = format!(
                    "(between {} and {})",
                    longest_time_pos,
                    longest_time_pos + 1
                );

                println!(
                    "Longest time between messages: {}s {}",
                    longest_time.to_std().unwrap().hhmmssxxx(),
                    longest_time_info
                );
                println!(
                    "Shortest time between messages: {}s",
                    shortest_time.to_std().unwrap().hhmmssxxx()
                );
            }
            "n" => {
                // Calculate (WHY!?) the NAME of the thing
                let mut found = 0;
                let c = &archive[current_thread].conversation;
                for i in (0..c.len()).rev() {
                    if c[i].action.is_some() {
                        if c[i].action.as_ref().unwrap().contains("named the group") {
                            let timestamp = c[i]
                                .created_at
                                .strip_suffix("+00:00")
                                .unwrap_or("err")
                                .green();
                            let parts: Vec<&str> = c[i]
                                .action
                                .as_ref()
                                .unwrap()
                                .split(" named the group ")
                                .collect();
                            if parts[0] == c[i].sender {
                                println!(
                                    "{}: {} named it '{}'",
                                    timestamp,
                                    parts[0].blue().bold(),
                                    parts[1]
                                );
                            } else {
                                println!(
                                    "{}: {} ({}) named it '{}'",
                                    timestamp,
                                    parts[0].blue().bold(),
                                    c[i].sender.yellow(),
                                    parts[1]
                                );
                            }
                            found += 1;
                        }
                    }
                }
                println!("Total changed {} times.", found);
            }
            "t" => {
                // Change thread
                if bits.len() > 1 {
                    let n = bits[1].parse::<usize>();
                    match n {
                        Ok(n) => {
                            // check if it's within scope
                            if n <= archive.len() {
                                current_thread = n;
                            } else {
                                println!(
                                    "thread id must be <= total threads (here {})",
                                    archive.len()
                                );
                            }
                        }
                        Err(_) => {
                            println!("thread id must be a valid int.");
                        }
                    }
                } else {
                    println!("syntax: 't n' where n is thread id");
                }
            }
            /*
            "dam" => {
                // Download all media for thread EXCEPT THIS IS IMPOSSIBLE
                // Instagram / Facebook makes CDN urls expire all the time
                // so I don't think I can actually do this

                println!("Downloading all media for thread... Note this only works if it's a fresh data thing");
                let mut urls: Vec<String> = vec![];
                for message in &archive[current_thread].conversation {
                    if message.media.is_some() {
                        urls.push(message.media.as_ref().unwrap().to_string());
                    } else if message.media_url.is_some() {
                        urls.push(message.media_url.as_ref().unwrap().to_string());
                    }
                }
                println!(
                    "Found {} media items... now I would download them...",
                    urls.len()
                );
                println!(
                    "For now, have the first and last url.\n{}\n{}\n",
                    urls[0],
                    urls.last().unwrap()
                );
            }
            */
            "m!" => {
                // Enter MESSAGE MODE (! indicates mode change)
                println!("Entering message mode for thread {} ...", current_thread);
                message_browse(&archive[current_thread], current_thread);
                println!("Exited message mode.");
            }
            "h" => {
                // Display help text
                help_text();
            }
            "q" => {
                // Exit program
                break;
            }
            _ => {
                // I have no idea.
                println!("?");
            }
        }
    }
}

/// Message browse console
fn message_browse(thread: &Thread, thread_id: usize) {
    let tlen = thread.conversation.len() - 1; // We use this to fix the reversed indexing

    let mut current_message = 0;

    let mut auto_print = true; // Things can disable this temp'ly
                               // if it will look nicer

    let mut auto_forward = true; // Move forwards on empty
    let mut rev_auto_forward = true; // Actually move backwards if false

    loop {
        if auto_print {
            println!("{}\n", thread.conversation[tlen - current_message]);
        } else {
            auto_print = true; // Reset this once we've done it.
        }
        // Prompt
        let ui = prompt(format!("[{}, {}]> ", thread_id, current_message));

        // Auto advancing of messages either way
        if ui.len() < 2 {
            if auto_forward {
                if rev_auto_forward {
                    // Move forwards
                    if current_message < thread.conversation.len() - 1 {
                        current_message += 1;
                    } else {
                        println!("(reached end)");
                        auto_print = false;
                    }
                } else {
                    // The other way
                    if current_message > 0 {
                        current_message -= 1;
                    } else {
                        println!("(reached start)");
                        auto_print = false;
                    }
                }
            }
            continue;
        }

        // Split command line into separated bits
        let bits = ui.split_whitespace().collect::<Vec<&str>>();
        if bits.len() == 0 {
            // just a space... so do nothing
            auto_print = false;
            continue;
        }
        match bits[0] {
            "." | "=" | "+" => {
                // Move forwards
                if current_message < thread.conversation.len() {
                    current_message += 1;
                } else {
                    println!("(reached end)");
                    auto_print = false;
                }
            }
            "," | "-" | "_" => {
                // Move backwards
                if current_message > 0 {
                    current_message -= 1;
                } else {
                    println!("(reached start)");
                    auto_print = false;
                }
            }
            "s" => {
                // Go to -f-l-y- start
                current_message = 0;
            }
            "e" => {
                // Go to end
                current_message = thread.conversation.len() - 1;
            }
            "af" => {
                // toggle auto-forward
                println!(
                    "Toggled auto-forward to {}",
                    match auto_forward {
                        true => "off",
                        false => "on",
                    }
                );
                auto_forward = !auto_forward;
                auto_print = false;
            }
            "raf" => {
                // toggle rev-auto-forward
                println!(
                    "Toggled auto-forward direction to {}",
                    match rev_auto_forward {
                        true => "reversed",
                        false => "forwards",
                    }
                );
                rev_auto_forward = !rev_auto_forward;
                auto_print = false;
            }
            "m" => {
                // view media also for 'm n'
                if bits.len() == 1 {
                    if thread.conversation[tlen - current_message].media.is_some() {
                        println!(
                            "Uploaded media: {}",
                            thread.conversation[tlen - current_message]
                                .media
                                .as_ref()
                                .unwrap()
                        );
                    } else if thread.conversation[tlen - current_message]
                        .media_owner
                        .is_some()
                    {
                        println!(
                            "Shared media from {}: {}",
                            thread.conversation[tlen - current_message]
                                .media_owner
                                .as_ref()
                                .unwrap(),
                            thread.conversation[tlen - current_message]
                                .media_share_url
                                .as_ref()
                                .unwrap(),
                        );
                    } else if thread.conversation[tlen - current_message]
                        .media_url
                        .is_some()
                    {
                        println!(
                            "Shared media_url media: {}",
                            thread.conversation[tlen - current_message]
                                .media_url
                                .as_ref()
                                .unwrap()
                        );
                    }
                }
            }
            "l" => {
                // view likes
                if bits.len() == 1 {
                    thread.conversation[tlen - current_message].print_likes();
                    auto_print = false;
                } else {
                    let n = bits[1].parse::<usize>();
                    match n {
                        Ok(n) => {
                            // check if it's within scope
                            if n > 0 && n < thread.conversation.len() {
                                thread.conversation[tlen - n].print_likes();
                                auto_print = false;
                            } else {
                                println!(
                                    "message id must be > 0 and < total conversation length (here {})",
                                    thread.conversation.len()
                                );
                                auto_print = false;
                            }
                        }
                        Err(_) => {
                            println!("syntax: 'l' to view for current or 'l n' where n is index.");
                            auto_print = false;
                        }
                    }
                }
            }
            "f" => {
                // Find
                let mut needle = String::new();
                if bits.len() == 1 {
                    println!("syntax: 'f blah blah ...'");
                    auto_print = false;
                    continue;
                } else {
                    for i in 0..bits.len() {
                        if i == 0 {
                            continue;
                        }
                        if i > 1 {
                            needle.push(' ');
                        }
                        needle.push_str(bits[i]);
                    }
                }

                if needle.len() < 3 {
                    println!("Needle too short.. you probably don't want to do that.");
                    auto_print = false;
                    continue;
                }
                let mut found = 0;

                for i in (0..thread.conversation.len()).rev() {
                    if thread.conversation[i].text.is_some() {
                        if thread.conversation[i]
                            .text
                            .as_ref()
                            .unwrap()
                            .to_lowercase()
                            .contains(&needle.to_lowercase())
                        {
                            // generate highlighted string
                            // oh no i probably need to clone the message don't I oh well
                            // not today
                            // TODO: highlight needle in find output

                            /*
                            let m = thread.conversation[i];
                            m.text = Some("".to_string());
                            for part in m.text.unwrap().to_lowercase().split(&needle.to_lowercase()) {
                                m.text.unwrap().push_str(part);
                                m.text.unwrap().push_str(&needle.red());
                                // ???
                            }
                            */

                            // print it
                            println!(
                                "  {} {}",
                                format!("[{}, {}]:", thread_id, tlen - i).yellow(),
                                thread.conversation[i]
                            );
                            found += 1;
                        }
                    }
                }
                println!("Found '{}' {} times.", needle, found);
                auto_print = false;
            }
            "fu" => {
                // Find by user
                if bits.len() != 2 {
                    println!("syntax: 'fu username'");
                    auto_print = false;
                    continue;
                }
                let username = bits[1];
                if !thread.participants.contains(&username.to_string()) {
                    println!("username not found in thread participants...");
                    auto_print = false;
                    continue;
                }
                let mut found = 0;
                for i in (0..thread.conversation.len()).rev() {
                    if thread.conversation[i].sender == username {
                        println!(
                            "  {} {}",
                            format!("[{}, {}]:", thread_id, tlen - i).yellow(),
                            thread.conversation[i]
                        );
                        found += 1;
                    }
                }
                println!("Found {} messages sent by {}.", found, username);
                auto_print = false;
            }
            "h" => {
                help_text();
            }
            "q" => {
                // exit message mode
                return;
            }
            _ => {
                // Not a command? Check if it's a valid message id
                let n = bits[0].parse::<usize>();
                match n {
                    Ok(n) => {
                        // check if it's within scope
                        if n > 0 && n < thread.conversation.len() {
                            current_message = n;
                        } else {
                            println!(
                                "message id must be > 0 and < total conversation length (here {})",
                                thread.conversation.len()
                            );
                        }
                    }
                    Err(_) => {
                        println!("?");
                        //println!("thread id must be a valid int.");
                    }
                }
            }
        }
    }
}

/// Prompt user for input
fn prompt(s: String) -> String {
    print!("{}", s);
    let _ = std::io::stdout().flush();
    let ui = read!("{}\n");
    ui
}

/// Display help/info text.
fn help_text() {
    // Oh gosh
    println!("Help text!");
    println!("Normal mode commands: ");
    println!(" *   t: change thread // 't n' where n is thread id to change to.");
    println!(" *  ti: thread info // 'ti' for current thread or 'ti n' where n is thread id to display info for");
    println!(" *  ct: current thread // display currently selected thread id");
    println!(" *   p: participants // list all participants in currently selected thread");
    println!(" * lat: list all threads // lists all threads in the archive");
    println!(" *   n: find name changes // lists all name changes that happened in the currently selected thread");
    println!(" * ams: advanced message stats // produces a leaderboard of # messages sent sorted by message count");
    println!(" * ams-mot: messages over time // 'ams-mot i' where i is interval (d, m, y); outputs message count at that interval");
    println!(" * ams-dt: message delta-time // calculates the shortest and longest time between messages in the thread");
    println!(" * m!: switch to the message-mode console, which lets you look at messages and perform per-message ops.");
    println!();
    println!("Message-mode commands: ");
    println!(" * (. = +): next message");
    println!(" * (, - _): prev message");
    println!(" * s: go to first message");
    println!(" * e: go to latest message");
    println!(" * af: toggle auto-forward (advances message index on empty console input)");
    println!(" * raf: toggle reverse-auto-forward (reverses auto-forward direction, for compatibility reasons or something)");
    println!(" * m: view media info // 'm' for current message or 'm n' where n is message id");
    println!(" * l: view message likes // 'l' for current message or 'l n' where n is message id");
    println!(" * f: find text in thread messages: 'f blah blah ...'");
    println!(" * fu: find by user: outputs all of user's messages in thread // 'fu username'");
    println!(" * q: quit message mode, return to normal console");
}

/// Message type formatter
impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Premake the sender, so it can be a pleasant blue colour
        let sender = self.sender.blue().bold();
        // Strip down the suffix a little bit
        let timestamp = self
            .created_at
            .strip_suffix("+00:00")
            .unwrap_or("err")
            .green();

        // Premake the likes thing, if it's any
        let mut likes = String::new();
        if self.likes.is_some() {
            likes = format!("[<3 x{}]", self.likes.as_ref().unwrap().len())
                .red()
                .to_string();
        }
        // Now format it based on what's available
        if self.text.is_some() {
            // Normal text message
            write!(
                f,
                "  user {} on {} wrote: {}\n  {}",
                sender,
                timestamp,
                likes,
                self.text.as_ref().unwrap()
            )
        } else if self.media.is_some() {
            // Media (ie. user uploaded)
            write!(
                f,
                "  user {} on {} sent media. {}",
                sender, timestamp, likes
            )
        } else if self.media_url.is_some() {
            write!(
                f,
                "  user {} on {} sent media_url media.... {}",
                sender, timestamp, likes
            )
        } else if self.media_owner.is_some() {
            // Shared media (ie. shared post?)
            write!(
                f,
                "  user {} on {} sent media from {}. {}",
                sender,
                timestamp,
                self.media_owner.as_ref().unwrap_or(&"UNKNOWN".to_string()),
                likes
            )
        } else if self.action.is_some() {
            // Misc action (there's lots of these)
            write!(
                f,
                "  user {} on {} took action '{}'",
                sender,
                timestamp,
                self.action.as_ref().unwrap()
            )
        } else if self.heart.is_some() {
            // Just a heart, which is somehow different from just sending one as text..
            write!(f, "  user {} on {} sent {}", sender, timestamp, "<3".red())
        } else if self.video_call_action.is_some() {
            // Special video call action.
            write!(
                f,
                "  user {} on {} took vidcall action '{}'",
                sender,
                timestamp,
                self.video_call_action.as_ref().unwrap()
            )
        } else if self.live_video_invite.is_some() {
            write!(
                f,
                "  user {} on {} sent a live video invite",
                sender, timestamp
            )
        } else {
            // I have no idea... check in the HUGE JSON VIEWER
            write!(f, "  user {} on {} did ???", sender, timestamp)
        }
    }
}

impl Message {
    /// Print likes of message
    fn print_likes(&self) {
        match &self.likes {
            Some(likes) => {
                println!(
                    "Has {} likes:",
                    self.likes.as_ref().unwrap().len().to_string().red()
                );
                for like in likes {
                    println!(
                        "... {} on {}",
                        like.username.blue().bold(),
                        like.date.strip_suffix("+00:00").unwrap_or("err").green()
                    );
                }
            }
            None => {
                println!("Has no likes.");
            }
        }
    }

    /// Generate NativeDateTime from the created_at field.
    fn to_ndt(&self) -> NaiveDateTime {
        NaiveDateTime::parse_from_str(&self.created_at, "%Y-%m-%dT%H:%M:%S%.6f+00:00").unwrap()
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct Message {
    // Constant values
    sender: String,     // Sender / author of message
    created_at: String, // Timestamp

    // Optional values lol
    text: Option<String>,              // Normal Text Message
    media: Option<String>,             // Media SENT? url
    media_url: Option<String>,         // A (different) media url??!~idk
    media_share_url: Option<String>,   // Media SHARED url
    media_owner: Option<String>,       // Media owner (if shared)
    action: Option<String>,            // Action taken
    video_call_action: Option<String>, // Video call action (?!?!!)
    live_video_invite: Option<String>, // Invited to live video
    heart: Option<String>,             // sent just a heart (why is this seperate?? idk)
    likes: Option<Vec<Like>>,          // vec of likes
}

#[derive(Debug, Deserialize, Serialize)]
struct Thread {
    participants: Vec<String>,  // List of usernames in thread
    conversation: Vec<Message>, // List of Messages in thread
}

#[derive(Debug, Deserialize, Serialize)]
struct Archive {
    threads: Vec<String>, // All threads in archive
}

#[derive(Debug, Deserialize, Serialize)]
struct Like {
    username: String, // Username who liked it
    date: String,     // Timestamp of like (?)
}
