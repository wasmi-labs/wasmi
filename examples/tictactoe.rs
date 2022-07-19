extern crate parity_wasm;
extern crate wasmi;

use std::{env, fmt, fs::File};
use wasmi::{
    Error as InterpreterError,
    Externals,
    FuncInstance,
    FuncRef,
    HostError,
    ImportsBuilder,
    ModuleImportResolver,
    ModuleInstance,
    ModuleRef,
    RuntimeArgs,
    RuntimeValue,
    Signature,
    Trap,
    ValueType,
};

#[derive(Debug)]
pub enum Error {
    OutOfRange,
    AlreadyOccupied,
    Interpreter(InterpreterError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<InterpreterError> for Error {
    fn from(e: InterpreterError) -> Self {
        Error::Interpreter(e)
    }
}

impl HostError for Error {}

mod tictactoe {
    use super::Error;

    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub enum Player {
        X,
        O,
    }

    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub enum GameResult {
        Draw,
        Won(Player),
    }

    impl Player {
        pub fn into_i32(maybe_player: Option<Player>) -> i32 {
            match maybe_player {
                None => 0,
                Some(Player::X) => 1,
                Some(Player::O) => 2,
            }
        }
    }

    #[derive(Debug)]
    pub struct Game {
        board: [Option<Player>; 9],
    }

    impl Game {
        pub fn new() -> Game {
            Game { board: [None; 9] }
        }

        pub fn set(&mut self, idx: i32, player: Player) -> Result<(), Error> {
            if !(0..9).contains(&idx) {
                return Err(Error::OutOfRange);
            }
            if self.board[idx as usize] != None {
                return Err(Error::AlreadyOccupied);
            }
            self.board[idx as usize] = Some(player);
            Ok(())
        }

        pub fn get(&self, idx: i32) -> Result<Option<Player>, Error> {
            if !(0..9).contains(&idx) {
                return Err(Error::OutOfRange);
            }
            Ok(self.board[idx as usize])
        }

        pub fn game_result(&self) -> Option<GameResult> {
            // 0, 1, 2
            // 3, 4, 5
            // 6, 7, 8
            let patterns = &[
                // Rows
                (0, 1, 2),
                (3, 4, 5),
                (6, 7, 8),
                // Columns
                (0, 3, 6),
                (1, 4, 7),
                (2, 5, 8),
                // Diagonals
                (0, 4, 8),
                (2, 4, 6),
            ];

            // Returns Some(player) if all cells contain same Player.
            let all_same = |i1: usize, i2: usize, i3: usize| -> Option<Player> {
                self.board[i1]?;
                if self.board[i1] == self.board[i2] && self.board[i2] == self.board[i3] {
                    return self.board[i1];
                }
                None
            };

            for &(i1, i2, i3) in patterns {
                if let Some(player) = all_same(i1, i2, i3) {
                    return Some(GameResult::Won(player));
                }
            }

            // Ok, there is no winner. Check if it's draw.
            let all_occupied = self.board.iter().all(|&cell| cell.is_some());
            if all_occupied {
                Some(GameResult::Draw)
            } else {
                // Nah, there are still empty cells left.
                None
            }
        }
    }
}

struct Runtime<'a> {
    player: tictactoe::Player,
    game: &'a mut tictactoe::Game,
}

const SET_FUNC_INDEX: usize = 0;
const GET_FUNC_INDEX: usize = 1;

impl<'a> Externals for Runtime<'a> {
    fn invoke_index(
        &mut self,
        index: usize,
        args: RuntimeArgs,
    ) -> Result<Option<RuntimeValue>, Trap> {
        match index {
            SET_FUNC_INDEX => {
                let idx: i32 = args.nth(0);
                self.game.set(idx, self.player)?;
                Ok(None)
            }
            GET_FUNC_INDEX => {
                let idx: i32 = args.nth(0);
                let val: i32 = tictactoe::Player::into_i32(self.game.get(idx)?);
                Ok(Some(val.into()))
            }
            _ => panic!("unknown function index"),
        }
    }
}

struct RuntimeModuleImportResolver;

impl ModuleImportResolver for RuntimeModuleImportResolver {
    fn resolve_func(
        &self,
        field_name: &str,
        _signature: &Signature,
    ) -> Result<FuncRef, InterpreterError> {
        let func_ref = match field_name {
            "set" => FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32][..], None),
                SET_FUNC_INDEX,
            ),
            "get" => FuncInstance::alloc_host(
                Signature::new(&[ValueType::I32][..], Some(ValueType::I32)),
                GET_FUNC_INDEX,
            ),
            _ => {
                return Err(InterpreterError::Function(format!(
                    "host module doesn't export function with name {}",
                    field_name
                )));
            }
        };
        Ok(func_ref)
    }
}

fn instantiate(path: &str) -> Result<ModuleRef, Error> {
    let module = {
        use std::io::prelude::*;
        let mut file = File::open(path).unwrap();
        let mut wasm_buf = Vec::new();
        file.read_to_end(&mut wasm_buf).unwrap();
        wasmi::Module::from_buffer(&wasm_buf)?
    };

    let mut imports = ImportsBuilder::new();
    imports.push_resolver("env", &RuntimeModuleImportResolver);

    let instance = ModuleInstance::new(&module, &imports)?.assert_no_start();

    Ok(instance)
}

fn play(
    x_instance: ModuleRef,
    o_instance: ModuleRef,
    game: &mut tictactoe::Game,
) -> Result<tictactoe::GameResult, Error> {
    let mut turn_of = tictactoe::Player::X;
    let game_result = loop {
        let (instance, next_turn_of) = match turn_of {
            tictactoe::Player::X => (&x_instance, tictactoe::Player::O),
            tictactoe::Player::O => (&o_instance, tictactoe::Player::X),
        };

        {
            let mut runtime = Runtime {
                player: turn_of,
                game,
            };
            let _ = instance.invoke_export("mk_turn", &[], &mut runtime)?;
        }

        if let Some(game_result) = game.game_result() {
            break game_result;
        }

        turn_of = next_turn_of;
    };

    Ok(game_result)
}

fn main() {
    let mut game = tictactoe::Game::new();

    let args: Vec<_> = env::args().collect();
    if args.len() < 3 {
        println!("Usage: {} <x player module> <y player module>", args[0]);
        return;
    }

    // Instantiate modules of X and O players.
    let x_instance = instantiate(&args[1]).expect("X player module to load");
    let o_instance = instantiate(&args[2]).expect("Y player module to load");

    let result = play(x_instance, o_instance, &mut game);
    println!("result = {:?}, game = {:#?}", result, game);
}
