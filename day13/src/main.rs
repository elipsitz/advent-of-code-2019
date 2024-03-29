use std::fs::File;
use std::io::{BufRead, BufReader};
use crate::MachineStatus::{BadOpcode, Finished, Blocked};
use std::collections::HashMap;
use std::cmp::Ordering;



fn read_lines(filename: &str) -> impl Iterator<Item=String> {
    let file = File::open(filename).unwrap();
    let reader = BufReader::new(file);

    reader.lines().map(|line| line.unwrap())
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum MachineStatus {
    Runnable,
    Blocked,
    Finished,
    BadOpcode(i64),
}

struct Machine {
    mem: Vec<i64>,
    pos: usize,
    inputs: Vec<i64>,
    outputs: Vec<i64>,
    input_pos: usize,
    output_pos: usize,
    status: MachineStatus,
    relative_base: i64,
}

impl Machine {
    fn new(mem: &Vec<i64>) -> Machine {
        let mut new_mem = Vec::new();
        new_mem.extend(mem);
        for _ in 0..1000 {
            new_mem.push(0);
        }

        Machine {
            mem: new_mem,
            pos: 0,
            inputs: Vec::new(),
            outputs: Vec::new(),
            input_pos: 0,
            output_pos: 0,
            status: MachineStatus::Runnable,
            relative_base: 0,
        }
    }

    fn arg(&mut self, arg: usize) -> &mut i64 {
        let addressing: i64 = self.mem[self.pos] / 100;
        let mode = (addressing / 10_i64.pow(arg as u32)) % 10;
        match mode {
            0 => {
                let addr = self.mem[self.pos + 1 + arg];
                &mut self.mem[addr as usize]
            },
            1 => &mut self.mem[self.pos + 1 + arg],
            2 => {
                let val = self.mem[self.pos + 1 + arg];
                &mut self.mem[(self.relative_base + val) as usize]
            }
            _ => { panic!(); }
        }
    }

    fn run(&mut self) {
        match self.status {
            BadOpcode(_) => { return; },
            Finished => { return; },
            _ => {}
        }

        loop {
            let opcode = self.mem[self.pos] % 100;
            // println!("raw: {}, pos: {}, opcode: {}, addressing: {}", mem[pos], pos, opcode, addressing);

            match opcode {
                1 => {
                    let a = *self.arg(0);
                    let b = *self.arg(1);
                    *self.arg(2) =  a + b;
                    self.pos += 4;
                }
                2 => {
                    let a = *self.arg(0);
                    let b = *self.arg(1);
                    *self.arg(2) = a * b;
                    self.pos += 4;
                }
                3 => {
                    if self.input_pos < self.inputs.len() {
                        let val = self.inputs[self.input_pos];
                        self.input_pos += 1;
                        *self.arg(0) = val;
                        self.pos += 2;
                    } else {
                        self.status = Blocked;
                        return;
                    }
                }
                4 => {
                    let val = *self.arg(0);
                    self.outputs.push(val);
                    self.pos += 2;
                }
                5 => {
                    let cond = *self.arg(0);
                    let target = *self.arg(1);
                    if cond != 0 {
                        self.pos = target as usize;
                    } else {
                        self.pos += 3;
                    }
                }
                6 => {
                    let cond = *self.arg(0);
                    let target = *self.arg(1);
                    if cond == 0 {
                        self.pos = target as usize;
                    } else {
                        self.pos += 3;
                    }
                }
                7 => {
                    let a = *self.arg(0);
                    let b = *self.arg(1);
                    let val = (a < b) as i64;
                    *self.arg(2) = val;
                    self.pos += 4;
                }
                8 => {
                    let a = *self.arg(0);
                    let b = *self.arg(1);
                    let val = (a == b) as i64;
                    *self.arg(2) = val;
                    self.pos += 4;
                }
                9 => {
                    let val = *self.arg(0);
                    self.relative_base += val;
                    self.pos += 2;
                }
                99 => {
                    self.status = Finished;
                    return;
                }
                _ => {
                    self.status = BadOpcode(opcode);
                    return;
                }
            }
        }
    }

    fn easy_run(&mut self, inputs: &Vec<i64>) -> &Vec<i64> {
        self.add_inputs(inputs);
        self.run();
        &self.outputs
    }

    fn add_input(&mut self, input: i64) {
        self.inputs.push(input);
    }

    fn add_inputs(&mut self, inputs: &Vec<i64>) {
        self.inputs.extend(inputs);
    }

    fn get_output(&mut self) -> Option<i64> {
        if self.output_pos < self.outputs.len() {
            let val = self.outputs[self.output_pos];
            self.output_pos += 1;
            Some(val)
        } else {
            None
        }
    }

    fn get_status(&self) -> MachineStatus {
        self.status
    }
}

struct World {
    machine:  Machine,
    tiles: HashMap<(i64, i64), i64>,
    score: i64,

    paddle_x: i64,
    ball_x: i64,
}

impl World {
    fn new(machine: Machine) -> World {
        World {
            machine,
            tiles: HashMap::new(),
            score: 0,
            paddle_x: 0,
            ball_x: 0,
        }
    }

    fn print(&self) {
        let (min_x, max_x) = range(self.tiles.keys().map(|(x, _)| *x));
        let (min_y, max_y) = range(self.tiles.keys().map(|(_, y)| *y));

        for y in min_y..(max_y + 1) {
            for x in min_x..(max_x + 1) {
                let tile = *self.tiles.get(&(x, y)).unwrap_or(&0);
                print!("{}", match tile {
                    0 => " ",
                    1 => "#",
                    2 => "x",
                    3 => "-",
                    4 => "o",
                    _ => "?",
                });
            }
            println!();
        }
        println!("Score: {}", self.score);
    }

    fn process(&mut self) {
        self.machine.run();
        let output = &self.machine.outputs;

        for chunk in output.chunks(3) {
            let x = chunk[0];
            let y = chunk[1];
            let tile = chunk[2];
            self.tiles.insert((x, y), tile);
        }
    }

    fn play(&mut self) {
        self.machine.mem[0] = 2;

        loop {
            self.machine.outputs.clear();
            self.machine.run();
            let output = &self.machine.outputs;

            // Process output.
            for chunk in output.chunks(3) {
                let x = chunk[0];
                let y = chunk[1];
                let tile = chunk[2];
                if x == -1 && y == 0 {
                    self.score = tile;
                } else {
                    self.tiles.insert((x, y), tile);

                    if tile == 3 {
                        self.paddle_x = x;
                    }
                    if tile == 4 {
                        self.ball_x = x;
                    }
                }
            }

            if self.count_blocks() == 0 {
                return;
            }

            if self.machine.status == MachineStatus::Finished {
                println!("Finished.");
                return;
            }

            self.machine.add_input(match self.ball_x.cmp(&self.paddle_x) {
                Ordering::Less => -1,
                Ordering::Equal => 0,
                Ordering::Greater => 1,
            });
        }
    }

    fn count_blocks(&self) -> usize {
        self.tiles.values().filter(|x| **x == 2).count()
    }
}

fn range(nums: impl Iterator<Item=i64> + std::clone::Clone) -> (i64, i64) {
    let min = nums.clone().min().unwrap();
    let max = nums.clone().max().unwrap();
    (min, max)
}

fn main() {
    let line = read_lines("input.in").nth(0).unwrap();
    let mem: Vec<i64> = line.split(",").map(|x| x.parse::<i64>().unwrap()).collect();

    // Part 1.
    let machine = Machine::new(&mem);
    let mut world = World::new(machine);
    world.process();
    println!("Num Blocks: {}", world.count_blocks());
    world.print();

    // Part 2.
    let machine = Machine::new(&mem);
    let mut world = World::new(machine);
    world.play();
    world.print();
}
