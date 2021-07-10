use std::fs::File;
use std::io::{Read, Write};
use std::io;
#[macro_use]
extern crate clap;
use clap::Arg;
fn pause() {
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    // We want the cursor to stay at the end of the line, so we print without a newline and flush manually.
    println!("Press any key to continue...");
    stdout.flush().expect("Could not flush stdout!");

    // Read a single byte and discard
    let _ = stdin.read(&mut [0u8]).expect("Could not read string from stdin!");
}
struct RLE<const W: usize, const H: usize> {
    rules_survival: [bool; 9],
    rules_birth: [bool; 9],
    map: [[bool; W]; H]
}
impl<const W:usize, const H:usize> RLE<W, H> {
    pub fn new(filename: &str) -> Self {
        let mut raw_data = String::new();
        {
            let mut f = File::open(filename)
                .expect("Could not open file for reading!");
                f.read_to_string(&mut raw_data)
                    .expect("Could not read string from file!");
        };
        let mut map = [[false; W]; H];
        let mut rules_birth = [false; 9];
        let mut rules_survival = [false; 9];
        let mut rules_touched = false;
        let mut first_line = true;
        let mut x_offset = 0;
        let mut x = 0;
        let mut y = 0;
        let mut repeat = 0;
        let mut do_repeat = false;
        'mainloop: for line in raw_data.lines() {
            if line.starts_with("#") {
                continue;
            }
            if first_line {
                first_line = false;
                for thing in line.split(",") {
                    let sides = thing.split("=").collect::<Vec<&str>>();
                    let lhs = sides[0].trim();
                    let rhs = sides[1].trim();
                    match lhs {
                        "rule" => {
                            rules_touched = true;
                            for rule in rhs.split("/") {
                                let prefix = rule.chars().next().expect("Unexpected rule ending");
                                let suffix = rule.strip_prefix(prefix).expect("Could not strip prefix from rule!");
                                let dest = match prefix {
                                    'S' => Ok(&mut rules_survival),
                                    'B' => Ok(&mut rules_birth),
                                    _ => Err("Invalid rule prefix!")
                                }.unwrap();
                                for a in suffix.chars() {
                                    let num = a.to_digit(10)
                                        .expect("Could not parse rule digit!");
                                    if num == 9 {
                                        panic!("Rule neighbor count out of range!");
                                    } else {
                                        dest[num as usize] = true;
                                    }
                                }
                            }
                        }
                        "x" => {
                            let num = rhs.parse::<u32>()
                                .expect("Invalid x parameter!");
                            if num > W as u32 {
                                panic!("Pattern too wide for program!");
                            }
                            x_offset = ((W / 2) - num as usize);
                            x = x_offset;
                        }
                        "y" => {
                            let num = rhs.parse::<u32>()
                                .expect("Invalid y parameter!");
                            if num > W as u32{
                                panic!("Pattern too long for program!");
                            }
                            y = ((H / 2) - num as usize);
                        }
                        _ => {

                        }
                    }
                }
                continue;
            }
            for char in line.chars() {
                match char.to_digit(10) {
                    Some(num) => {
                        if !do_repeat {
                            repeat = 0;
                        }
                        repeat *= 10;
                        repeat += num;
                        do_repeat = true;
                    }
                    None => {
                        match char {
                            '!' => {
                                break 'mainloop;
                            }
                            _ => {
                                for _ in 0..(match do_repeat{
                                    false => 1,
                                    true => repeat
                                }) {
                                    match char {
                                        'b' => {
                                            x += 1;
                                            x %= W;
                                        }
                                        'o' => {
                                            map[y][x] = true;
                                            x += 1;
                                            x %= W;
                                        }
                                        '$' => {
                                            x = x_offset;
                                            y += 1;
                                            y %= H;
                                        }
                                        _ => {}
                                    }
                                }
                                do_repeat = false;
                            }
                        }
                    }
                }

            }
        }
        if !rules_touched {
            rules_birth = [false, false, false, true, false, false, false, false, false];
            rules_survival = [false, false, true, true, false, false, false, false, false];
        }
        return RLE {
            map,
            rules_birth,
            rules_survival
        }
    }
    pub fn do_iteration(&mut self) {
        self.print_grid();
        let mut return_value = [[false;W];H];
        let offsets = [
            [-1 as isize,1],
            [-1,0],
            [-1,-1],
            [0,1],
            [0,-1],
            [1,1],
            [1,0],
            [1,-1]
        ];
        for x in 0..W {
            for y in 0..H {
                let mut neighs = 0;
                for offset in offsets {
                    if self.map[((y as isize + offset[0] + H as isize) as usize % H) as usize][((x as isize + offset[1] + W as isize) as usize % W) as usize] {
                        neighs += 1;
                    }
                }
                return_value[y as usize][x as usize] = match self.map[y as usize][x as usize] {
                    true => {
                        self.rules_survival[neighs]
                    }
                    false => {
                        self.rules_birth[neighs]
                    }
                }
            }
        }
        self.map = return_value;
    }
    fn print_grid(&self) {
        pause();
        print!("\x1B[2J\x1B[1;1H"); // clear the screen
        for y in 0..H {
            for x in 0..W {
                print!("{}", match self.map[y][x] {
                    true => "*",
                    false => " "
                });
            }
            println!();
        }
    }
}
fn main() {
    let cfg = app_from_crate!()
        .arg(Arg::with_name("file")
            .required(false)
            .default_value("thing.rle")
            .help("the RLE file to execute")
            .index(1)
        ).get_matches();
    let filename = cfg.value_of("file")
        .expect("No filename found (somehow!)");
    let mut rle = RLE::<16,16>::new(filename);
    loop {
        rle.do_iteration();
    }
}