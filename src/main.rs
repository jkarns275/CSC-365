#![allow(dead_code)]
#![feature(box_syntax)]
use std::error::Error;
use std::default::Default;
use std::fs::File;
use std::sync::Mutex;
use std::cell::RefCell;
use std::borrow::BorrowMut;
use std::io::Write;
use std::fs;
extern crate regex;

extern crate crossbeam;

#[macro_use]
extern crate lazy_static;

// Twitter API
extern crate egg_mode;
use egg_mode::*;

// Date and Time API
extern crate chrono;

// Random access file
extern crate random_access_file;
extern crate cfile_rs;

// GUI library (termbox)
extern crate rustbox;
use rustbox::RustBox;
use rustbox::Key;

// phash.rs
mod phash;
use phash::*;

// ptree.rs
mod ptree;
use ptree::*;

// log.rs
#[macro_use]
mod log;

// twitter.rs
mod twitter;
use twitter::TweetList;
mod twithash;

// gui.rs
mod gui;
use gui::*;

use std::sync::Arc;
use std::thread;

// used to log debug info without printing to stdout (since it's being used for the GUI)
lazy_static! {
    static ref LOG_FILE: Mutex<File> = Mutex::new(match File::create("LOG.txt") {
        Ok(x) => x,
        _ => panic!("Failed to create log file..."),
    });
}

static mut CURRENT_GROUP: usize = 0;

fn load() {
    // The include_str macro reads the file with the given name into a string at
    // compile time
    // These keys are needed to log in to twitter
    let con_key = include_str!("consumer_key").trim();
    let con_secret = include_str!("consumer_secret").trim();
    let acc_key = include_str!("access_token").trim();
    let acc_secret = include_str!("access_secret").trim();

    // Create the access tokens
    let con_token = egg_mode::KeyPair::new(con_key, con_secret);
    let acc_token = egg_mode::KeyPair::new(acc_key, acc_secret);

    log!("Retreiving twitter data...");
    let token = egg_mode::Token::Access {
        consumer: con_token,
        access: acc_token
    };

    // list named 'us-house' from the twitter account 'gov'
    let (slug, name) = ("us-house", "gov");
    let list = list::ListID::from_slug_name(slug, name);

    let list_info = match list.show(&token) {
        Ok(val) => {
            log!("Successfully authenticated with twitter.");
            val
        },
        Err(e) => {
            error!("Failed to authenticate with twitter: {}", e);
            panic!("");
        }
    };

    let num_tweets = 200;    // Request up to 200 tweets from each user
    let num_users = 419;       // 419 Congressmen

    log!("Requesting list of users...");
    let mut n = list_info.into_list(None, &token);
    n.user_count = num_users;
    n.count = num_tweets;
    let mut users = Vec::new();
    let mut iter = n.members();
    while users.len() < num_users as usize {
        match iter.call() {
            Ok(mut res) => users.append(&mut res.response.users),
            _ => {
                error!("Failed to retrieve list of users.");
                panic!("");
            }
        };
    }


    log!("Successfully found {} users", num_users);
    for i in users.iter() {
        log!("Found user {} @{}", i.name, i.screen_name);
    }

    let mut tweets = vec![];
    let mut cache_res = PHash::<String, twitter::TweetList>::open("data/tweet_cache");
    let mut cache;
    if let Ok(c) = cache_res {
        cache = c;
    } else {
        cache = PHash::<String, twitter::TweetList>::new("data/tweet_cache").unwrap();
    }

    while let Some(user) = users.pop() {
        log!("Loading tweets from user '{}'", user.screen_name);
        // There is already data for this user, pull it out and append it to the list of tweets.
        match cache.get(&user.screen_name) {
            Some(mut t) => {
                log!("Cache already contained tweets for user {}", user.screen_name);
                let TweetList(mut list) = t;
                tweets.append(&mut list);
                continue;
            },
            _ => {}
        }

        // There wasnt data for the user, pull some from twitter!

        let mut timeline = tweet::user_timeline(user.id, false, false, &token)
            .with_page_size(num_tweets);

        let mut converted_tweets = vec![];
        match timeline.start() {
            Ok(resp) => {
                for tweet in timeline.start().unwrap().response.into_iter() {
                    converted_tweets.push(twitter::Tweet::new(tweet));
                }
            },
            Err(e) => {
                println!("Encountered error: {}", e);
                break;
            }
        }
        let mut tweet_list = TweetList(converted_tweets);
        cache.insert(&user.screen_name, &tweet_list);
        let TweetList(mut tweets_again) = tweet_list;
        tweets.append(&mut tweets_again);
    }

    let mut processor = twitter::TweetProcessor::new();

    for tweet in tweets.iter() {
        processor.process_tweet(&tweet);
    }

    let mut tree = PTree::<String, twitter::FrequencyTable>::new("data/tweet_tree").unwrap();

    for key in processor.map.keys() {
        match processor.map.get(&key) {
            Some(table) => {
                tree.insert(&key, &table);
            },
            None => {
                panic!("This is impossible")
            }
        }
    }
}

fn run(num_groups: usize, num_iters: usize) {
    /**
                                          .`*`
                                        .'* *.'
                                     __/_*_*(_
                                    / _______ \       _____________________________
                                   _\_)/___\(_/_      |                            |
                                  / _((\- -/))_ \     |    B E  W A R N E D -      |
                                  \ \())(-)(()/ /     |  M A G I C  N U M B E R S  |
                                   ' \(((()))/ '      |     L I E  A H E A D       |
                                  / ' \)).))/ ' \     |   _________________________|
                                 / _ \ - | - /_  \   /___/
                                (   ( .;''';. .'  )
                                _\"__ /    )\ __"/_
                                  \/  \   ' /  \/
                                   .'  '...' ' )
                                    / /  |  \ \
                                   / .   .   . \
                                  /   .     .   \
                                 /   /   |   \   \
                               .'   /    b    '.  '.
                           _.-'    /     Bb     '-. '-._
                       _.-'       |      BBb       '-.  '-.
                       (________mrf\____.dBBBb.________)____)


          "Magic" numbers are used for positioning and sizing the GUI components.
          They make for a reasonably good looking GUI though.
     */

    let mut tree;
    // If the tree fails to load for some reason, recreate it and try again.
    match PTree::<String, twitter::FrequencyTable>::open("data/tweet_tree") {
        Ok(t) => tree = t,
        _ => {
            log!("Failed to find any data... Will load some right now :)");
            load();
            run(num_groups, num_iters);
            return;
        }
    }

    let mut processor = twitter::TweetProcessor::new();
    let users;
    match tree.keys() {
        Ok(t) => {
            log!("Successfully found key list from PTree");
            users = t;
        },
        _ => {
            log!("Failed to open data.. Exiting...");
            return;
        }
    }

    for user in users.into_iter() {
        match tree.search(&user) {
            Ok(Some(r)) => {
                log!("Found data from tree for user {}", user);
                processor.map.insert(user, r);
            },
            _ => log!("Failed to find user {}", user),
        }
    }

    let mut handles = processor.k_means_groups(num_groups, num_iters);
    handles = handles.into_iter().filter(|ref i| i.len() != 0).collect::<Vec<Vec<String>>>();
    log!("Finished creating k-means groups");
    log!("Num groups: {}", handles.len());

    log!("Creating GUI...");
    let mut container2 = Container::new(0, 0, 90, 90);
    let t1 = GuiText::new(1, 8, "ᴧ");
    let t2 = GuiText::new(1, 9, "v");
    let mut cur_label = GuiText::new(1, 3, "Currently viewing group #0");
    let selector = GuiSelection2D::new_default(3, 4, 32, 24, handles).component(Box::new(
        |s, k| {
            match k {
                Key::Left => {
                    s.left();
                    unsafe { CURRENT_GROUP = s.cur_choice_x; }
                },
                Key::Right => {
                    s.right();
                    unsafe { CURRENT_GROUP = s.cur_choice_x; }
                },
                Key::Up => {
                    s.down();
                },
                Key::Down => {
                    s.up();
                },
                Key::Enter => {
                    unsafe { DISPLAY_TWEETS = true; }
                },
                _ => {}
            }
        }
    ));
    container2.add(Box::new(selector)).unwrap();
    let exit_button = GuiSelection::button_default(36, 4, "Exit").component(Box::new(
        |_, k| {
            match k {
                Key::Enter => unsafe { KILL = true; },
                _ => {}
            };
        }
    ));
    container2.add(Box::new(exit_button)).unwrap();

    //container2.add(box t1);
    //container2.add(box t2);

    /* This screen goes unused
    // Screen 3 (Selected screen)
    let mut container3  = Container::new(0, 0, 100, 100);
    let you_selected    = GuiText::new(2, 3, "You selected:");
    let most_similar    = GuiText::new(35, 3, "Which is most similar to:");
    let go_back         = GuiText::new(24, 16, "Press 'q' to go back");

    container3.add(Box::new(you_selected));
    container3.add(Box::new(most_similar));
    container3.add(Box::new(go_back));
    */
    log!("Diplaying GUI...");

    let rustbox = match RustBox::init(Default::default()) {
        Result::Ok(v) => v,
        Result::Err(e) => panic!("{}", e),
    };

    container2.next();

    container2.draw(&rustbox);
    cur_label.draw(&rustbox);
    rustbox.present();
    let mut container2 = container2.component(box |s, k| {
        match k {
            Key::Tab => {
                s.next();
            },
            Key::Enter => {
                s.components[s.selected as usize].handle_input(k);
            },
            _ => {
                s.components[s.selected as usize].handle_input(k);
            }
        }
    });

    loop {
        match rustbox.poll_event(false) {
            Ok(rustbox::Event::KeyEvent(tw)) => {
                if let Key::Esc = tw {
                    break;
                }
                container2.handle_input(tw);
                /*unsafe { don't bother displaying the most similar tweet
                    if DISPLAY_TWEETS {
                        let t = mux.components[0].to_string();
                        let key = processor.closest_key(&t);
                        let container_sub = Container::new(0, 0, 100, 100);
                        let selected_display    = processor.map.get(&&mux.components[1].to_string()).unwrap().display();
                        let closest_display     = processor.map.get(&&key).unwrap().display();
                        let selected = GuiSelection::new_default(3, 5, 30, 10, selected_display);
                        let selected_name = "@".to_string() + &mux.components[1].to_string();
                        let selected_label = GuiTextBox::new(2, 4, 22, &selected_name);
                        let closest = GuiSelection::new_default(35, 5, 30, 10, closest_display);
                        let closest_name = "@".to_string() + &key;
                        let closest_label = GuiTextBox::new(33, 4, 22, &closest_name);
                        rustbox.clear();
                        selected.draw(&rustbox);
                        closest.draw(&rustbox);
                        selected_label.draw(&rustbox);
                        closest_label.draw(&rustbox);
                        container3.draw(&rustbox);
                        rustbox.present();
                        loop {
                            match rustbox.poll_event(false) {
                                Ok(rustbox::Event::KeyEvent(a)) => {
                                    if let Key::Char(_) = a {
                                        DISPLAY_TWEETS = false;
                                        break;
                                    }
                                },
                                _ => {}
                            };
                            container_sub.draw(&rustbox);
                            selected.draw(&rustbox);
                            closest.draw(&rustbox);
                            selected_label.draw(&rustbox);
                            closest_label.draw(&rustbox);
                            container3.draw(&rustbox);
                            rustbox.present();
                        }
                    }
                }*/
            },
            Err(e) => panic!("{}", e.description()),
            _ => { }
        }
        unsafe { if KILL { break; } }
        rustbox.clear();
        container2.draw(&rustbox);
        unsafe { cur_label.text = format!("Currently viewing group #{}", CURRENT_GROUP); }
        cur_label.draw(&rustbox);
        t1.draw(&rustbox);
        t2.draw(&rustbox);
        rustbox.present();
    }
}

static mut KILL: bool = false;
static mut DISPLAY_TWEETS: bool = false;

use std::env;
fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 {
        println!("No arguments provided...");
        println!("Usage:\n\tload - Loads data into the persistant data structures\n\n\tdisplay - Loads the data from the persistant data structures into a gui. If there is no data in the persistant data structures the program will say so and exit.");
    } else if args[1].to_uppercase() == "LOAD".to_string() {
        load();
    } else if args[1].to_uppercase() == "CLEAR".to_string() {
        fs::remove_dir_all("data").unwrap();
        fs::create_dir("data").unwrap();
    } else if args[1].to_uppercase() == "DISPLAY".to_string() {
        if args.len() < 4 {
            println!("display command requires two arguments, the number of groups and the number of iters.");
            return;
        }
        let k;
        match args[2].parse::<usize>() {
            Ok(x) => k = x,
            Err(e) => {
                println!("Failed to parse second argument '{}'", args[2]);
                return
            }
        };
        let g;
        match args[3].parse::<usize>() {
            Ok(x) => g = x,
            Err(e) => {
                println!("Failed to parse third argument '{}'", args[3]);
                return
            }
        }
        run(k, g);
    } else {
        println!("No valid arguments provided...");
        println!("Usage:\n\tload - Loads data into the persistant data structures\n\n\tdisplay - Loads the data from the persistant data structures into a gui. If there is no data in the persistant data structures the program will say so and exit.");
    }
}
