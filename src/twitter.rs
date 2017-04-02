use egg_mode::*;
use twithash::*;
use regex::*;
use std::f64;
use std::cmp::Ordering;
use LOG_FILE;
use std::io::{ Seek, SeekFrom, Read, Write };
use std::io;
use random_access_file::Serialize;
use std::mem;
use crossbeam;

/// An incredibly useful macro. It will check an expression of type Error<T, Z>,
/// if it is Err(err) it will RETURN in whatever function it is placed in with
/// error, otherwise it will continue in the function. This cuts down on the amount
/// of error checking code that will clog things up.
/// Optionally, it will also store the value x in $v (e.g. if it is Ok(x), $v = x).
macro_rules! check {
    ( $e:expr ) => (
    match $e {
        Ok(_) => {},
    Err(e) => return Err(e)
        }
    );
    ( $e:expr, $v:ident) => (
        match $e {
            Ok(r) => $v = r,
            Err(e) => return Err(e)
        }
    )
}

const MAX_THREADS: i32 = 8;

/// A structure representing the occurences of words from a collection
/// of tweets. The tweets are not stored.
#[derive(PartialEq, Eq)]
pub struct FrequencyTable {
    /// The actual frequency table, word -> occurences
    table: TwitHash<String, usize>,
    /// The total number of words (non-unique) that have been entered
    word_count: usize
}

impl Serialize for FrequencyTable {
    type DeserializeOutput = Self;

    fn serialize(&self, to: &mut Write) -> Result<(), io::Error> {
        check!((self.word_count as u64).serialize(to));
        check!((self.table.count as u64).serialize(to));
        let keys = self.table.keys();
        for key in keys {
            check!(key.serialize(to));
            check!((*(self.table.get(key).unwrap()) as u64).serialize(to));
        }
        Ok(())
    }

    fn deserialize(from: &mut Read) -> Result<Self::DeserializeOutput, io::Error> {
        let word_count;
        check!(u64::deserialize(from), word_count);
        let num_pairs;
        check!(u64::deserialize(from), num_pairs);

        let mut table = TwitHash::new();

        for _ in 0..num_pairs {
            let key;
            check!(String::deserialize(from), key);
            let value;
            check!(u64::deserialize(from), value);
            table.insert(key, value as usize);
        }

        Ok(FrequencyTable {
            table: table,
            word_count: word_count as usize
        })
    }

    fn serialized_len(&self) -> u64 {
        let mut count = 16;

        for key in self.table.keys() {
            count += 16 + key.len();
        }

        count as u64
    }
}

/// A helper method that creates a set-type union out of two vectors,
/// consuming both of them.
fn union<T: PartialEq>(mut v1: Vec<T>, v2: Vec<T>) -> Vec<T> {
    for element in v2.into_iter() {
        if !v1.contains(&element) {
            v1.push(element);
        }
    }
    v1
}

fn intersection<T: PartialEq>(mut v1: Vec<T>, v2: Vec<T>) -> Vec<T> {
    v1.retain(|x| v2.contains(x));
    v1
}

fn magnitude(s: &str) -> f64 {
    let mut acc = 0.0;
    for i in s.as_bytes().iter() {
        acc += (i*i) as f64;
    }
    acc.sqrt()
}

impl FrequencyTable {

    /// Creates a new empty FrequencyTable
    pub fn new() -> Self {
        FrequencyTable {
            table: TwitHash::new(),
            word_count: 0usize
        }
    }

    /// Adds the words from a tweet to the FrequencyTable.
    /// It essentially splits by whitespace and adds increments the
    /// matching value in the table (if it exists, otherwise it is initialized
    /// to 1)
    pub fn add_tweet(&mut self, t: &Tweet) {
        for w in t.stripped_text.split_whitespace() {
            // not sure if I want this here or not
            //if w.len() < 2 { continue };
            let v =
                if let Some(x) = self.table.get_mut(&w.to_string()) {
                    *x
                } else {
                    0
                };
            self.table.insert(w.to_string(), v + 1);
        }
    }

    /// Returns log2 of the number of occurences of the given word.
    pub fn tf(&self, word: &String) -> f64 {
        match self.table.get(word) {
            Some(x) => {
                (*x as f64).log2() // + 1 maybe?
            },
            None => 0.0,
        }
    }

    pub fn freq(&self, word: &String) -> usize {
        match self.table.get(word) {
            Some(x) => {
                *x
            },
            None => 1usize,     // Divide by zero prevention
        }
    }

    /// Compares two FrequencyTables and returns a f64 which represents their
    /// difference. The closer to 1, the more similar.
    /// The algorithm used is a spin off of cosine similarity.
    pub fn compare(&self, other: &Self) -> f64 {
        /*
        let v = union(self.table.keys(), other.table.keys());
        let mut difference = 0f64;
        for word in v.into_iter() {
            let tf1 = self.tf(word);
            let f1 = self.freq(word) as f64;
            let tf2 = other.tf(word);
            let f2 = self.freq(word) as f64;
            if tf1 != 0.0 && tf2 != 0.0 {
                difference += (tf1 / f2 - tf2 / f1).abs();
            }
        }
        difference
        */
        let num = self.mul(other);
        let denom = self.abs() * other.abs();
        if denom == 0.0 {
            0.0
        } else {
            num / denom
        }
    }

    /// Returns a vector to be used to display the top 10 most
    /// frequent words using a GuiSelection.
    pub fn display(&self) -> Vec<String> {
        let mut disp = Vec::new();
        for key in self.table.keys().into_iter() {
            disp.push((self.table.get(key).unwrap(), key));
        }
        disp.sort_by(|a, b| {
            if a.0 > b.0        { Ordering::Less }
            else if a.0 < b.0   { Ordering::Greater }
            else                { Ordering::Equal }
        });
        let size = if 10 >= self.table.count { self.table.count } else { 10 };
        let mut ret = Vec::with_capacity(size);
        for i in 0..size {
            let (freq, key) = disp[i];
            // A poor attempt to allign the words
            if *freq >= 10 {
                ret.push(freq.to_string() + "x - " + key)
            } else {
                ret.push(freq.to_string() + "x  - " + key);
            }
        }
        ret
    }

    pub fn sum(&self) -> f64 {
        let mut acc = 0.0;
        for key in self.table.keys() {
            acc += *self.table.get(key).unwrap() as f64;
        }
        acc
    }

    pub fn angle(&self) -> f64 {
        let len = self.table.count as f64;
        let x = ((1.0 + len) / self.abs() as f64).acos();
        if x == f64::NAN {
            3.1415926
        } else {
            x
        }
    }

    pub fn abs(&self) -> f64 {
        let mut acc = 0.0;
        for key in self.table.keys() {
            let freq = *self.table.get(key).unwrap();
            acc += (freq * freq) as f64;
        }
        acc.sqrt()
    }

    pub fn mul(&self, rhs: &Self) -> f64 {
        let keys = intersection(self.table.keys(), rhs.table.keys());
        let mut acc = 0.0;
        for key in keys.iter() {
            acc += (self.table.get(key).unwrap() * rhs.table.get(key).unwrap()) as f64;
        }
        acc
    }
}

// Lazy static will compile the regular expressions once at the first use,
// rather than every time it gets called. The lazy static macro has to be used
// because rust doesn't like static / global variables.
lazy_static! {
    /// Roughly recognizes URLs. NOTE: this regex is awful.
    static ref URL: Regex = Regex::new(r#"(https://|http://)?[a-zA-Z\-]+\.([a-zA-Z]{2,3})(/\S+)?"#).unwrap();
    /// Recognizes html escaped characters, e.g. &amp;. Twitter likes to html
    /// escape some characters even though it's response is in json
    static ref ESCAPED_HTML: Regex = Regex::new(r#"&.+;"#).unwrap();
}

/// Strips text of all non alpha-numeric-emoji-or-@-#-'-_ characters.
fn strip_text(x: &str) -> String {
    let mut s = String::with_capacity(x.len());
    let z = URL.replace_all(x, "");
    let y = ESCAPED_HTML.replace_all(&z, "");
    let ret = y.chars()
        .fold(&mut s, |mut acc, c| {
            match c {
                'A'...'Z' | 'a'...'z' | '0'...'9' | '‼' ... '🧀' |
                '_' | '@' | '#' | '\'' | ' ' => acc.push(c),
                _ => {},
            }
            acc
        }).chars()
        .flat_map(char::to_lowercase)
        .collect::<String>();
    ret
}

/// A struct that contains a tweet, the handle of the person who tweeted it,
/// and the tweet itself that has been passed through the strip_text function.
/// This is mostly to avoid calling strip_text more than once per tweet,
/// otherwise the Tweet struct in egg_mode would be fine.
pub struct Tweet {
    //pub tweet: tweet::Tweet,
    pub stripped_text: String,
    pub text: String,
    pub id: u64,
    pub handle: String
}

impl Tweet {
    /// A new tweet from an egg_mode tweet. This consumes the tweet::Tweet
    pub fn new(tweet: tweet::Tweet) -> Tweet {
        let stripped_text = strip_text(&tweet.text);
        let user = {
            if let Some(ref x) = tweet.user {
                x.screen_name.clone()
            } else {
                "".to_string()
            }
        };
        Tweet {
            stripped_text: stripped_text,
            text: tweet.text,
            handle: user,
            id: tweet.id
        }
    }
}

impl Serialize for Tweet {
    type DeserializeOutput = Tweet;
    fn serialize(&self, to: &mut Write) -> Result<(), io::Error> {
        if let Err(e) = self.id.serialize(to) {
            return Err(e);
        };
        if let Err(e) = self.stripped_text.serialize(to) {
            return Err(e);
        };
        if let Err(e) = self.text.serialize(to) {
            return Err(e);
        };
        if let Err(e) = self.handle.serialize(to) {
            return Err(e);
        };
        Ok(())
    }
    fn deserialize(from: &mut Read) -> Result<Self, io::Error> {
        let id = u64::deserialize(from);
        let stripped_text = String::deserialize(from);
        let text = String::deserialize(from);
        let handle = String::deserialize(from);
        if id.is_err() || stripped_text.is_err() || text.is_err() || handle.is_err() {
            if let Err(e) = id {
                return Err(e);
            }
            if let Err(e) = stripped_text {
                return Err(e);
            }
            if let Err(e) = text {
                return Err(e);
            }
            if let Err(e) = handle {
                return Err(e);
            }
        }
        Ok(Tweet {
            stripped_text: stripped_text.unwrap(),
            id: id.unwrap(),
            text: text.unwrap(),
            handle: handle.unwrap()
        })
    }
    fn serialized_len(&self) -> u64 {
        self.stripped_text.serialized_len() +
        self.text.serialized_len() +
        self.id.serialized_len() +
        self.handle.serialized_len()
    }
}

pub struct TweetList(pub Vec<Tweet>);

impl Serialize for TweetList {
    type DeserializeOutput = TweetList;
    fn serialize(&self, to: &mut Write) -> Result<(), io::Error> {
        let TweetList(ref x) = *self;
        let _ = (x.len() as u64).serialize(to);
        for i in x {
            let _ = i.serialize(to);
        }
        Ok(())
    }
    fn deserialize(from: &mut Read) -> Result<Self, io::Error> {
        let mut r = Vec::new();
        let len = u64::deserialize(from);
        if len.is_err() {
            return Err(len.err().unwrap());
        }
        for _ in 0..len.unwrap() {
            let t = Tweet::deserialize(from);
            if t.is_err() {
                return Err(t.err().unwrap());
            }
            r.push(t.unwrap());
        }
        Ok(TweetList(r))
    }
    fn serialized_len(&self) -> u64 {
        let TweetList(ref x) = *self;
        let mut len = 8u64;
        for i in 0..x.len() {
            len += x[i].serialized_len();
        }
        len
    }
}

#[allow(non_snake_case)]
#[allow(dead_code)]
/// A functional cosine similarity function that is not used (as of right now).
/// (AB)^2 / (A^2 + B^2)^.5 or something like that
fn cos_similarity(astr: &str, bstr: &str) -> f64 {
    let a = astr.as_bytes();
    let b = bstr.as_bytes();
    let sum_a = a.iter().fold(0f64, |sum, c| sum + *c as f64 * *c as f64).sqrt();
    let sum_b = b.iter().fold(0f64, |sum, c| sum + *c as f64 * *c as f64).sqrt();

    let denom = sum_a * sum_b;

    if denom == 0.0 { return 0f64 }

    let min = if a.len() > b.len() { b.len() } else { a.len() };

    let num = (0..min).fold(0f64, |acc, i| acc + (a[i] as f64 * b[i] as f64) as f64);

    num / denom
}

#[derive(Clone, Copy)]
struct PolarCoord {
    r: f64,
    a: f64,
}

impl PolarCoord {
    pub fn new(r: f64, a: f64) -> PolarCoord {
        PolarCoord { r: r, a: a }
    }
    pub fn dist(&self, other: &Self) -> f64 {
        (self.r + other.r - 2.0 * self.r * other.r * (other.a - self.a).cos()).abs().sqrt()
    }
}

/// A helper struct used to process the tweets.
pub struct TweetProcessor {
    pub map: TwitHash<String, FrequencyTable>,
}

impl TweetProcessor {

    /// Creates a new empty TweetProcessor
    pub fn new() -> TweetProcessor {
        TweetProcessor { map: TwitHash::new() }
    }

    pub fn k_means_groups(&self, k: usize, itters: usize) -> Vec<Vec<String>> {
        let mut keys = self.map.keys();
        let mut groups: Vec<f64> = Vec::with_capacity(k);

        for i in 0..k {
            let t = self.map.get(keys[i]).unwrap();
            groups.push(t.abs());
        }

        let mut count = 0;
        let mut members = vec![0i32; k];
        let mut averages: Vec<f64> = Vec::with_capacity(k);
        let mut abs_hash = TwitHash::new();

        for key in keys.iter() {
            abs_hash.insert(key, unsafe { mem::transmute::<f64, u64>(self.map.get(&key).unwrap().abs()) });
        }
        loop {
            averages.drain(..);
            for i in 0..k {
                averages.push(0.0);
                members[i] = 0;
            }
            for key in keys.iter() {
                let p = unsafe { mem::transmute::<u64, f64>(*abs_hash.get(&key).unwrap()) };
                let mut closest_dist = (groups[0] - p).abs();
                let mut val = 0;
                for i in 1..k {
                    let dist = (groups[i] - p).abs();
                    if closest_dist > (groups[i] - p).abs() {
                        closest_dist = dist;
                        val = i;
                    }
                }
                members[val] += 1;
                averages[val] += p;
            }

            let mut delta = 0.0;

            for i in 0..k {
                averages[i] = averages[i] / members[i] as f64;
                delta += (averages[i] - groups[i]).abs();
                groups[i] = averages[i];
            }

            if delta < 0.01 { break; }
            log!("Itter {}", count);
            count += 1;
            if count > itters {
                break;
            }
        }

        let mut final_groups = vec![Vec::new(); k];

        for key in keys.iter() {
            let p = unsafe { mem::transmute::<u64, f64>(*abs_hash.get(&key).unwrap()) };
            let mut ind = 0;
            let mut closest_dist = ((p - groups[0])).abs();
            for i in 1..k {
                let dist = (groups[i] - p).abs();
                if closest_dist > (groups[i] - p).abs() {
                    closest_dist = dist;
                    ind = i;
                }
            }
            final_groups[ind].push((*key).to_string());
        }
        final_groups
    }

    /// Adds a tweet to the proper FrequencyTable (if it exists, otherwise it is
    /// created).
    pub fn process_tweet(&mut self, t: &Tweet) {
        if !self.map.contains_key(&t.handle) {
            let copy = t.handle.clone();
            self.map.insert(copy, FrequencyTable::new());
        }
        self.map.get_mut(&t.handle).unwrap().add_tweet(t);
    }

    /// This is the similarity metric!
    /// It uses the compare method in the FrequencyTable to find the most similar
    /// (or in most cases, the least different) tweeter.
    pub fn closest_key(&self, s: &String) -> String {
        if self.map.is_empty() {
            panic!("Can't call closest key on an empty map");
        }
        log_file!("Comparisons to {}:\n", s);
        let t = self.map.keys();
        let mut closest = if *s == *t[0] { t[1] } else { t[0] };
        let mut closest_sim = self.map.get(s).unwrap().compare(self.map.get(closest).unwrap());
        for key in t.into_iter() {
            let x = self.map.get(key).unwrap().compare(self.map.get(s).unwrap());
            if x == 0.0 { continue; }
            if *s == *key { continue; }
            if x > closest_sim {
                closest = key;
                closest_sim = x;
            }
        }
        closest.to_string()
    }
}
