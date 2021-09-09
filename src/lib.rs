use std::io::{self, prelude::*};
use std::collections::{HashMap, HashSet};
use bimap::BiMap;
use rand::Rng;

// Set to true to enable debug messages
const DEBUG: bool = false;

// Round a u8 to the nearest multiple of 5
fn cell_round(n: u8) -> u8 {
    let o = n % 5;
    let n = 5*(n/5);
    if o <= 2 {
        n
    } else {
        n.wrapping_add(5)
    }
}

// The tape
pub struct Tape {
    data_l: Vec<u8>, // stack to the left of the head
    data_r: Vec<u8>, // stack to the right of the head
    cell: u8, // cell at head
}

impl Tape {
    pub fn new() -> Self {
        Self { data_l: vec![], data_r: vec![], cell: 0 }
    }

    // Get current cell
    pub fn get(&self) -> u8 { self.cell }

    // Set current cell
    pub fn set(&mut self, val: u8) { self.cell = val }

    // Get next cell
    pub fn get_next(&self) -> u8 {
        *self.data_r.last().unwrap_or(&0)
    }

    // Set next cell
    pub fn set_next(&mut self, val: u8) {
        if self.data_r.len() > 0 {
            *self.data_r.last_mut().unwrap() = val
        } else {
            self.data_r.push(val)
        }
    }

    // Move the head right
    pub fn next(&mut self) {
        self.data_l.push(self.cell);
        self.cell = self.data_r.pop().unwrap_or(0);
    }

    // Move the head left
    pub fn prev(&mut self) {
        self.data_r.push(self.cell);
        self.cell = self.data_l.pop().unwrap_or(0);
    }

    // Insert a new cell on the left/right side
    pub fn insert_left(&mut self, val: u8) { self.data_l.push(val); }
    pub fn insert_right(&mut self, val: u8) { self.data_r.push(val); }
    // Delete a cell from the left or right side
    pub fn delete_left(&mut self) -> u8 { self.data_l.pop().unwrap_or(0) }
    pub fn delete_right(&mut self) -> u8 { self.data_r.pop().unwrap_or(0) }

    // Duplicate each cell
    pub fn expand_2(&mut self) {
        self.data_l = (&mut self.data_l).into_iter().map(|x| vec![*x,*x]).flatten().collect();
        self.data_r = (&mut self.data_r).into_iter().map(|x| vec![*x,*x]).flatten().collect();
        self.data_r.push(self.cell);
    }

    // Triplicate each cell
    pub fn expand_3(&mut self) {
        self.data_l = (&mut self.data_l).into_iter().map(|x| vec![*x,*x,*x]).flatten().collect();
        self.data_r = (&mut self.data_r).into_iter().map(|x| vec![*x,*x,*x]).flatten().collect();
        self.data_r.push(self.cell);
        self.data_r.push(self.cell);
    }

    // For each cell that has been accessed, 50% chance of adding a number in -5..=5
    pub fn randomize(&mut self) {
        let mut rng = rand::thread_rng();
        self.data_l = (&mut self.data_l).into_iter().map(|x|
            if rand::random() {
                (*x as i16 + rng.gen_range(-5..=5)) as u8
            } else {
                *x
            }).collect();
        self.data_r = (&mut self.data_l).into_iter().map(|x| 
            if rand::random() {
                (*x as i16 + rng.gen_range(-5..=5)) as u8
            } else {
                *x
            }).collect();
        if rand::random() {
            self.cell = (self.cell as i16 + rng.gen_range(-5..=5)) as u8;
        }
    }
}

// Table containing function names and contents
pub struct FnTable {
    funcs: HashMap<char, (Vec<char>, usize)>,
    creating: HashSet<char>
}

impl FnTable {
    pub fn new() -> Self {
        Self { funcs: HashMap::new(), creating: HashSet::new() }
    }
    // When a new char is encountered, add it to all active functions
    pub fn put(&mut self, c: char) {
        for f in &self.creating {
            self.funcs.get_mut(f).unwrap().0.push(c)
        }
    }
    // Is a function defined
    pub fn exists(&self, c: char) -> bool {
        self.funcs.contains_key(&c)
    }
    // Is a function in the process of being created
    pub fn is_creating(&self, c: char) -> bool {
        self.creating.contains(&c)
    }
    pub fn get(&self, c: char) -> Option<&(Vec<char>, usize)> {
        if self.creating.contains(&c) { None }
        else { self.funcs.get(&c) }
    }
    // Begin a new function
    pub fn begin(&mut self, c: char, i: usize) {
        self.funcs.insert(c, (vec![], i));
        self.creating.insert(c);
    }
    // End a function
    pub fn end(&mut self, c: char) {
        self.creating.remove(&c);
    }
    // Returns true if there is a function being created
    pub fn any_creating(&self) -> bool {
        !self.creating.is_empty()
    }
    // copies one function to another
    pub fn copy_fn(&mut self, from: char, to: char) {
        if let Some(x) = self.funcs.get(&from).cloned() {
            self.funcs.insert(to, x);
        }
    }
}

// Generate a BiMap between positions of opening and closing pairs of symbols for use later
fn gen_index_table(code: &[char]) -> Result<BiMap<usize, usize>, &'static str> {
    let mut map: BiMap<usize, usize> = BiMap::new();
    let mut brackstack: Vec<usize> = vec![];
    let mut last_comment: Option<usize> = None;
    let mut last_quote: Option<usize> = None;
    let mut last_percent: Option<usize> = None;
    let mut last_zero: Option<usize> = None;
    for (i,c) in code.iter().enumerate() {
        match (c, last_comment.is_none(), last_quote.is_none()) {
            ('^',_,_) => match last_comment {
                None => last_comment = Some(i),
                Some(o) => {
                    last_comment = None;
                    map.insert(o, i);
                }
            },
            ('"',true,_) => match last_quote {
                None => last_quote = Some(i),
                Some(o) => {
                    last_quote = None;
                    map.insert(o, i);
                }
            },
            ('%',true,true) => match last_percent {
                None => last_percent = Some(i),
                Some(o) => {
                    last_percent = None;
                    map.insert(o, i);
                }
            },
            ('0',true,true) => match last_zero {
                None => last_zero = Some(i),
                Some(o) => {
                    last_zero = None;
                    map.insert(o, i);
                }
            },
            ('[',true,true) => brackstack.push(i),
            (']',true,true) => { 
                let o = brackstack.pop().ok_or("mismatched brackets")?; 
                map.insert(o, i); 
            },
            _ => ()
        }
    }
    Ok(map)
}

// Offset the index table BiMap for new run() calls
fn offset_index_table(itable: &BiMap<usize, usize>, start: usize) -> BiMap<usize, usize> {
    itable.iter().filter(|(&l, &r)| l >= start && r >= start).map(|(l, r)| (l-start, r-start)).collect()
}

// Not functions
const RESERVED_CHARS: &'static str = "<>{}[]()+-*/!.,[]\\#?$&@\"`~|;^:'_%=0123456789 \n\t";
const BFMODE_ALLOW: &'static str = "<>+-[].,_ \n\t";

// ooh boy
fn run(
    code: Vec<char>, 
    source_str: &str,
    index_table: &BiMap<usize, usize>, 
    tape: &mut Tape, 
    printed: &mut Vec<u8>, 
    fntable: &mut FnTable
) -> Result<(),&'static str> {
    // current char to execute
    let mut idx: usize = 0;
    // bfmode triggered by '_' command
    let mut bfmode = false;
    // nice mode triggered by '6' command
    let mut nicemode = false;
    while idx < code.len() {
        let c = code[idx];
        if DEBUG {
            println!("::DEBUG:: running '{}' (idx {})", c, idx);
        }
        // process each "mode"
        if nicemode {
            if c == '9' {
                nicemode = false;
            } else {
                println!("Nice.");
                printed.append(&mut vec![b'N', b'i', b'c', b'e', b'.', b'\n']);
            }
            idx += 1;
            continue;
        } else if bfmode {
            if !BFMODE_ALLOW.contains(c) {
                idx += 1;
                continue;
            }
        } else if !RESERVED_CHARS.contains(c) {
            if idx+2 < code.len() && code[idx+1] == '=' {
                let fn1 = c;
                let fn2 = code[idx+2];
                fntable.copy_fn(fn2, fn1);
                idx += 3;
                continue
            }
        } else if fntable.any_creating() {
            fntable.put(c);
            idx += 1;
            continue
        }
        // if we haven't continued yet then we are in normal mode
        match c {
            '>' => tape.next(),
            '<' => tape.prev(),
            '{' => {tape.delete_left();},
            '}' => {tape.delete_right();},
            '(' => tape.insert_left(0),
            ')' => tape.insert_right(0),
            '+' => tape.set(tape.get().wrapping_add(1)),
            '-' => tape.set(tape.get().wrapping_sub(1)),
            '*' => tape.set(tape.get().wrapping_mul(tape.get_next())),
            '/' => tape.set(tape.get().checked_div(tape.get_next()).unwrap_or(255)), // TODO div/0
            '!' => match tape.get() { 
                0 => tape.set(1), 
                1 => tape.set(0), 
                _ => () 
            },
            '.' => {
                io::stdout().write(&[tape.get()]).unwrap(); 
                io::stdout().flush().unwrap(); 
                printed.push(tape.get());
            },
            ',' => {
                tape.set(io::stdin().bytes().next().unwrap_or(Ok(0)).unwrap_or(0));
            }, 
            '[' => if tape.get() == 0 {
                idx = *index_table.get_by_left(&idx).unwrap();
            },
            ']' => if tape.get() != 0 {
                idx = *index_table.get_by_right(&idx).unwrap();
            },
            '\\' => {
                let mut i = idx;
                while i < code.len() {
                    if code[i] == ']' && index_table.get_by_right(&i).is_some() {
                        idx = i;
                        break;
                    }
                    i += 1;
                }
            },
            '#' => if tape.get() != tape.get_next() {
                idx += 1;
            },
            '?' => tape.set(rand::random()),
            '$' => if rand::random() {
                tape.next();
            } else {
                tape.prev();
            },
            '&' => if rand::random() {
                idx += 1;
            },
            '@' => return Ok(()),
            '"' => {
                let end = *index_table.get_by_left(&idx).unwrap();
                let strpart = &code[(idx+1)..end];
                for c in strpart {
                    tape.next();
                    tape.set(*c as u8);
                }
                idx = end;
            },
            '`' => tape.set(tape.get() << 1),
            '~' => tape.set(tape.get() >> 1),
            '|' => { idx = 0; continue },
            ';' => {
                print!("{}", source_str);
                printed.append(&mut source_str.as_bytes().to_vec())
            },
            '^' => idx = *index_table.get_by_left(&idx).unwrap(),
            ':' => tape.set_next(tape.get()),
            '\'' => {
                let newcode = std::str::from_utf8(printed).expect("Output is not valid UTF-8").chars().collect();
                run(newcode, source_str, &index_table, tape, printed, fntable)?;
            },
            '_' => bfmode = !bfmode,
            '%' => if let Some(o) = index_table.get_by_left(&idx) {
                if tape.get() == 0 {
                    idx = *o
                }
            } else if let Some(o) = index_table.get_by_right(&idx) {
                if tape.get() != 0 {
                    idx = *o
                }
            },
            '=' => unreachable!(), // special case covered above
            '0' => if let Some(o) = index_table.get_by_left(&idx) {
                idx = *o
            } else if let Some(o) = index_table.get_by_right(&idx) {
                idx = *o
            },
            '1' => todo!("Command '1' is not yet implemented."), // TODO 1 instruction
            '2' => tape.expand_2(),
            '3' => tape.expand_3(),
            '4' => tape.randomize(), 
            '5' => tape.set(cell_round(tape.get())),
            '6' => nicemode = true,
            '7' => todo!("Command '7' is not yet implemented"), // TODO 7 instruction
            '8' => {
                tape.prev();
                tape.prev();
                tape.prev();
                for _ in 0..7 {
                    tape.set(8);
                    tape.next();
                }
            },
            '9' => (), 
            ' ' | '\n' | '\t' => (),
            _ => {
                if let Some((func, start)) = fntable.get(c) {
                    let new_itable = offset_index_table(&index_table, *start);
                    run(func.to_vec(), source_str, &new_itable, tape, printed, fntable)?;
                } else if fntable.is_creating(c) {
                    fntable.end(c);
                } else {
                    fntable.begin(c, idx+1);
                    idx += 1;
                    continue
                }
            },
        }
        idx += 1;
    }
    Ok(())
}

// wrapper for run() that does the setup and args and stuff
pub fn exec(code: &str) -> Result<(), &'static str> {
    let chars: Vec<char> = code.chars().collect();
    let map = gen_index_table(&chars)?;
    run(chars, code, &map, &mut Tape::new(), &mut vec![], &mut FnTable::new())
}
